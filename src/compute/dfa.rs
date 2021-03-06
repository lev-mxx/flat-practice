use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::str::from_utf8;

use anyhow::{Error, Result};
use pyo3::Python;
use pyo3::types::PyModule;

use super::graph::{Edge, Ends, ExtractPairs, Graph};

#[derive(Clone)]
pub struct Dfa {
    pub graph: Graph,
    pub initials: HashSet<usize>,
    pub finals: HashSet<usize>,
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
        let line = reader
            .lines()
            .skip(2)
            .next()
            .ok_or_else(|| Error::msg("invalid format"))??;
        Dfa::from_regex(line.as_str())
    }

    pub fn from_regex(regex: &str) -> Result<Dfa> {
        let (initial, finals, edges) =
            Python::with_gil(|py| -> Result<(usize, Vec<usize>, Vec<Edge>)> {
                let module = PyModule::from_code(
                    py,
                    from_utf8(include_bytes!("py/regex_to_edges.py"))?,
                    "a.compute.py",
                    "a",
                )?;
                let py_res: (usize, Vec<usize>, Vec<(usize, usize, String)>) =
                    module.call1("regex_to_edges", (regex,))?.extract()?;
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
            finals,
        }
    }

    pub fn accepts(&self, word: &[&str]) -> bool {
        let word = word.as_ref();
        self.initials.iter().any(|start| self.walk(*start, word))
    }

    fn walk(&self, position: usize, word: &[&str]) -> bool {
        if word.len() == 0 && self.finals.contains(&position) {
            return true;
        }

        if let Some(matrix) = self.graph.get(word[0]) {
            let pairs =
                matrix.extract_pairs_filter(|p| if p.0 == position { Some(p) } else { None });

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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use anyhow::Result;

    use crate::compute::dfa::*;
    use crate::compute::graph::{Ends, Graph};

    fn assert_reachable(a: &Graph, b: &Dfa, pairs: &[Ends]) {
        let res = a.rpq(b);
        let actual: HashSet<&Ends> = res.iter().collect();
        let expected: HashSet<&Ends> = pairs.iter().collect();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_reachable() -> Result<()> {
        let a = &Graph::build(&[
            (0, 0, "a".to_string()),
            (0, 2, "a".to_string()),
            (2, 3, "a".to_string()),
            (3, 1, "a".to_string()),
        ]);
        let b = Dfa::from_regex("a*")?;

        assert_reachable(
            &a,
            &b,
            &[(0, 0), (0, 1), (0, 2), (0, 3), (2, 3), (2, 1), (3, 1)],
        );
        Ok(())
    }

    #[test]
    fn test_intersection_empty() -> Result<()> {
        let a = Graph::build(&[(0, 0, "a".to_string())]);
        let b = Dfa {
            graph: Graph::build(&[(1, 1, "b".to_string())]),
            initials: [0, 1].iter().cloned().collect(),
            finals: [0, 1].iter().cloned().collect(),
        };

        assert_reachable(&a, &b, &[]);
        Ok(())
    }

    #[test]
    fn test_regex() -> Result<()> {
        let ab = Dfa::from_regex("(a|b)*")?;

        assert!(ab.accepts(&["a", "a"]));
        assert!(ab.accepts(&["b", "b"]));
        assert!(ab.accepts(&["a", "b", "a", "b"]));
        assert!(!ab.accepts(&["c", "c"]));

        Ok(())
    }

    #[test]
    fn test_intersection() -> Result<()> {
        let ab = Dfa::from_regex("(a|b)*")?;
        let bc = Dfa::from_regex("(c|b)*")?;
        let bi = ab.intersection(&bc);

        assert!(bi.accepts(&["b", "b"]));
        assert!(!bi.accepts(&["a", "a"]));
        assert!(!ab.accepts(&["c", "c"]));

        Ok(())
    }
}
