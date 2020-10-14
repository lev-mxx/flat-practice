use anyhow::{Result, Error};
use std::path::Path;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::str::FromStr;
use std::collections::{HashMap, HashSet};
use graphblas::*;
use std::borrow::Borrow;
use std::hash::Hash;

pub type Ends = (u64, u64);
pub type Edge = (u64, u64, String);
pub(crate) type BooleanMatrix = BaseTypeMatrix<bool>;

#[derive(Debug, Clone)]
pub struct Graph {
    pub(crate) matrices: HashMap<String, BooleanMatrix>,
    pub size: u64,
}

impl Graph {
    pub(crate) fn with_size(size: u64) -> Graph {
        Graph {
            matrices: HashMap::new(),
            size
        }
    }

    pub fn read_from<P: AsRef<Path>>(path: P) -> Result<Graph> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Graph::from_lines(reader.lines().map(|f| Ok(f?)))
    }

    pub fn from_text(text: &str) -> Result<Graph> {
        Graph::from_lines(text.to_string().split("\n").map(str::to_string).map(Ok))
    }

    pub fn from_lines<I: Iterator<Item=Result<String>>>(lines: I) -> Result<Graph> {
        let edges = lines
            .filter(|line| if let Ok(line) = line { !line.is_empty() } else { true })
            .map(|line| match line {
            Ok(line) => {
                let split: Vec<&str> = line.split(" ").collect();
                if split.len() != 3 {
                    Err(Error::msg("invalid format"))?
                } else {
                    let from = u64::from_str(split[0])?;
                    let to = u64::from_str(split[2])?;

                    let label = split[1].to_string();
                    Ok((from, to, label))
                }
            },
            Err(e) => Err(e)
        });

        Graph::from_edges(edges)
    }

    pub fn from_edges<I: Iterator<Item=Result<Edge>>>(edges: I) -> Result<Graph> {
        let mut size: u64 = 0;
        let mut label_paths = HashMap::<String, (Vec<u64>, Vec<u64>)>::new();

        for edge in edges {
            let (from, to, label) = edge?;
            if from >= size { size = from + 1 }
            if to >= size { size = to + 1 }

            let (from_vertices, to_vertices) = label_paths.entry(label.to_string()).or_insert_with(|| (Vec::new(), Vec::new()));
            from_vertices.push(from);
            to_vertices.push(to);
        }

        let mut graph = Graph::with_size(size);
        for (label, (froms, tos)) in label_paths {
            let matrix = graph.get_mut(label);
            matrix.build(&froms, &tos, vec!(true; froms.len()), BinaryOp::<bool, bool, bool>::first());
        }

        Ok(graph)
    }

    pub(crate) fn get<S: ?Sized + Hash + Eq>(&self, label: &S) -> Option<&BooleanMatrix>
        where String: Borrow<S>
    {
        self.matrices.get(label)
    }

    pub(crate) fn get_mut(&mut self, label: String) -> &mut BooleanMatrix {
        let size = self.size;
        self.matrices.entry(label).or_insert_with(|| Matrix::<bool>::new(size, size))
    }

    pub fn build(edges: &[Edge]) -> Graph {
        Graph::from_edges(edges.iter().cloned().map(Ok)).unwrap()
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        self.matrices.iter().map(|(label, matrix)| (label.to_string(), matrix.nvals())).collect()
    }

    pub fn kronecker(&self, b: &Graph) -> Graph {
        let mut graph = Graph::with_size(self.size * b.size);
        self.matrices.iter()
            .filter_map(|(label, m)| b.matrices.get(label).map(|om| (label, m, om)))
            .for_each(|(label, m, om)| {
                let matrix = graph.get_mut(label.clone());
                matrix.assign_kronecker(Semiring::<bool>::lor_land(), &m, &om);
            });
        graph
    }

    pub fn reachable_pairs(&self) -> Vec<Ends> {
        let mut closure = self.adjacency_matrix();
        Graph::close_with_squaring(&mut closure);
        closure.extract_pairs()
    }

    pub fn reachable_pairs_filter<F: Fn(Ends) -> Option<Ends>>(&self, filter: F) -> HashSet<Ends> {
        let mut closure = self.adjacency_matrix();
        Graph::close_with_squaring(&mut closure);
        closure.extract_pairs_filter(filter)
    }

    pub(crate) fn adjacency_matrix(&self) -> BooleanMatrix {
        let mut m = Matrix::<bool>::new(self.size, self.size);
        for (_, matrix) in &self.matrices {
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), matrix);
        }
        m
    }

    pub(crate) fn close_with_squaring(m: &mut BooleanMatrix) {
        let mut prev = 0;
        let mut square = Matrix::<bool>::new(m.nrows(), m.ncols());
        while prev != m.nvals() {
            prev = m.nvals();
            square.clear();
            square.accumulate_mxm(BinaryOp::<bool, bool, bool>::lor(), Semiring::<bool>::lor_land(), m, m);
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &square);
        }
    }

    pub(crate) fn close_with_adjacency_matrix(m: &mut BooleanMatrix) {
        let adj = m.clone();
        let mut prev = 0;
        let mut production = Matrix::<bool>::new(m.nrows(), m.ncols());
        while prev != m.nvals() {
            prev = m.nvals();
            production.clear();
            production.accumulate_mxm(BinaryOp::<bool, bool, bool>::lor(), Semiring::<bool>::lor_land(), &adj, m);
            m.accumulate_apply(BinaryOp::<bool, bool, bool>::lor(), UnaryOp::<bool, bool>::identity(), &production);
        }
    }
}

pub(crate) trait ExtractPairs {
    fn extract_pairs(&self) -> Vec<Ends>;

    fn extract_pairs_filter<F: Fn(Ends) -> Option<Ends>>(&self, filter: F) -> HashSet<Ends>;
}

impl ExtractPairs for BooleanMatrix {
    fn extract_pairs(&self) -> Vec<Ends> {
        let (froms, tos, _) = self.extract_tuples();
        froms.into_iter().zip(tos.into_iter()).collect()
    }

    fn extract_pairs_filter<F: Fn(Ends) -> Option<Ends>>(&self, filter: F) -> HashSet<Ends> {
        let (froms, tos, _) = self.extract_tuples();
        froms.into_iter().zip(tos.into_iter()).filter_map(filter).collect()
    }
}
