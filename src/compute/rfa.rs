use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::from_utf8;

use anyhow::Result;
use pyo3::Python;
use pyo3::types::PyModule;

use graphblas::*;

use super::dfa::Dfa;
use super::graph::{Edge, Ends, ExtractPairs, Graph, BooleanMatrix};
use super::cfg::{ContextFreeGrammar, ContextFreeResult};

#[derive(Debug)]
pub struct Rfa {
    pub(crate) dfa: Dfa,
    pub(crate) nonterminals: HashSet<String>,
    pub(crate) ends2nonterminal: HashMap<(u64, u64), String>,
    pub(crate) with_epsilon: HashSet<String>,
    pub(crate) initial: String
}

impl Rfa {
    pub fn read_from<P: AsRef<Path>>(path: P) -> Result<Rfa> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut cfg = String::new();
        reader.read_to_string(&mut cfg)?;
        Rfa::from_text(cfg.as_str())
    }

    pub fn from_text_without_regex(text: &str) -> Result<Rfa> {
        let converted = ContextFreeGrammar::convert(text.lines().map(Ok))?;

        let (initial, nonterminals, productions, produces_epsilon) = Python::with_gil(|py| -> Result<(String, Vec<String>, Vec<(String, Vec<String>)>, bool)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/read_cfg_in_cnf.py"))?, "a.compute.py", "a")?;
            Ok(module.call1("parse_cfg", (converted.as_str(),))?.extract()?)
        })?;

        let mut edges = Vec::<Edge>::new();
        let mut initials = HashSet::<u64>::new();
        let mut finals = HashSet::<u64>::new();
        let mut with_epsilon = HashSet::<String>::new();
        let mut ends = HashMap::<String, (u64, u64)>::new();

        let mut size: u64 = 0;

        if produces_epsilon {
            with_epsilon.insert(initial.clone());
            initials.insert(0);
            finals.insert(0);
            size += 1;
        }

        for (head, body) in productions {
            let (begin, end) = ends.entry(head.clone()).or_insert((size, size + 1));

            if begin == &size {
                initials.insert(size);
                finals.insert(size + 1);
                size += 2;
            }

            match body.len() {
                0 => unreachable!(),
                1 => { edges.push((*begin, *end, body.into_iter().next().unwrap())); },
                n => {
                    let n = n as u64;
                    let mut body = body.into_iter();
                    edges.push((*begin, size, body.next().unwrap()));
                    size += 1;

                    for _ in 0..n - 2 {
                        edges.push((size - 1, size, body.next().unwrap()));
                        size += 1;
                    }

                    edges.push((size - 1, *end, body.next().unwrap()));
                }
            }
        }

        let dfa = Dfa {
            graph: Graph::build(edges.as_slice()),
            initials,
            finals,
        };

        let mut ends2nonterminal = HashMap::<(u64, u64), String>::new();
        for nonterminal in &with_epsilon {
            let (begin, _) = ends.get(nonterminal).unwrap();
            ends2nonterminal.insert((*begin, *begin), nonterminal.clone());
        }

        ends.into_iter().for_each(|(nt, ends)| { ends2nonterminal.insert(ends, nt); });

        Ok(Rfa {
            dfa,
            nonterminals: nonterminals.into_iter().collect(),
            ends2nonterminal,
            with_epsilon,
            initial,
        })
    }

    pub fn from_text(text: &str) -> Result<Rfa> {
        let mut edges = Vec::<Edge>::new();
        let mut initials = HashSet::<u64>::new();
        let mut finals = HashSet::<u64>::new();
        let mut with_epsilon = HashSet::<String>::new();
        let mut nonterminals = HashSet::<String>::new();
        let mut ends2nonterminal = HashMap::<(u64, u64), String>::new();

        let mut size: u64 = 0;

        for line in text.lines() {
            let (head, body) = if let Some(space) = line.find(" ") {
                line.split_at(space)
            } else {
                continue
            };

            let nonterminal: String = head.chars()
                .skip_while(|c| c.is_whitespace())
                .take_while(|c| !c.is_whitespace())
                .collect();

            let (line_initial, line_finals, line_edges) = Python::with_gil(|py| -> Result<(u64, Vec<u64>, Vec<Edge>)> {
                let module = PyModule::from_code(py, from_utf8(include_bytes!("py/regex_to_edges.py"))?, "a.compute.py", "a")?;
                let py_res: (u64, Vec<u64>, Vec<Edge>) = module.call1("regex_to_edges", (body,))?.extract()?;
                Ok(py_res)
            })?;

            let mut max = line_initial;

            initials.insert(size + line_initial);
            for line_final in line_finals {
                ends2nonterminal.insert((size + line_initial, size + line_final), nonterminal.clone());

                finals.insert(size + line_final);
                if line_final > max { max = line_final; }
            }

            if line_edges.len() > 0 {
                for (from, to, label) in line_edges {
                    edges.push((size + from, size + to, label));
                    if from > max { max = from; }
                    if to > max { max = to; }
                }
            } else {
                with_epsilon.insert(nonterminal.clone());
            }

            nonterminals.insert(nonterminal);
            size += max + 1;
        }

        let dfa = Dfa {
            graph: Graph::build(edges.as_slice()),
            initials,
            finals,
        };

        Ok(Rfa {
            dfa,
            nonterminals,
            ends2nonterminal,
            with_epsilon,
            initial: "S".to_string(),
        })
    }
}

impl Graph {

    pub fn cfpq_tensor_product<'a>(&self, rfa: &'a Rfa) -> ResultTensors<'a> {
        let mut m2 = self.clone();

        for nonterminal in &rfa.with_epsilon {
            let matrix = m2.get_mut(nonterminal.clone());
            for i in 0..self.size {
                matrix.insert(i, i, true);
            }
        }

        let mut changing = true;
        while changing {
            changing = false;
            let intersection = rfa.dfa.graph.kronecker(&m2);
            for (from, to) in intersection.reachable_pairs() {
                let ref rfa_c = (from / self.size, to / self.size);
                let (rfa_from, rfa_to) = rfa_c;
                if  rfa.dfa.initials.contains(rfa_from) && rfa.dfa.finals.contains(rfa_to) {
                    let (from, to) = (from % self.size, to % self.size);
                    let nt = rfa.ends2nonterminal.get(rfa_c).unwrap();
                    let matrix = m2.get_mut(nt.clone());
                    if let None = matrix.get(from, to) {
                        matrix.insert(from, to, true);
                        changing = true;
                    }
                }
            }
        }

        ResultTensors {
            map: m2.matrices,
            nonterminals: &rfa.nonterminals,
        }
    }
}

pub struct ResultTensors<'a> {
    pub(crate) map: HashMap<String, BooleanMatrix>,
    pub(crate) nonterminals: &'a HashSet<String>,
}

impl<'a> ContextFreeResult for ResultTensors<'a> {
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

