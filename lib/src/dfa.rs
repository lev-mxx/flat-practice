use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::from_utf8;

use anyhow::{Error, Result};
use pyo3::Python;
use pyo3::types::PyModule;

use crate::graph::{Edge, Ends, ExtractPairs, Graph};

#[derive(Debug, Clone)]
pub struct Dfa {
    pub graph: Graph,
    pub initials: HashSet<u64>,
    pub finals: HashSet<u64>,
}

impl Dfa {
    pub fn read_regex_from<P: AsRef<Path>>(path: P) -> Result<Dfa> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut regex = String::new();
        reader.read_to_string(&mut regex)?;
        Dfa::from_regex(regex.as_str())
    }

    pub fn read_query_from<P: AsRef<Path>>(path: P) -> Result<Dfa> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let line = reader.lines().skip(2).next().ok_or_else(|| Error::msg("invalid format"))??;
        Dfa::from_regex(line.as_str())
    }

    pub fn from_regex(regex: &str) -> Result<Dfa> {
        let (initial, finals, edges) = Python::with_gil(|py| -> Result<(u64, Vec<u64>, Vec<Edge>)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/regex_to_edges.py"))?, "a.py", "a")?;
            let py_res: (u64, Vec<u64>, Vec<(u64, u64, String)>) = module.call1("regex_to_edges", (regex,))?.extract()?;
            Ok(py_res)
        })?;

        Ok(Dfa {
            graph: Graph::build(edges.as_slice()),
            initials: [initial].iter().cloned().collect(),
            finals: finals.into_iter().collect(),
        })
    }

    pub fn intersection(&self, b: &Dfa) -> Dfa {
        let graph = self.graph.kronecker(&b.graph);
        let mut initials = HashSet::with_capacity(self.initials.len() * b.initials.len());

        for i in &self.initials {
            for j in &b.initials {
                initials.insert(i * b.graph.size + j);
            }
        }

        let mut finals = HashSet::with_capacity(self.finals.len() * b.finals.len());
        for i in &self.finals {
            for j in &b.finals {
                finals.insert(i * b.graph.size + j);
            }
        }

        Dfa {
            graph,
            initials,
            finals
        }
    }

    pub fn accepts(&self, word: &[&str]) -> bool {
        let word = word.as_ref();
        self.initials.iter().any(|start| self.walk(*start, word))
    }

    fn walk(&self, position: u64, word: &[&str]) -> bool {
        if word.len() == 0 && self.finals.contains(&position) {
            return true;
        }

        if let Some(matrix) = self.graph.get(word[0]) {
            let pairs = matrix.extract_pairs_filter(|p| if p.0 == position {
                Some(p)
            } else {
                None
            });

            pairs.into_iter().any(|(_, to)| self.walk(to, &word[1..]))
        } else {
            false
        }
    }
}

impl Graph {

    pub fn rpq(&self, request: &Dfa) -> HashSet<Ends> {
        let g = self.kronecker(&request.graph);
        let size = request.graph.size;
        g.reachable_pairs_filter(|(from, to)| {
            let ref request_from = from % size;
            let ref request_to = to % size;
            if request.initials.contains(request_from) && request.finals.contains(request_to) {
                Some((from / size, to / size))
            } else {
                None
            }
        })
    }
}
