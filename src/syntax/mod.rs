use std::str::from_utf8;

use anyhow::Result;

use crate::compute::cfg::ContextFreeGrammar;
use crate::syntax::ast::Script;

pub mod ast;
pub mod dot;
pub use dot::to_dot;

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
    test!("open", true);
    test!("let", true);
    test!("get", true);
    test!("cond", true);
    test!("precedence", true);
    test!("complex", true);

    test!("inv_empty", false);
    test!("inv_open", false);
    test!("inv_let", false);
    test!("inv_cond", false);
    test!("inv_precedence", false);
}

#[cfg(test)]
mod test_ast {
    use std::str::from_utf8;

    use anyhow::Result;

    use ast::*;

    use super::*;

    macro_rules! test {
        ($script: expr, $expected: expr) => {
            paste::paste! {
                #[test]
                fn [<test _ $script>]() -> Result<()> {
                    assert_eq!($expected, build_ast(from_utf8(include_bytes!(concat!("../../test_data/scripts/", $script)))?)?);
                    Ok(())
                }
            }
        };
    }

    test!("empty", Sequence(Vec::new()));
    test!("open", Sequence(vec!(Connect("db".to_string()))));
    test!("let", Sequence(vec!(Define("a".to_string(), Term("a".to_string())))));
    test!("precedence", Sequence(vec!(Define("a".to_string(),
        Alt(
            Box::new(Seq(vec!(Term("b".to_string()), Star(Box::new(Term("c".to_string())))))),
            Box::new(Seq(vec!(Var("d".to_string()), Maybe(Box::new(Var("e".to_string()))))))
        )
    ))));
    test!("cond", Sequence(vec!(
        Get(
            List(Filter(Box::new(Edges), Cond("c".to_string(), "a".to_string(), "b".to_string(), And(
                Box::new(IsStart("a".to_string())),
                Box::new(IsStart("b".to_string())),
            )))),
            Graph { name: "g".to_string() }
        )
    )));
    test!("get", Sequence(vec!(
        Get(
            Count(Edges),
            Graph { name: "g".to_string() }
        )
    )));
}
