use flat_practice_lib::cfg::{ContextFreeGrammar};
use anyhow::Result;
use std::str::from_utf8;

mod util;

#[test]
fn epsilon() -> Result<()> {
    let cfg = ContextFreeGrammar::from_text(text!("grammars/epsilon")?)?;

    assert!(cfg.cyk(&[]));
    assert!(!cfg.cyk(&[&"a".to_string()]));
    Ok(())
}

#[test]
fn none() -> Result<()> {
    let cfg = ContextFreeGrammar::from_text(text!("grammars/none")?)?;

    assert!(!cfg.cyk(&[]));
    Ok(())
}

#[test]
fn test() -> Result<()> {
    let cfg = ContextFreeGrammar::from_text(text!("grammars/operations")?)?;

    let ref a = String::from("a");
    let ref b = String::from("b");
    let ref c = String::from("c");
    let ref n = String::from("n");

    assert!(!cfg.cyk(&[]));
    assert!(cfg.cyk(&[c, n, a, n, c, b, n]));
    assert!(cfg.cyk(&[n, a, n, b, n]));
    assert!(cfg.cyk(&[n, a, n, a, n, a, n]));
    assert!(cfg.cyk(&[n, a, c, n, b, n, c, a, n]));

    Ok(())
}
