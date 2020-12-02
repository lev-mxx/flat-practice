use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, Write};
use std::path::Path;
use std::str::from_utf8;

use anyhow::Result;
use pyo3::Python;
use pyo3::types::PyModule;

use graphblas::*;

use super::graph::{BooleanMatrix, Ends, ExtractPairs, Graph};

#[derive(Debug)]
pub struct ContextFreeGrammar {
    pub initial: String,
    pub(crate) nonterminals: HashSet<String>,
    pub(crate) produces_epsilon: bool,
    pub(crate) unit_from_variable: HashMap<String, HashSet<String>>,
    pub(crate) pair_from_variable: HashMap<String, HashMap<String, HashSet<String>>>,
}

impl ContextFreeGrammar {

    pub(crate) fn get_producers_by_pair(&self, p1: &str, p2: &str) -> Option<&HashSet<String>>  {
        self.pair_from_variable.get(p1).and_then(|x| x.get(p2))
    }

    pub(crate) fn get_producers(&self, p: &String) -> Option<&HashSet<String>> {
        self.unit_from_variable.get(p)
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<ContextFreeGrammar> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let converted = ContextFreeGrammar::convert(reader.lines())?;

        ContextFreeGrammar::_from_text(converted.as_str())
    }

    pub fn from_text(text: &str) -> Result<ContextFreeGrammar> {
        let converted = ContextFreeGrammar::convert(text.lines().map(Ok))?;

        ContextFreeGrammar::_from_text(converted.as_str())
    }

    pub fn _from_text(text: &str) -> Result<ContextFreeGrammar> {
        let (initial, nonterminals, productions, produces_epsilon) = Python::with_gil(|py| -> Result<(String, Vec<String>, Vec<(String, Vec<String>)>, bool)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/read_cfg_in_cnf.py"))?, "a.compute.py", "a")?;
            Ok(module.call1("parse_cfg", (text,))?.extract()?)
        })?;

        let mut unit_from_variable = HashMap::<String, HashSet<String>>::new();
        let mut pair_from_variable = HashMap::<String, HashMap<String, HashSet<String>>>::new();

        for (head, body) in productions {
            let mut first = None;
            let mut second = None;
            for s in body {
                if first.is_none() {
                    first = Some(s)
                } else {
                    second = Some(s)
                }
            }

            let first = first.unwrap();
            if let Some(second) = second {
                let set = pair_from_variable
                    .entry(first)
                    .or_insert_with(|| HashMap::new())
                    .entry(second)
                    .or_insert_with(|| HashSet::new());
                set.insert(head);
            } else {
                let set = unit_from_variable
                    .entry(first)
                    .or_insert_with(|| HashSet::new());
                set.insert(head);
            }
        }

        Ok(ContextFreeGrammar {
            initial,
            nonterminals: nonterminals.into_iter().collect(),
            produces_epsilon,
            unit_from_variable,
            pair_from_variable,
        })
    }

    pub fn cyk(&self, word: &[&String]) -> bool {
        if word.is_empty() {
            return self.produces_epsilon
        }
        let mut m = vec!(vec!(HashSet::<&String>::new(); word.len()); word.len());
        word.iter().enumerate().for_each(|(i, char)| {
            if let Some(vars) = self.get_producers(char) {
                let set = &mut m[i][i];
                vars.iter().for_each(|x| { set.insert(x); });
            }
        });

        for length in 1..word.len() {
            for pos in 0..(word.len() - length) {
                let mut for_insert = HashSet::<&String>::new();
                for split in 0..(word.len() - pos) {
                    for left in &m[pos][pos + split] {
                        for right in &m[pos + split + 1][pos + length] {
                            if let Some(vars) = self.get_producers_by_pair(left, right) {
                                vars.iter().for_each(|x| { for_insert.insert(x); });
                            }
                        }
                    }
                }
                let set = &mut m[pos][pos + length];
                for_insert.iter().for_each(|x| { set.insert(x); });
            }
        }
        m[0][word.len() - 1].contains(&self.initial)
    }

    pub(crate) fn convert<'a, S: AsRef<str>, I: Iterator<Item=std::io::Result<S>>>(lines: I) -> Result<String> {
        let vec = Vec::<u8>::new();
        let mut writer = std::io::BufWriter::new(vec);

        for line in lines {
            let line = line?;
            let line = line.as_ref();
            let (head, body) = if let Some(space) = line.find(" ") {
                line.split_at(space)
            } else {
                continue
            };
            writer.write(head.as_bytes())?;
            writer.write(" -> ".as_bytes())?;
            writer.write(body.as_bytes())?;
            writer.write("\n".as_bytes())?;
        }
        writer.flush()?;

        Ok(String::from_utf8(writer.into_inner()?)?)
    }
}

impl Graph {
    pub fn cfpq_hellings<'a>(&self, cfg: &'a ContextFreeGrammar) -> ResultWithSets<'a> {
        let mut r = HashMap::<&str, HashSet<Ends>>::new();
        let mut m = VecDeque::<(Ends, &str)>::new();

        if cfg.produces_epsilon {
            let mut set = HashSet::<Ends>::new();
            for v in 0..self.size {
                set.insert((v, v));
                m.push_back(((v, v), cfg.initial.as_str()));
            }

            r.insert(cfg.initial.as_str(), set);
        }

        for (label, matrix) in &self.matrices {
            if let Some(vars) = cfg.get_producers(&label) {
                let pairs = matrix.extract_pairs();

                for var in vars {
                    for (from, to) in &pairs {
                        let set = r.entry(var).or_insert_with(HashSet::new);
                        if set.insert((*from, *to)) {
                            m.push_back(((*from, *to), var));
                        }
                    }
                }
            }
        }

        while !m.is_empty() {
            let ((v, u), n_i) = m.pop_front().unwrap();
            let mut new = HashSet::<(Ends, &str)>::new();

            for (n_j, set) in &r {
                for (v_, u_) in set {
                    if *u_ == v {
                        if let Some(n_ks) = cfg.get_producers_by_pair(n_j, n_i) {
                            for n_k in n_ks {
                                let p = ((*v_, u), n_k.as_str());
                                if let Some(set) = r.get(n_k.as_str()) {
                                    if !set.contains(&(*v_, u)) {
                                        new.insert(p);
                                    }
                                } else {
                                    new.insert(p);
                                }
                            }
                        }
                    }

                    if *v_ == u {
                        if let Some(n_ks) = cfg.get_producers_by_pair(n_i, n_j) {
                            for n_k in n_ks {
                                let p = ((v, *u_), n_k.as_str());
                                if let Some(set) = r.get(n_k.as_str()) {
                                    if !set.contains(&(v, *u_)) {
                                        new.insert(p);
                                    }
                                } else {
                                    new.insert(p);
                                }
                            }
                        }
                    }
                }
            }

            for ((from, to), var) in new {
                let set = r.entry(var).or_insert_with(HashSet::new);
                set.insert((from, to));
                m.push_back(((from, to), var));
            }
        }

        ResultWithSets {
            map: r,
            nonterminals: &cfg.nonterminals,
        }
    }

    pub fn cfpq_matrix_product<'a>(&self, cfg: &'a ContextFreeGrammar) -> ResultWithMatrices<'a> {
        let mut matrices = HashMap::<&str, BooleanMatrix>::new();
        for (body, heads) in &cfg.unit_from_variable {
            if let Some(body_matrix) = self.matrices.get(body) {
                for head in heads {
                    let matrix = matrices.entry(head).or_insert_with(|| Matrix::<bool>::new(self.size, self.size));
                    matrix.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), body_matrix);
                }
            }
        }

        if cfg.produces_epsilon {
            let matrix = matrices.entry(&cfg.initial).or_insert_with(|| Matrix::<bool>::new(self.size, self.size));
            for i in 0..self.size {
                matrix.insert(i, i, true);
            }
        }

        let mut changing = true;
        while changing {
            changing = false;
            for (left, map) in &cfg.pair_from_variable {
                for (right, heads) in map {
                    for head in heads {
                        let matrix = matrices.remove(head.as_str());
                        let mut matrix = matrix.unwrap_or_else(|| Matrix::<bool>::new(self.size, self.size));

                        let n = matrix.nvals();
                        if left == head && right == head {
                            let production = Matrix::<bool>::mxm(Semiring::<bool>::lor_land(), &matrix, &matrix);
                            matrix.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &production);
                        } else if left == head {
                            if let Some(right_matrix) = matrices.get(right.as_str()) {
                                let production = Matrix::<bool>::mxm(Semiring::<bool>::lor_land(), &matrix, &right_matrix);
                                matrix.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &production);
                            }
                        } else if right == head {
                            if let Some(left_matrix) = matrices.get(left.as_str()) {
                                let production = Matrix::<bool>::mxm(Semiring::<bool>::lor_land(), &left_matrix, &matrix);
                                matrix.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &production);
                            }
                        }else if let Some(left_matrix) = matrices.get(left.as_str()) {
                            if let Some(right_matrix) = matrices.get(right.as_str()) {
                                matrix.accumulate_mxm(BinaryOp::<bool, bool, bool>::lor(), Semiring::<bool>::lor_land(), left_matrix, right_matrix);
                            }
                        }
                        changing |= n != matrix.nvals();
                        matrices.insert(head, matrix);
                    }
                }
            }
        }

        ResultWithMatrices {
            map: matrices,
            nonterminals: &cfg.nonterminals,
        }
    }
}

pub trait ContextFreeResult {
    fn reachable_edges(&self, nonterminal: &str) -> Vec<Ends>;
    fn nonterminals(&self) -> &HashSet<String>;
}

pub struct ResultWithSets<'a> {
    pub(crate) map: HashMap<&'a str, HashSet<Ends>>,
    pub(crate) nonterminals: &'a HashSet<String>,
}

pub struct ResultWithMatrices<'a> {
    pub(crate) map: HashMap<&'a str, BooleanMatrix>,
    pub(crate) nonterminals: &'a HashSet<String>,
}

impl<'a> ContextFreeResult for ResultWithSets<'a> {
    fn reachable_edges(&self, nonterminal: &str) -> Vec<Ends> {
        if let Some(set)  = self.map.get(nonterminal) {
            set.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    fn nonterminals(&self) -> &HashSet<String> {
        self.nonterminals
    }
}

impl<'a> ContextFreeResult for ResultWithMatrices<'a> {
    fn reachable_edges(&self, nonterminal: &str) -> Vec<Ends> {
        if self.nonterminals.contains(nonterminal) {
            if let Some(matrix)  = self.map.get(nonterminal) {
                return matrix.extract_pairs();
            }
        }

        Vec::new()
    }

    fn nonterminals(&self) -> &HashSet<String> {
        self.nonterminals
    }
}

#[cfg(test)]
mod cfg {
    use std::str::from_utf8;

    use anyhow::Result;

    use super::ContextFreeGrammar;

    #[test]
    fn epsilon() -> Result<()> {
        let cfg = ContextFreeGrammar::from_text(from_utf8(include_bytes!("../../test_data/grammars/epsilon"))?)?;

        assert!(cfg.cyk(&[]));
        assert!(!cfg.cyk(&[&"a".to_string()]));
        Ok(())
    }

    #[test]
    fn none() -> Result<()> {
        let cfg = ContextFreeGrammar::from_text(from_utf8(include_bytes!("../../test_data/grammars/none"))?)?;

        assert!(!cfg.cyk(&[]));
        Ok(())
    }

    #[test]
    fn test() -> Result<()> {
        let cfg = ContextFreeGrammar::from_text(from_utf8(include_bytes!("../../test_data/grammars/operations"))?)?;

        let ref a = String::from("a");
        let ref b = String::from("b");
        let ref c = String::from("c");
        let ref n = String::from("n");

        assert!(!cfg.cyk(&[]));
        assert!(cfg.cyk(&[c, n, a, n, c, b, n]));
        assert!(cfg.cyk(&[n, a, n, b, n]));
        assert!(cfg.cyk(&[n, a, n, a, n, a, n]));
        assert!(cfg.cyk(&[n, a, c, n, b, n, c, a, n]));

        Ok(())
    }
}

#[cfg(test)]
mod cfpq {
    use std::collections::HashSet;
    use std::str::from_utf8;

    use anyhow::Result;

    use crate::compute::graph::{Ends, Graph};
    use crate::compute::rfa::Rfa;

    use super::{ContextFreeGrammar, ContextFreeResult};

    fn test<F: Fn(&str, &Graph) -> Result<Vec<Ends>>>(graph: &str, grammar: &str, fun: F, expected: &[Ends]) -> Result<()> {
        let graph = Graph::from_text(graph)?;
        let res = fun(grammar, &graph)?;
        let actual: HashSet<&Ends> = res.iter().collect();
        let expected: HashSet<&Ends> = expected.iter().collect();

        assert_eq!(expected, actual);

        Ok(())
    }

    macro_rules! test {
    ($fun: ident, $graph: expr, $grammar: expr, $expected: expr) => {
        paste::paste! {
            #[test]
            fn [<$fun _ $graph _ $grammar>]() -> Result<()> {
                test(from_utf8(include_bytes!(concat!("../../test_data/graphs/", $graph)))?,
                    from_utf8(include_bytes!(concat!("../../test_data/grammars/", $grammar)))?, $fun, $expected)
            }
        }
    };
}

    fn hellings(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
        let grammar = ContextFreeGrammar::from_text(text)?;
        Ok(graph.cfpq_hellings(&grammar).reachable_edges("S"))
    }

    fn matrices(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
        let grammar = ContextFreeGrammar::from_text(text)?;
        Ok(graph.cfpq_matrix_product(&grammar).reachable_edges("S"))
    }

    fn tensors(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
        let rfa = Rfa::from_text(text)?;
        Ok(graph.cfpq_tensor_product(&rfa).reachable_edges("S"))
    }

    test!(hellings, "graph1", "epsilon", &[(0, 0), (1, 1)]);
    test!(hellings, "graph1", "none", &[]);
    test!(hellings, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
    test!(hellings, "graph1", "grammar2", &[(0, 1)]);
    test!(hellings, "graph1", "grammar3", &[(0, 0), (0, 1)]);
    test!(hellings, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
    test!(hellings, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
    test!(hellings, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);

    test!(matrices, "graph1", "epsilon", &[(0, 0), (1, 1)]);
    test!(matrices, "graph1", "none", &[]);
    test!(matrices, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
    test!(matrices, "graph1", "grammar2", &[(0, 1)]);
    test!(matrices, "graph1", "grammar3", &[(0, 0), (0, 1)]);
    test!(matrices, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
    test!(matrices, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
    test!(matrices, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);

    test!(tensors, "graph1", "epsilon", &[(0, 0), (1, 1)]);
    test!(tensors, "graph1", "none", &[]);
    test!(tensors, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
    test!(tensors, "graph1", "grammar2", &[(0, 1)]);
    test!(tensors, "graph1", "grammar3", &[(0, 0), (0, 1)]);
    test!(tensors, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
    test!(tensors, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
    test!(tensors, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);
}
