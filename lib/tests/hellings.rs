
use flat_practice_lib::automaton::{Endpoints, Automaton};
use flat_practice_lib::cfg::ContextFreeGrammar;
use anyhow::Result;
use std::collections::HashSet;
use std::str::from_utf8;

mod util;

fn test(graph: &str, grammar: &str, expected: &[Endpoints]) -> Result<()> {
    let grammar = ContextFreeGrammar::from_text(grammar)?;
    let graph = Automaton::from_text(graph)?;

    let res = graph.hellings(grammar);
    let actual: HashSet<&Endpoints> = res.iter().collect();
    let expected: HashSet<&Endpoints> = expected.iter().collect();

    assert_eq!(expected, actual);

    Ok(())
}

macro_rules! test {
    ($graph: expr, $grammar: expr, $expected: expr) => {
        paste::paste! {
            #[test]
            fn [<$graph _ $grammar>]() -> Result<()> {
                test(text!(concat!("graphs/", $graph))?, text!(concat!("grammars/", $grammar))?, $expected)
            }
        }
    };
}

test!("graph1", "epsilon", &[(0, 0), (1, 1)]);
test!("graph1", "none", &[]);
test!("graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
test!("graph1", "grammar2", &[(0, 1)]);
test!("graph1", "grammar3", &[(0, 0), (0, 1)]);

test!("graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
test!("graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
test!("graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);
