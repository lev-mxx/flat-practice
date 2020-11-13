
use flat_practice_lib::cfg::{ContextFreeGrammar, ContextFreeResult};
use anyhow::Result;
use std::collections::HashSet;
use std::str::from_utf8;
use flat_practice_lib::graph::{Ends, Graph};
use flat_practice_lib::rfa::Rfa;

mod util;

fn test<F: Fn(&str, &Graph) -> Result<Vec<Ends>>>(graph: &str, grammar: &str, fun: F, expected: &[Ends]) -> Result<()> {
    let graph = Graph::from_text(graph)?;
    let res = fun(grammar, &graph)?;
    let actual: HashSet<&Ends> = res.iter().collect();
    let expected: HashSet<&Ends> = expected.iter().collect();

    assert_eq!(expected, actual);

    Ok(())
}

macro_rules! test {
    ($fun: ident, $graph: expr, $grammar: expr, $expected: expr) => {
        paste::paste! {
            #[test]
            fn [<$fun _ $graph _ $grammar>]() -> Result<()> {
                test(text!(concat!("graphs/", $graph))?, text!(concat!("grammars/", $grammar))?, $fun, $expected)
            }
        }
    };
}

fn hellings(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
    let grammar = ContextFreeGrammar::from_text(text)?;
    Ok(graph.cfpq_hellings(&grammar).reachable_edges("S"))
}

fn matrices(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
    let grammar = ContextFreeGrammar::from_text(text)?;
    Ok(graph.cfpq_matrix_product(&grammar).reachable_edges("S"))
}

fn tensors(text: &str, graph: &Graph) -> Result<Vec<Ends>> {
    let rfa = Rfa::from_text(text)?;
    Ok(graph.cfpq_tensor_product(&rfa).reachable_edges("S"))
}

test!(hellings, "graph1", "epsilon", &[(0, 0), (1, 1)]);
test!(hellings, "graph1", "none", &[]);
test!(hellings, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
test!(hellings, "graph1", "grammar2", &[(0, 1)]);
test!(hellings, "graph1", "grammar3", &[(0, 0), (0, 1)]);
test!(hellings, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
test!(hellings, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
test!(hellings, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);

test!(matrices, "graph1", "epsilon", &[(0, 0), (1, 1)]);
test!(matrices, "graph1", "none", &[]);
test!(matrices, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
test!(matrices, "graph1", "grammar2", &[(0, 1)]);
test!(matrices, "graph1", "grammar3", &[(0, 0), (0, 1)]);
test!(matrices, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
test!(matrices, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
test!(matrices, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);

test!(tensors, "graph1", "epsilon", &[(0, 0), (1, 1)]);
test!(tensors, "graph1", "none", &[]);
test!(tensors, "graph1", "grammar1", &[(0, 0), (1, 1), (0, 1)]);
test!(tensors, "graph1", "grammar2", &[(0, 1)]);
test!(tensors, "graph1", "grammar3", &[(0, 0), (0, 1)]);
test!(tensors, "graph2", "grammar1", &[(0, 0), (1, 1), (2, 2), (3, 3), (4, 4), (0, 4), (1, 3)]);
test!(tensors, "graph2", "grammar2", &[(0, 0), (1, 3), (0, 4)]);
test!(tensors, "graph2", "grammar3", &[(0, 1), (1, 2), (0, 4), (1, 3), (1, 0)]);
