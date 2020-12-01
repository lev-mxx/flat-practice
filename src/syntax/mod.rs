use std::str::from_utf8;

use anyhow::Result;

use crate::compute::cfg::ContextFreeGrammar;
use crate::syntax::ast::Script;

pub mod ast;
pub mod dot;

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

lalrpop_mod!(parser, "/syntax/parser.rs");

pub fn build_ast(text: &str) -> Result<Script> {
    let parser = parser::scriptParser::new();
    Ok(parser.parse(text).unwrap())
}

pub fn to_dot(s: &Script) -> String { dot::to_dot(s) }

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use anyhow::Result;

    use super::*;

    macro_rules! test {
    ($script: expr, $expected: expr) => {
        paste::paste! {
            #[test]
            fn [<test _ $script>]() -> Result<()> {
                assert_eq!($expected, check(from_utf8(include_bytes!(concat!("../../test_data/scripts/", $script)))?)?);
                Ok(())
            }
        }
    };
}

    test!("empty", true);
    test!("word", false);
    test!("open", true);
    test!("let", true);
    test!("get", true);
    test!("cond", true);
    test!("complex", true);

}
