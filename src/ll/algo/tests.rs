use super::{Cfg, Symbol, END_SYMBOL_CODE, Table, EPSILON_RULE_CODE, First};
use crate::ll::algo::{Tokens, Node, Child};

macro_rules! set {
        () => { ::std::collections::HashSet::new() };
        ( $( $x:expr ),+ ) => {
            {
                let mut set = ::std::collections::HashSet::new();
                $(set.insert($x);)+
                set
            }
        };
    }

macro_rules! map {
        () => { ::std::collections::HashMap::new() };
        ( $($key:expr => $value:expr),+ ) => {
            {
                let mut map = ::std::collections::HashMap::new();
                $(map.insert($key, $value);)+
                map
            }
        };
    }

macro_rules! assert_fail {
        ( $x:expr ) => {
            match $x {
                Ok(_) => assert!(false),
                Err(_) => assert!(true),
            }
        };
    }

macro_rules! tokens {
        () => { TokenVec { vec: Vec::new() } };
        ( $( $x:expr ),+ ) => {
            {
                let mut vec = Vec::new();
                $(vec.push($x);)+
                vec.reverse();
                TokenVec { vec }
            }
        };
    }

mod epsilon {
    use super::*;

    lazy_static! {
        static ref CFG: Cfg = Cfg {
            epsilon_producers: set![0],
            productions: vec![],
            nonterminals_count: 1,
        };
        }

    #[test]
    fn first() {
        let actual = CFG.firsts();
        let expected = vec![
            First {
                epsilon: true,
                others: set![],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn follow() {
        let firsts = CFG.firsts();
        let actual = CFG.follows(&firsts);
        let expected = vec![set![END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn table() {
        let actual = Table::build(CFG.clone()).unwrap().table;
        let expected = vec![map! { END_SYMBOL_CODE => EPSILON_RULE_CODE }];
        assert_eq!(expected, actual);
    }

    mod ast {
        use super::*;

        #[test]
        fn epsilon() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![]
            };
            assert_eq!(expected, actual);
            Ok(())
        }

        #[test]
        fn any() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0];
            assert_fail!(table.build_ast(&mut tokens));
        }
    }
}

mod char {
    use super::*;

    lazy_static! {
        static ref CFG: Cfg = Cfg {
            epsilon_producers: set![],
            productions: vec![(0, vec![Symbol::terminal(0)])],
            nonterminals_count: 1,
        };
        }

    #[test]
    fn first() {
        let actual = CFG.firsts();
        let expected = vec![
            First {
                epsilon: false,
                others: set![0],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn follow() {
        let firsts = CFG.firsts();
        let actual = CFG.follows(&firsts);
        let expected = vec![set![END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn table() {
        let actual = Table::build(CFG.clone()).unwrap().table;
        let expected = vec![map! { 0 => 0 }];
        assert_eq!(expected, actual);
    }

    mod ast {
        use super::*;

        #[test]
        fn epsilon() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![];
            assert_fail!(table.build_ast(&mut tokens));
        }

        #[test]
        fn a() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![Child::Terminal(0)]
            };
            assert_eq!(expected, actual);
            Ok(())
        }

        #[test]
        fn aa() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0, 0];
            assert_fail!(table.build_ast(&mut tokens));
        }

        #[test]
        fn b() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![1];
            assert_fail!(table.build_ast(&mut tokens));
        }
    }
}

mod dyck {
    use super::*;

    lazy_static! {
        static ref CFG: Cfg = Cfg {
            epsilon_producers: set![0],
            nonterminals_count: 1,
            productions: vec![(0, vec![Symbol::terminal(0), Symbol::nonterminal(0), Symbol::terminal(1), Symbol::nonterminal(0)])]
        };
        }

    #[test]
    fn first() {
        let actual = CFG.firsts();
        let expected = vec![
            First {
                epsilon: true,
                others: set![0],
            }
        ];
        assert_eq!(expected, actual);
    }

    #[test]
    fn follow() {
        let firsts = CFG.firsts();
        let actual = CFG.follows(&firsts);
        let expected = vec![set![1, END_SYMBOL_CODE]];
        assert_eq!(expected, actual);
    }

    #[test]
    fn table() {
        let actual = Table::build(CFG.clone()).unwrap().table;
        let expected = vec![map! { 0 => 0, 1 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE }];
        assert_eq!(expected, actual);
    }

    mod ast {
        use super::*;

        #[test]
        fn epsilon() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![]
            };
            assert_eq!(expected, actual);
            Ok(())
        }

        #[test]
        fn abab() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0, 1, 0, 1];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![
                    Child::Terminal(0),
                    Child::Nonterminal(Node {
                        nonterminal: 0,
                        children: vec![]
                    }),
                    Child::Terminal(1),
                    Child::Nonterminal(Node {
                        nonterminal: 0,
                        children: vec![
                            Child::Terminal(0),
                            Child::Nonterminal(Node {
                                nonterminal: 0,
                                children: vec![]
                            }),
                            Child::Terminal(1),
                            Child::Nonterminal(Node {
                                nonterminal: 0,
                                children: vec![]
                            }),
                        ]
                    }),
                ]
            };
            assert_eq!(expected, actual);
            Ok(())
        }

        #[test]
        fn a() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0];
            assert_fail!(table.build_ast(&mut tokens));
        }
    }
}

mod expr {
    use super::*;

    lazy_static! {
        static ref CFG: Cfg = Cfg {
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
    fn first() {
        let actual = CFG.firsts();
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
    fn follow() {
        let firsts = CFG.firsts();
        let actual = CFG.follows(&firsts);
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
    fn table() {
        let actual = Table::build(CFG.clone()).unwrap().table;
        let expected = vec![
            map! { 0 => 0, 3 => 0 },
            map! { 1 => 1, 4 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE },
            map! { 0 => 2, 3 => 2 },
            map! { 1 => EPSILON_RULE_CODE, 2 => 3, 4 => EPSILON_RULE_CODE, END_SYMBOL_CODE => EPSILON_RULE_CODE },
            map! { 0 => 4, 3 => 5 },
        ];
        assert_eq!(expected, actual);
    }

    mod ast {
        use super::*;

        #[test]
        fn epsilon() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![];
            assert_fail!(table.build_ast(&mut tokens));
        }

        #[test]
        fn n() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![
                    Child::Nonterminal(Node {
                        nonterminal: 2,
                        children: vec![
                            Child::Nonterminal(Node {
                                nonterminal: 4,
                                children: vec![Child::Terminal(0)]
                            }),
                            Child::Nonterminal(Node {
                                nonterminal: 3,
                                children: vec![]
                            }),
                        ]
                    }),
                    Child::Nonterminal(Node {
                        nonterminal: 1,
                        children: vec![]
                    })
                ]
            };
            assert_eq!(expected, actual);
            Ok(())
        }

        #[test]
        fn nn() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0, 0];
            assert_fail!(table.build_ast(&mut tokens));
        }

        #[test]
        fn np() {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0, 1];
            assert_fail!(table.build_ast(&mut tokens));
        }

        #[test]
        fn npn() -> Result<()> {
            let table = Table::build(CFG.clone()).unwrap();
            let mut tokens = tokens![0, 1, 0];
            let actual = table.build_ast(&mut tokens)?;
            let expected = Node {
                nonterminal: 0,
                children: vec![
                    Child::Nonterminal(Node {
                        nonterminal: 2,
                        children: vec![
                            Child::Nonterminal(Node {
                                nonterminal: 4,
                                children: vec![Child::Terminal(0)]
                            }),
                            Child::Nonterminal(Node {
                                nonterminal: 3,
                                children: vec![]
                            }),
                        ]
                    }),
                    Child::Nonterminal(Node {
                        nonterminal: 1,
                        children: vec![Child::Terminal(1), Child::Nonterminal(Node {
                            nonterminal: 0,
                            children: vec![
                                Child::Nonterminal(Node {
                                    nonterminal: 2,
                                    children: vec![
                                        Child::Nonterminal(Node {
                                            nonterminal: 4,
                                            children: vec![Child::Terminal(0)]
                                        }),
                                        Child::Nonterminal(Node {
                                            nonterminal: 3,
                                            children: vec![]
                                        }),
                                    ]
                                }),
                                Child::Nonterminal(Node {
                                    nonterminal: 1,
                                    children: vec![]
                                })
                            ]
                        })]
                    })
                ]
            };
            assert_eq!(expected, actual);
            Ok(())
        }
    }
}

struct TokenVec {
    vec: Vec<usize>,
}

use anyhow::Result;

impl Tokens<()> for TokenVec {
    fn pop(&mut self) -> Result<Option<()>> {
        match self.vec.pop() {
            Some(_) => Ok(None),
            None => Err(anyhow::Error::msg("")),
        }
    }

    fn peek(&mut self) -> Result<usize> {
        match self.vec.last() {
            Some(&x) => Ok(x),
            None => Ok(END_SYMBOL_CODE),
        }
    }
}
