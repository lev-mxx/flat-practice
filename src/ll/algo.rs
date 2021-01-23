use anyhow::Result;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serializer, Deserializer};
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Node<T> {
    pub nonterminal: usize,
    pub children: Vec<Child<T>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Child<T> {
    Terminal(usize),
    Value(usize, T),
    Nonterminal(Node<T>),
    Epsilon(usize),
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

        let mut ast_path = Vec::<(Node<T>, usize)>::new();

        loop {
            match stack.pop().unwrap().destruct() {
                (Variant::Nonterminal, nonterminal) => {
                    let token = tokens.peek()?;
                    if let Some(&production) = self.table[nonterminal].get(&token) {
                        if production != EPSILON_RULE_CODE {
                            let node = Node {
                                nonterminal,
                                children: vec![]
                            };
                            ast_path.push((node, stack.len()));

                            let production = &self.productions[production];
                            production.iter().rev().for_each(|s| { stack.push(*s); })
                        } else {
                            ast_path.last_mut().unwrap().0.children.push(Child::Epsilon(nonterminal))
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
                        ast_path.last_mut().unwrap().0.children.push(child);
                    } else {
                        return Err(anyhow::Error::msg(format!("Invalid token {}: {}", tn, terminal)))
                    }
                }
            }

            let (_, mut node_stack_size) = ast_path.last().unwrap();
            while node_stack_size == stack.len() {
                let (node, _) = ast_path.pop().unwrap();
                if let Some((parent, _)) = ast_path.last_mut() {
                    parent.children.push(Child::Nonterminal(node))
                } else {
                    if tokens.peek()? == END_SYMBOL_CODE {
                        return Ok(node)
                    }
                    return Err(anyhow::Error::msg(format!("Not the end: {}", tn)))
                }
                node_stack_size = ast_path.last().unwrap().1;
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
mod tests {
    use super::{Cfg, Symbol, END_SYMBOL_CODE, Table, EPSILON_RULE_CODE, First};

    macro_rules! set {
        () => { ::std::collections::HashSet::new() };
        ( $( $x:expr ),+ ) => {
            {
                let mut set = ::std::collections::HashSet::new();
                $(set.insert($x);)*
                set
            }
        };
    }

    macro_rules! map {
        () => { ::std::collections::HashMap::new() };
        ( $($key:expr => $value:expr),* ) => {
            {
                let mut map = ::std::collections::HashMap::new();
                $(map.insert($key, $value);)+
                map
            }
        };
    }

    lazy_static! {
    static ref CFG_EPSILON: Cfg = Cfg {
        epsilon_producers: set![0],
        productions: vec![],
        nonterminals_count: 1,
    };

    static ref CFG_CHAR: Cfg = Cfg {
        epsilon_producers: set![],
        productions: vec![(0, vec![Symbol::terminal(0)])],
        nonterminals_count: 1,
    };

    static ref CFG_DYCK: Cfg = Cfg {
        epsilon_producers: set![0],
        nonterminals_count: 1,
        productions: vec![(0, vec![Symbol::terminal(0), Symbol::nonterminal(0), Symbol::terminal(1), Symbol::nonterminal(0)])]
    };

    static ref CFG_EXPR: Cfg = Cfg {
        nonterminals_count: 5,
        // nonterminals: 0 - expr, 1 - expr cont, 2 - factor, 3 - factor cont, 4 - primary
        // terminals: 0 - 'n', 1 - '+', 2 - '*', 3 - '(', 4 - ')'
        epsilon_producers: set![1, 3],
        productions: vec![
            (0, vec![Symbol::nonterminal(2), Symbol::nonterminal(1)]),
            (1, vec![Symbol::terminal(1), Symbol::nonterminal(0)]),
            (2, vec![Symbol::nonterminal(4), Symbol::nonterminal(3)]),
            (3, vec![Symbol::terminal(2), Symbol::nonterminal(2)]),
            (4, vec![Symbol::terminal(0)]),
            (4, vec![Symbol::terminal(3), Symbol::nonterminal(0), Symbol::terminal(4)]),
        ]
    };
    }

    #[test]
    fn epsilon_first() {
        let actual = CFG_EPSILON.firsts();
        let expected = vec![
            First {
                epsilon: true,
                others: set![],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn epsilon_follows() {
        let firsts = CFG_EPSILON.firsts();
        let actual = CFG_EPSILON.follows(&firsts);
        let expected = vec![set![END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn epsilon_table() {
        let actual = Table::build(CFG_EPSILON.clone()).unwrap().table;
        let expected = vec![map! { END_SYMBOL_CODE => EPSILON_RULE_CODE }];
        assert_eq!(expected, actual);
    }

    #[test]
    fn char_first() {
        let actual = CFG_CHAR.firsts();
        let expected = vec![
            First {
                epsilon: false,
                others: set![0],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn char_follows() {
        let firsts = CFG_CHAR.firsts();
        let actual = CFG_CHAR.follows(&firsts);
        let expected = vec![set![END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn char_table() {
        let actual = Table::build(CFG_CHAR.clone()).unwrap().table;
        let expected = vec![map! { 0 => 0 }];
        assert_eq!(expected, actual);
    }

    #[test]
    fn dyck_first() {
        let actual = CFG_DYCK.firsts();
        let expected = vec![
            First {
                epsilon: true,
                others: set![0],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn dyck_follows() {
        let firsts = CFG_DYCK.firsts();
        let actual = CFG_DYCK.follows(&firsts);
        let expected = vec![set![1, END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn dyck_table() {
        let actual = Table::build(CFG_DYCK.clone()).unwrap().table;
        let expected = vec![map! { 0 => 0, 1 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE }];
        assert_eq!(expected, actual);
    }

    #[test]
    fn expr_first() {
        let actual = CFG_EXPR.firsts();
        let expected = vec![
            First {
                epsilon: false,
                others: set![0, 3],
            },
            First {
                epsilon: true,
                others: set![1],
            },
            First {
                epsilon: false,
                others: set![0, 3],
            },
            First {
                epsilon: true,
                others: set![2],
            },
            First {
                epsilon: false,
                others: set![0, 3],
            },
        ];

        assert_eq!(expected, actual);
    }

    #[test]
    fn expr_follows() {
        let firsts = CFG_EXPR.firsts();
        let actual = CFG_EXPR.follows(&firsts);
        let expected = vec![
            set![4, END_SYMBOL_CODE],
            set![4, END_SYMBOL_CODE],
            set![1, 4, END_SYMBOL_CODE],
            set![1, 4, END_SYMBOL_CODE],
            set![1, 2, 4, END_SYMBOL_CODE],
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn expr_table() {
        let actual = Table::build(CFG_EXPR.clone()).unwrap().table;
        let expected = vec![
            map! { 0 => 0, 3 => 0 },
            map! { 1 => 1, 4 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE },
            map! { 0 => 2, 3 => 2 },
            map! { 1 => EPSILON_RULE_CODE, 2 => 3, 4 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE },
            map! { 0 => 4, 3 => 5 },
        ];
        assert_eq!(expected, actual);
    }
}
