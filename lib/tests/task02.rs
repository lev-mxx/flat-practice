use std::collections::HashSet;
use flat_practice_lib::automaton::*;
use flat_practice_lib::graph::{Ends, Graph};
use anyhow::Result;

fn assert_reachable(a: &Graph, b: &Automaton, pairs: &[Ends]) {
    let res = a.rpq(b);
    let actual: HashSet<&Ends> = res.iter().collect();
    let expected: HashSet<&Ends> = pairs.iter().collect();

    assert_eq!(expected, actual);
}

#[test]
fn test_reachable() -> Result<()> {
    let a = &Graph::build(&[
        (0, 0, "a".to_string()),
        (0, 2, "a".to_string()),
        (2, 3, "a".to_string()),
        (3, 1, "a".to_string()),
    ]);
    let b = Automaton::from_regex("a*")?;

    assert_reachable(&a, &b,
        &[
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (2, 3),
            (2, 1),
            (3, 1),
        ]
    );
    Ok(())
}

#[test]
fn test_intersection_empty() -> Result<()> {
    let a = Graph::build(&[(0, 0, "a".to_string())]);
    let b = Automaton {
        graph: Graph::build(&[(1, 1, "b".to_string())]),
        initials: [0, 1].iter().cloned().collect(),
        finals: [0, 1].iter().cloned().collect()
    };

    assert_reachable(&a, &b,&[]);
    Ok(())
}

#[test]
fn test_regex() -> Result<()> {
    let ab = Automaton::from_regex("(a|b)*")?;

    assert!(ab.accepts(&["a", "a"]));
    assert!(ab.accepts(&["b", "b"]));
    assert!(ab.accepts(&["a", "b", "a", "b"]));
    assert!(!ab.accepts(&["c", "c"]));

    Ok(())
}

#[test]
fn test_intersection() -> Result<()> {
    let ab = Automaton::from_regex("(a|b)*")?;
    let bc = Automaton::from_regex("(c|b)*")?;
    let bi = ab.intersection(&bc);

    assert!(bi.accepts(&["b", "b"]));
    assert!(!bi.accepts(&["a", "a"]));
    assert!(!ab.accepts(&["c", "c"]));

    Ok(())
}
