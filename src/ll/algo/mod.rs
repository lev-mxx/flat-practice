use anyhow::Result;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serializer, Deserializer};
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Node<T> {
    pub nonterminal: usize,
    pub children: Vec<Child<T>>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Child<T> {
    Terminal(usize),
    Value(usize, T),
    Nonterminal(Node<T>),
}

pub trait Tokens<T> {
    fn pop(&mut self) -> Result<Option<T>>;

    fn peek(&mut self) -> Result<usize>;
}

#[derive(Serialize, Deserialize)]
pub struct Table {
    pub productions: Vec<Production>,
    pub table: Vec<HashMap<usize, usize>>,
}

impl Table {
    fn get(&self, nt: usize, t: usize) -> Option<(usize, &Production)> {
        self.table.get(nt).unwrap().get(&t)
            .map(|code| (*code, &self.productions[*code]))
    }

    pub fn build_ast<T, I: Tokens<T>>(&self, tokens: &mut I) -> Result<Node<T>> {
        let mut stack = Vec::<Symbol>::new();
        stack.push(Symbol::nonterminal(0));
        let mut tn = 0;

        struct AstPathElement<T> {
            node: Node<T>,
            stack_size: usize,
        }

        let mut ast_path = Vec::<AstPathElement<T>>::new();

        loop {
            match stack.pop().unwrap().destruct() {
                (Variant::Nonterminal, nonterminal) => {
                    let token = tokens.peek()?;
                    if let Some(&production) = self.table[nonterminal].get(&token) {
                        ast_path.push(AstPathElement {
                            node: Node {
                                nonterminal,
                                children: vec![]
                            },
                            stack_size: stack.len(),
                        });

                        if production != EPSILON_RULE_CODE {
                            let production = &self.productions[production];
                            production.iter().rev().for_each(|s| { stack.push(*s); })
                        }
                    } else {
                        return Err(anyhow::Error::msg(format!("No rule {}: {} {}", tn, nonterminal, token)))
                    }
                }
                (Variant::Terminal, terminal) => {
                    let token = tokens.peek()?;
                    if token == terminal {
                        tn += 1;
                        let child = if let Some(value) = tokens.pop()? {
                            Child::Value(token, value)
                        } else {
                            Child::Terminal(token)
                        };
                        ast_path.last_mut().unwrap().node.children.push(child);
                    } else {
                        return Err(anyhow::Error::msg(format!("Invalid token {}: {}", tn, terminal)))
                    }
                }
            }

            let mut stack_size = ast_path.last().unwrap().stack_size;
            while stack_size == stack.len() {
                let AstPathElement { node, .. } = ast_path.pop().unwrap();
                if let Some(AstPathElement { node: parent, .. }) = ast_path.last_mut() {
                    parent.children.push(Child::Nonterminal(node))
                } else {
                    if tokens.peek()? == END_SYMBOL_CODE {
                        return Ok(node)
                    }
                    return Err(anyhow::Error::msg(format!("Not the end: {}", tn)))
                }
                stack_size = ast_path.last().unwrap().stack_size;
            }
        }
    }

    pub fn build(cfg: Cfg) -> Result<Self> {
        let firsts = cfg.firsts();
        let follows = cfg.follows(&firsts);

        let mut table = vec![HashMap::new(); cfg.nonterminals_count];

        for &nonterminal in &cfg.epsilon_producers {
            let t2p = &mut table[nonterminal];
            for terminal in &follows[nonterminal] {
                if let Some(_) = t2p.insert(*terminal, EPSILON_RULE_CODE) {
                    return Err(anyhow::Error::msg("ambiguity"));
                }
            }
        }

        for (code, (nonterminal, production)) in cfg.productions.iter().enumerate() {
            let line = &mut table[*nonterminal];

            let mut epsilon = false;
            let mut first = Default::default();
            Cfg::first(&firsts, &production, &mut epsilon, &mut first);

            for terminal in first {
                if let Some(_) = line.insert(terminal, code) {
                    return Err(anyhow::Error::msg("ambiguity"));
                }
            }

            if epsilon {
                for terminal in &follows[*nonterminal] {
                    if let Some(_) = line.insert(*terminal, code) {
                        return Err(anyhow::Error::msg("ambiguity"));
                    }
                }
            }
        }

        Ok(Table {
            productions: cfg.productions.into_iter().map(|(_, p)| p).collect(),
            table,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cfg {
    pub epsilon_producers: HashSet<usize>,
    pub nonterminals_count: usize,
    pub productions: Vec<(usize, Production)>,
}

impl Cfg {
    fn extend(a: &mut HashSet<usize>, b: &HashSet<usize>) {
        for t in b {
            a.insert(*t);
        }
    }

    fn first(firsts: &Firsts, seq: &[Symbol], epsilon: &mut bool, others: &mut HashSet<usize>) {
        if seq.is_empty() {
            *epsilon = true;
        } else {
            match seq[0].destruct() {
                (Variant::Terminal, code) => { others.insert(code); },
                (Variant::Nonterminal, a) => {
                    let first_a = &firsts[a];
                    Cfg::extend(others, &first_a.others);
                    if first_a.epsilon {
                        Self::first(firsts, &seq[1..], epsilon, others);
                    }
                }
            }
        }
    }

    fn firsts(&self) -> Firsts {
        let mut firsts: Vec<First> = vec![Default::default(); self.nonterminals_count];
        let mut first = Default::default();

        for &nonterminal in &self.epsilon_producers {
            firsts[nonterminal].epsilon = true;
        }

        let mut changed = true;
        while changed {
            changed = false;
            for (nonterminal, production) in &self.productions {
                first = std::mem::replace(&mut firsts[*nonterminal], first);

                let old_eps = first.epsilon;
                let old_size = first.others.len();
                Self::first(&firsts, &production, &mut first.epsilon, &mut first.others);

                if first.epsilon != old_eps || first.others.len() != old_size {
                    changed = true;
                }

                first = std::mem::replace(&mut firsts[*nonterminal], first);
            }
        }

        firsts
    }

    fn follows(&self, firsts: &Firsts) -> Follows {
        let mut follows = vec![HashSet::new(); self.nonterminals_count];
        let mut follow_a = HashSet::new();

        follows[0].insert(END_SYMBOL_CODE);
        let mut changed = true;
        while changed {
            changed = false;
            for (a, production) in &self.productions {
                follow_a = std::mem::replace(&mut follows[*a], follow_a);

                for (i, sym) in production.iter().enumerate() {
                    if let (Variant::Nonterminal, x) = sym.destruct() {
                        let old_size;
                        let mut epsilon = false;
                        {
                            let follow_x = if x == *a {
                                &mut follow_a
                            } else {
                                &mut follows[x]
                            };
                            old_size = follow_x.len();
                            Self::first(firsts, &production[(i + 1)..], &mut epsilon, follow_x);
                        }

                        if x == *a {
                            if old_size != follow_a.len() {
                                changed = true;
                            }
                        } else {
                            let follow_x = &mut follows[x];
                            if epsilon {
                                Self::extend(follow_x, &follow_a);
                            }
                            if old_size != follow_x.len() {
                                changed = true;
                            }
                        }
                    }
                }

                follow_a = std::mem::replace(&mut follows[*a], follow_a);
            }
        }

        follows
    }
}

type Firsts = Vec<First>;
type Follows = Vec<HashSet<usize>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct First {
    epsilon: bool,
    others: HashSet<usize>,
}

pub type Production = Vec<Symbol>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Symbol {
    code: usize
}

pub enum Variant {
    Terminal,
    Nonterminal,
}

impl Symbol {
    #[inline(always)]
    pub(crate) fn destruct(self: Symbol) -> (Variant, usize) {
        let code = self.code & !MASK;
        if self.code & MASK != 0 {
            (Variant::Terminal, code)
        } else {
            (Variant::Nonterminal, code)
        }
    }

    #[inline(always)]
    pub fn terminal(code: usize) -> Symbol {
        Symbol {
            code: code | MASK
        }
    }

    #[inline(always)]
    pub fn nonterminal(code: usize) -> Symbol {
        Symbol {
            code
        }
    }
}

impl Serialize for Symbol {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        Serialize::serialize(&self.code, serializer)
    }
}

impl<'de> Deserialize<'de> for Symbol {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        Ok(Symbol { code: Deserialize::deserialize(deserializer)? })
    }
}

const USIZE_BITS: usize = std::mem::size_of::<usize>() * 8;
const MASK: usize = 1 << (USIZE_BITS - 1);

const EPSILON_RULE_CODE: usize = usize::MAX;
pub const END_SYMBOL_CODE: usize = usize::MAX & !MASK;

#[cfg(test)]
mod tests;
