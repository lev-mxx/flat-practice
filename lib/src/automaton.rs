use std::fs::File;
use std::io::{BufReader, BufRead, Error, ErrorKind};
use std::str::{FromStr, from_utf8};

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use anyhow::Result;

use graphblas::*;
use std::fmt::{Debug};
use pyo3::Python;
use pyo3::types::PyModule;

use crate::cfg::{ContextFreeGrammar};

#[derive(Debug)]
pub struct Automaton {
    matrices: HashMap<String, BaseTypeMatrix<bool>>,
    pub size: u64,
    pub initial_states: HashSet<u64>,
    pub final_states: HashSet<u64>,
}
pub type Endpoints = (u64, u64);

pub type Edge = (u64, u64, String);

impl Automaton {

    pub fn read_graph<P: AsRef<Path>>(path: P) -> Result<Automaton> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::<String>::new();
        for line in reader.lines() {
            let line = line?;
            lines.push(line);
        }

        Automaton::from_lines(lines)
    }

    pub fn from_text(text: &str) -> Result<Automaton> {
        Automaton::from_lines(text.to_string().split("\n").map(str::to_string).collect())
    }

    fn from_lines(lines: Vec<String>) -> Result<Automaton> {
        let mut edges = Vec::<Edge>::new();
        let mut max: u64 = 0;

        for line in lines {
            if line.is_empty() {
                continue
            }
            let split: Vec<&str> = line.split(" ").collect();
            if split.len() != 3 {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidData))?
            }

            let from = u64::from_str(split[0])?;
            if from > max { max = from }
            let to = u64::from_str(split[2])?;
            if to > max { max = to }

            let label = split[1].to_string();
            edges.push((from, to, label));
        }

        Ok(Automaton::build_graph(max + 1, edges.as_slice()))
    }

    pub fn read_regex<P: AsRef<Path>>(path: P) -> Result<Automaton> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let first_line = reader.lines().next().ok_or_else(|| Error::from(ErrorKind::InvalidInput))??;
        Automaton::from_regex(first_line.as_str())
    }

    pub fn read_query<P: AsRef<Path>>(path: P) -> Result<Automaton> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let line = reader.lines().skip(2).next().ok_or_else(|| Error::from(ErrorKind::InvalidInput))??;
        Automaton::from_regex(line.as_str())
    }

    pub fn build_graph(size: u64, edges: &[Edge]) -> Automaton {
        Automaton::from_edges(size, edges.iter().cloned(), (0..size).into_iter(), (0..size).into_iter())
    }

    pub fn from_regex(regex: &str) -> Result<Automaton> {
        let (initial, finals, edges) = Python::with_gil(|py| -> Result<(u64, Vec<u64>, Vec<(u64, u64, String)>)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/regex_to_edges.py"))?, "a.py", "a")?;
            let py_res: (u64, Vec<u64>, Vec<(u64, u64, String)>) = module.call1("regex_to_edges", (regex,))?.extract()?;
            Ok(py_res)
        })?;

        Ok(Automaton::from_edges(0, edges.iter().cloned(), [initial].iter().cloned(), finals.iter().cloned()))
    }

    fn from_edges<I, IS, IF>(size: u64, edges_it: I, initials: IS, finals: IF) -> Automaton
        where I : Iterator<Item = Edge>, IS : Iterator<Item = u64>, IF: Iterator<Item = u64> {
        let mut label_paths = HashMap::<String, (Vec<u64>, Vec<u64>)>::new();
        let mut size = size;

        edges_it.for_each(|a| {
            let (from, to, label) = a;
            if from >= size { size = from + 1 }
            if to >= size { size = to + 1 }

            let (from_vertices, to_vertices) = label_paths.entry(label.to_string()).or_insert_with(|| (Vec::new(), Vec::new()));
            from_vertices.push(from);
            to_vertices.push(to);
        });

        let mut matrices: HashMap<String, BaseTypeMatrix<bool>> = HashMap::new();
        for (label, (froms, tos)) in label_paths {
            let mut matrix = Matrix::<bool>::new(size as u64, size as u64);
            matrix.build(&froms, &tos, vec!(true; froms.len()), BinaryOp::<bool, bool, bool>::first());
            matrices.insert(label, matrix);
        }

        Automaton {
            matrices,
            size,
            initial_states: initials.collect(),
            final_states: finals.collect()
        }
    }

    pub fn intersection(&self, b: &Automaton) -> Automaton {
        let mut matrices = HashMap::<String, BaseTypeMatrix<bool>>::new();
        self.matrices.iter()
            .filter_map(|(label, m)| {
                b.matrices.get(label).map(|om| (label, m, om))
            })
            .for_each(|(label, m, om)| {
                matrices.insert(label.clone(), Matrix::kronecker(Semiring::<bool>::lor_land(), &m, &om));
            });
        let size = self.size * b.size;
        let mut initial_states = HashSet::with_capacity(self.initial_states.len() * b.initial_states.len());
        for i in &self.initial_states {
            for j in &b.initial_states {
                initial_states.insert(i * b.size + j);
            }
        }

        let mut final_states = HashSet::with_capacity(self.final_states.len() * b.final_states.len());
        for i in &self.final_states {
            for j in &b.final_states {
                final_states.insert(i * b.size + j);
            }
        }

        Automaton {
            matrices,
            size,
            initial_states,
            final_states
        }
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        self.matrices.iter().map(|(label, matrix)| (label.to_string(), matrix.nvals())).collect()
    }

    fn reachable_pairs<C: Fn(&mut BaseTypeMatrix<bool>), F: Fn(&u64, &u64) -> bool>(&self, close: C, filter: F) -> Vec<Endpoints> {
        let mut closure = self.adjacency_matrix();
        close(&mut closure);
        Automaton::extract_pairs(&closure, filter)
    }

    pub fn reachable_pairs_all(&self) -> Vec<Endpoints> {
        self.reachable_pairs(
            Automaton::close_with_adjacency_matrix,
            |_, _| true
        )
    }

    pub fn reachable_pairs_from(&self, froms: &HashSet<u64>) -> Vec<Endpoints> {
        self.reachable_pairs(
            Automaton::close_with_adjacency_matrix,
            |from, _| froms.contains(from)
        )
    }

    pub fn reachable_pairs_from_to(&self, froms: &HashSet<u64>, tos: &HashSet<u64>) -> Vec<Endpoints> {
        self.reachable_pairs(
            Automaton::close_with_adjacency_matrix,
            |from, to| froms.contains(from) && tos.contains(to)
        )
    }

    pub fn hellings(&self, cfg: ContextFreeGrammar) -> Vec<Endpoints> {
        let mut r = HashSet::<(Endpoints, &String)>::new();
        let mut m = VecDeque::<(Endpoints, &String)>::new();

        if cfg.produces_epsilon {
            for v in 0..self.size {
                let p = ((v, v), &cfg.initial);
                r.insert(p.clone());
                m.push_back(p);
            }
        }

        for (label, matrix) in &self.matrices {
            if let Some(vars) = cfg.get_producers(&label) {
                let pairs = Automaton::extract_pairs(&matrix, |_, _| true);

                for var in vars {
                    for (from, to) in &pairs {
                        let p = ((*from, *to), var);
                        if r.insert(p.clone()) {
                            m.push_back(p);
                        }
                    }
                }
            }
        }

        while !m.is_empty() {
            let ((v, u), n_i) = m.pop_front().unwrap();
            let mut new = HashSet::<(Endpoints, &String)>::new();

            r.iter().filter(|((_, x), _)| *x == v)
                .for_each(|((v_, _), n_j)| {
                    if let Some(n_ks) = cfg.get_producers_by_pair(n_j, n_i) {
                        for n_k in n_ks {
                            let p = ((*v_, u), n_k);
                            if !r.contains(&p) {
                                new.insert(p);
                            }
                        }
                    }
                });

            r.iter().filter(|((x, _), _)| *x == u)
                .for_each(|((_, v_), n_j)| {
                    if let Some(n_ks) = cfg.get_producers_by_pair(n_i, n_j) {
                        for n_k in n_ks {
                            let p = ((v, *v_), n_k);
                            if !r.contains(&p) {
                                new.insert(p);
                            }
                        }
                    }
                });

            for n in new {
                r.insert(n.clone());
                m.push_back(n);
            }
        }

        r.into_iter().filter(|(_, s)| s == &&cfg.initial).map(|(p, _)| p).collect()
    }

    pub fn accepts<S: AsRef<str>>(&self, word: S) -> bool {
        let word = word.as_ref();
        self.initial_states.iter().any(|start| self.walk(*start, word))
    }

    fn walk(&self, position: u64, word: &str) -> bool {
        if word.len() == 0 && self.final_states.contains(&position) {
            return true;
        }
        self.matrices.iter()
            .filter(|(label, _)| word.starts_with(label.as_str()))
            .any(|(label, matrix)| {
                let (froms, tos, _) = matrix.extract_tuples();
                for i in 0..froms.len() {
                    if *froms.get(i).unwrap() == position {
                        if self.walk(*tos.get(i).unwrap(), &word[label.as_str().len()..word.len()]) {
                            return true;
                        }
                    }
                }
                false
            })
    }

    pub(crate) fn extract_pairs<F: Fn(&u64, &u64) -> bool>(m: &BaseTypeMatrix<bool>, filter: F) -> Vec<Endpoints> {
        let (froms, tos, _) = m.extract_tuples();
        froms.into_iter().zip(tos.into_iter()).filter(|(from, to)| filter(&from, &to)).map(|(from, to)| (from, to)).collect()
    }

    pub(crate) fn adjacency_matrix(&self) -> BaseTypeMatrix<bool> {
        let mut m = Matrix::<bool>::new(self.size, self.size);
        for (_, matrix) in &self.matrices {
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), matrix);
        }
        m
    }

    pub(crate) fn close_with_squaring(m: &mut BaseTypeMatrix<bool>) {
        let mut prev = 0;
        let mut square = Matrix::<bool>::new(m.nrows(), m.ncols());
        while prev != m.nvals() {
            prev = m.nvals();
            square.clear();
            square.accumulate_mxm(BinaryOp::<bool, bool, bool>::second(), Semiring::<bool>::lor_land(), m, m);
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &square);
        }
    }

    pub(crate) fn close_with_adjacency_matrix(m: &mut BaseTypeMatrix<bool>) {
        let adj = m.clone();
        let mut prev = 0;
        let mut production = Matrix::<bool>::new(m.nrows(), m.ncols());
        while prev != m.nvals() {
            prev = m.nvals();
            production.clear();
            production.accumulate_mxm(BinaryOp::<bool, bool, bool>::second(), Semiring::<bool>::lor_land(), &adj, m);
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &production);
        }
    }
}
