use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::from_utf8;

use anyhow::Result;
use pyo3::Python;
use pyo3::types::PyModule;
use graphblas::*;

use crate::automaton::{Automaton};
use crate::graph::{Graph, Ends, Edge, ExtractPairs};


#[derive(Debug)]
pub struct Rfa {
    pub(crate) dfa: Automaton,
    pub(crate) nonterminals: HashMap<(u64, u64), String>,
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

    pub fn from_text(text: &str) -> Result<Rfa> {
        let (initial, productions) = Python::with_gil(|py| -> Result<(String, Vec<(String, Vec<String>)>)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/read_cfg.py"))?, "a.py", "a")?;
            let py_res: (String, Vec<(String, Vec<String>)>) = module.call1("read_cfg", (text,))?.extract()?;
            Ok(py_res)
        })?;

        let mut edges = Vec::<Edge>::new();
        let mut initials = HashSet::<u64>::new();
        let mut finals = HashSet::<u64>::new();
        let mut with_epsilon = HashSet::<String>::new();

        let mut size: u64 = 0;

        let mut ends = HashMap::<String, (u64, u64)>::new();
        for (head, body) in productions {
            let (begin, end) = ends.entry(head.clone()).or_insert((size, size + 1));

            if begin == &size {
                initials.insert(size);
                finals.insert(size + 1);
                size += 2;
            }

            match body.len() {
                0 => {
                    if finals.insert(*begin) {
                        with_epsilon.insert(head.clone());
                    }
                },
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

        let dfa = Automaton {
            graph: Graph::build(edges.as_slice()),
            initials,
            finals,
        };

        let mut nonterminals = HashMap::<(u64, u64), String>::new();
        for nonterminal in &with_epsilon {
            let (begin, _) = ends.get(nonterminal).unwrap();
            nonterminals.insert((*begin, *begin), nonterminal.clone());
        }

        ends.into_iter().for_each(|(nt, ends)| { nonterminals.insert(ends, nt); });

        Ok(Rfa {
            dfa,
            nonterminals,
            with_epsilon,
            initial,
        })
    }
}

impl Graph {

    pub fn cfpq_tensor_product(&self, rfa: &Rfa) -> Vec<Ends> {
        println!("{:?}", self);
        println!("{:?}", rfa);
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
            println!("{:?}", intersection.reachable_pairs());
            for (from, to) in intersection.reachable_pairs() {
                let ref rfa_c = (from / self.size, to / self.size);
                let (rfa_from, rfa_to) = rfa_c;
                if  rfa.dfa.initials.contains(rfa_from) && rfa.dfa.finals.contains(rfa_to) {
                    let (from, to) = (from % self.size, to % self.size);
                    let nt = rfa.nonterminals.get(rfa_c).unwrap();
                    println!("{} {} {}", from, to, nt);
                    let matrix = m2.get_mut(nt.clone());
                    if let None = matrix.get(from, to) {
                        matrix.insert(from, to, true);
                        changing = true;
                    }
                }
            }
        }

        if let Some(matrix) = m2.matrices.get(&rfa.initial) {
            matrix.extract_pairs()
        } else {
            Vec::new()
        }
    }
}
