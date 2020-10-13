use std::path::Path;
use anyhow::{Result};
use pyo3::types::PyModule;
use std::str::from_utf8;
use pyo3::Python;
use std::io::{BufRead, Write};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ContextFreeGrammar {
    pub(crate) initial: String,
    pub(crate) produces_epsilon: bool,
    terminal_from_variable: HashMap<String, HashSet<String>>,
    pair_from_variable: HashMap<String, HashMap<String, HashSet<String>>>,
}

impl ContextFreeGrammar {

    pub(crate) fn get_producers_by_pair(&self, p1: &String, p2: &String) -> Option<&HashSet<String>>  {
        self.pair_from_variable.get(p1).and_then(|x| x.get(p2))
    }

    pub(crate) fn get_producers(&self, p: &String) -> Option<&HashSet<String>> {
        self.terminal_from_variable.get(p)
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
        let (initial, productions, generate_epsilon) = Python::with_gil(|py| -> Result<(String, Vec<(String, Vec<String>)>, bool)> {
            let module = PyModule::from_code(py, from_utf8(include_bytes!("py/parse_cfg.py"))?, "a.py", "a")?;
            Ok(module.call1("parse_cfg", (text,))?.extract()?)
        })?;

        let mut terminal_from_variable = HashMap::<String, HashSet<String>>::new();
        let mut variables_from_variable = HashMap::<String, HashMap<String, HashSet<String>>>::new();

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
                let set = variables_from_variable
                    .entry(first)
                    .or_insert_with(|| HashMap::new())
                    .entry(second)
                    .or_insert_with(|| HashSet::new());
                set.insert(head);
            } else {
                let set = terminal_from_variable
                    .entry(first)
                    .or_insert_with(|| HashSet::new());
                set.insert(head);
            }
        }

        Ok(ContextFreeGrammar {
            initial,
            produces_epsilon: generate_epsilon,
            terminal_from_variable,
            pair_from_variable: variables_from_variable,
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
