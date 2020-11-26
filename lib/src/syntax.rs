use anyhow::Result;

use crate::cfg::ContextFreeGrammar;
use std::str::from_utf8;

fn input_map(c: char) -> char {
    if c.is_whitespace() {
        '!'
    } else if c == '.' {
        '@'
    } else if c == '|' {
        '^'
    } else {
        c
    }
}

pub fn check(text: &str) -> Result<bool> {
    let cfg = ContextFreeGrammar::_from_text(from_utf8(include_bytes!("syntax.cfg"))?)?;
    let chars: Vec<String> = text.chars().map(input_map).map(String::from).collect();
    let refs: Vec<&String> = chars.iter().collect();
    Ok(cfg.cyk(refs.as_slice()))
}
