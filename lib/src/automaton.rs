use std::fs::File;
use std::io::{BufReader, BufRead, Error, ErrorKind};
use std::str::FromStr;

use rustomaton::regex::*;
use rustomaton::dfa::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use anyhow::Result;

use graphblas::*;
use std::hash::Hash;
use std::fmt::{Debug, Display};

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
        let mut edges = Vec::<Edge>::new();
        let mut max: u64 = 0;

        for line in reader.lines() {
            let line = line?;
            let split: Vec<&str> = line.split(" ").collect();
            if split.len() == 0 {
                continue
            }
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

    const TOKENS: &'static [char] = &['(', ')', '+', '*', '?', '.', '|'];

    pub fn read_query<P: AsRef<Path>>(path: P) -> Result<Automaton> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let line = reader.lines().skip(2).next().ok_or_else(|| Error::from(ErrorKind::InvalidInput))??;

        let mut regex = Vec::<RegexPart<String>>::new();
        let mut chars = line.chars();
        let mut next = Some(' ');

        while next.is_some() && next.unwrap() != '>' {
            next = chars.next();
        }
        next = chars.next();

        while next.is_some() {
            let c = next.unwrap();

            if c.is_whitespace() {
                next = chars.next();
            } else if Automaton::TOKENS.iter().any(|x| *x == c) {
                regex.push(RegexPart::Token(c));
                next = chars.next();
            } else {
                let mut symbol = Vec::<char>::new();
                while next.is_some() && next.unwrap().is_alphanumeric() {
                    symbol.push(next.unwrap());
                    next = chars.next();
                }
                regex.push(RegexPart::Symbol(symbol.iter().collect()));
            }
        }

        let regex = Regex::parse(regex.as_slice()).map_err(|e| anyhow::Error::msg(e))?;
        let dfa = regex.to_dfa().minimize();
        Ok(Automaton::from_dfa(&dfa))
    }

    pub fn build_graph(size: u64, edges: &[Edge]) -> Automaton {
        let matrices = Automaton::from_edges(size, edges.iter().cloned());
        let initial_states: HashSet<u64> = (0..size).enumerate().map(|(_, b)| b).collect();
        let final_states: HashSet<u64> = (0..size).enumerate().map(|(_, b)| b).collect();

        Automaton {
            matrices,
            size,
            initial_states,
            final_states,
        }
    }

    pub fn from_regex(regex: &str) -> Result<Automaton> {
        let regex = Regex::from_str(regex).map_err(|str| Error::new(ErrorKind::Other, str))?;
        let dfa = regex.to_dfa().minimize();
        Ok(Automaton::from_dfa(&dfa))
    }

    fn from_dfa<A: ToString + Hash + Ord + Clone + Debug + Display>(dfa: &DFA<A>) -> Automaton {
        let mut initial_states = HashSet::<u64>::new();
        initial_states.insert(dfa.initial as u64);
        let final_states: HashSet<u64> = dfa.finals.iter().map(|x| *x as u64).collect();
        let size = dfa.transitions.len() as u64;
        let matrices = Automaton::from_edges(size, dfa.transitions.iter().enumerate()
            .flat_map(|(from, map)|
                map.iter().map(move |(c, to)| (from as u64, *to as u64, c.to_string()))));

        Automaton {
            matrices,
            size,
            initial_states,
            final_states
        }
    }

    fn from_edges<I : Iterator<Item = Edge>>(size: u64, edges_it: I) -> HashMap<String, BaseTypeMatrix<bool>> {
        let mut label_paths = HashMap::<String, (Vec<u64>, Vec<u64>)>::new();

        edges_it.for_each(|a| {
            let (from, to, label) = a;

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

        matrices
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
        froms.iter().zip(tos.iter()).filter(|(from, to)| filter(from, to)).map(|(from, to)| (*from, *to)).collect()
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
