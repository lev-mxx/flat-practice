
use flat_practice_lib::automaton::{Endpoints, Automaton};
use flat_practice_lib::cfg::ContextFreeGrammar;
use anyhow::Result;
use std::collections::HashSet;
use std::str::from_utf8;

fn test(grammar: &str, graph: &str, expected: &[Endpoints]) -> Result<()> {
    let grammar = ContextFreeGrammar::from_text(grammar)?;
    let graph = Automaton::from_text(graph)?;

    let res = graph.hellings(grammar);
    let actual: HashSet<&Endpoints> = res.iter().collect();
    let expected: HashSet<&Endpoints> = expected.iter().collect();

    assert_eq!(expected, actual);

    Ok(())
}

#[test]
fn test_1_1() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar1"))?, from_utf8(include_bytes!("data04/graph1"))?, &[(0, 0), (1, 1), (0, 1)])
}

#[test]
fn test_2_1() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar2"))?, from_utf8(include_bytes!("data04/graph1"))?, &[(0, 1)])
}

#[test]
fn test_3_1() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar3"))?, from_utf8(include_bytes!("data04/graph1"))?, &[(0, 0), (0, 1)])
}

#[test]
fn test_1_2() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar1"))?, from_utf8(include_bytes!("data04/graph2"))?, &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)])
}

#[test]
fn test_2_2() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar2"))?, from_utf8(include_bytes!("data04/graph2"))?, &[(0, 0), (1, 3), (0, 4)])
}

#[test]
fn test_3_2() -> Result<()> {
    test(from_utf8(include_bytes!("data04/grammar3"))?, from_utf8(include_bytes!("data04/graph2"))?, &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)])
}
