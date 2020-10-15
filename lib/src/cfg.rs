use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{BufRead, Write};
use std::path::Path;
use std::str::from_utf8;

use anyhow::Result;
use pyo3::Python;
use pyo3::types::PyModule;

use graphblas::*;

use crate::graph::{BooleanMatrix, Ends, ExtractPairs, Graph};

#[derive(Debug)]
pub struct ContextFreeGrammar {
    pub(crate) initial: String,
    pub(crate) produces_epsilon: bool,
    pub(crate) unit_from_variable: HashMap<String, HashSet<String>>,
    pub(crate) pair_from_variable: HashMap<String, HashMap<String, HashSet<String>>>,
}

impl ContextFreeGrammar {

    pub(crate) fn get_producers_by_pair(&self, p1: &String, p2: &String) -> Option<&HashSet<String>>  {
        self.pair_from_variable.get(p1).and_then(|x| x.get(p2))
    }

    pub(crate) fn get_producers(&self, p: &String) -> Option<&HashSet<String>> {
        self.unit_from_variable.get(p)
    }

    pub fn read<P: AsRef<Path>>(path: P) -> Result<ContextFreeGrammar> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        let vec = Vec::<u8>::new();
        let mut writer = std::io::BufWriter::new(vec);

        for line in reader.lines() {
            let line = line?;
            let (head, body) = if let Some(space) = line.find(" ") {
                line.split_at(space)
            } else {
                (line.as_str(), "")
            };
            writer.write(head.as_bytes())?;
            writer.write(" -> ".as_bytes())?;
            writer.write(body.as_bytes())?;
            writer.write("\n".as_bytes())?;
        }
        writer.flush()?;
        ContextFreeGrammar::from_text(std::str::from_utf8(writer.get_ref().as_slice())?)
    }

    pub fn from_text(text: &str) -> Result<ContextFreeGrammar> {
        let (initial, productions, produces_epsilon) = Python::with_gil(|py| -> Result<(String, Vec<(String, Vec<String>)>, bool)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/parse_cfg.py"))?, "a.py", "a")?;
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
        return m[0][word.len() - 1].contains(&self.initial);
    }
}

impl Graph {
    pub fn cfpq_hellings(&self, cfg: &ContextFreeGrammar) -> Vec<Ends> {
        let mut r = HashSet::<(Ends, &String)>::new();
        let mut m = VecDeque::<(Ends, &String)>::new();

        if cfg.produces_epsilon {
            for v in 0..self.size {
                let p = ((v, v), &cfg.initial);
                r.insert(p);
                m.push_back(p);
            }
        }

        for (label, matrix) in &self.matrices {
            if let Some(vars) = cfg.get_producers(&label) {
                let pairs = matrix.extract_pairs();

                for var in vars {
                    for (from, to) in &pairs {
                        let p = ((*from, *to), var);
                        if r.insert(p) {
                            m.push_back(p);
                        }
                    }
                }
            }
        }

        while !m.is_empty() {
            let ((v, u), n_i) = m.pop_front().unwrap();
            let mut new = HashSet::<(Ends, &String)>::new();

            for ((v_, u_), n_j) in &r {
                if *u_ == v {
                    if let Some(n_ks) = cfg.get_producers_by_pair(n_j, n_i) {
                        for n_k in n_ks {
                            let p = ((*v_, u), n_k);
                            if !r.contains(&p) {
                                new.insert(p);
                            }
                        }
                    }
                }

                if *v_ == u {
                    if let Some(n_ks) = cfg.get_producers_by_pair(n_i, n_j) {
                        for n_k in n_ks {
                            let p = ((v, *u_), n_k);
                            if !r.contains(&p) {
                                new.insert(p);
                            }
                        }
                    }
                }
            }

            for n in new {
                r.insert(n);
                m.push_back(n);
            }
        }

        r.into_iter().filter(|(_, s)| s == &&cfg.initial).map(|(p, _)| p).collect()
    }
}
