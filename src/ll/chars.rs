use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use serde::Serialize;
use anyhow::Result;

use crate::ll::algo::{Production, Cfg, Tokens, END_SYMBOL_CODE};
use std::str::Chars;

#[derive(Debug, Serialize, Deserialize)]
pub struct CharInfo {
    pub nonterminals: Vec<String>,
}

pub struct ParserCtx {
    pub nonterminals: HashMap<String, usize>,
}

lalrpop_mod!(parser, "/ll/parser.rs");

pub fn get_rules(ctx: &mut ParserCtx, text: &str) -> Result<Vec<(usize, Production)>> {
    let parser = parser::rulesParser::new();
    Ok(parser.parse(ctx, text).unwrap())
}

pub fn parse_cfg(text: &str) -> Result<(CharInfo, Cfg)> {
    let mut ctx = ParserCtx {
        nonterminals: Default::default()
    };
    let rules = get_rules(&mut ctx, text)?;
    //println!("{:?}", rules);

    let mut epsilon_producers = HashSet::new();
    let productions = rules.into_iter().filter(|(nt, p)| if p.is_empty() {
        epsilon_producers.insert(*nt);
        false
    } else {
        true
    }).collect();

    let mut nonterminals: Vec<_> = ctx.nonterminals.into_iter().collect();
    nonterminals.sort_by_key(|(_, c)| *c);

    let nonterminals_count = nonterminals.len();
    Ok((
        CharInfo {
            nonterminals: nonterminals.into_iter().map(|(x, _)| x).collect()
        },
        Cfg {
            epsilon_producers,
            productions,
            nonterminals_count,
        }
    ))
}

pub struct CharTokenizer<'a> {
    pub chars: Chars<'a>,
    pub current: Option<usize>,
}

impl<'a> Tokens<()> for CharTokenizer<'a> {
    fn pop(&mut self) -> Result<Option<()>> {
        let _ = self.peek()?;
        self.current = None;
        Ok(None)
    }

    fn peek(&mut self) -> Result<usize> {
        match self.current {
            None => match self.chars.next() {
                None => {
                    self.current = Some(END_SYMBOL_CODE);
                    Ok(END_SYMBOL_CODE)
                }
                Some(ch) => {
                    let code = ch as usize;
                    self.current = Some(code);
                    Ok(code)
                }
            },
            Some(code) => Ok(code),
        }
    }
}
