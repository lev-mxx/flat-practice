
use std::collections::HashSet;
use flat_practice_lib::automaton::*;

fn assert_reachable(g: &Automaton, pairs: &[Endpoints]) {
    let actual: HashSet<Endpoints> = g.reachable_pairs_from_to(&g.initial_states, &g.final_states).iter().cloned().collect();

    let expected: HashSet<Endpoints> = pairs.iter().cloned().collect();

    assert_eq!(expected, actual);
}

#[test]
fn test_empty_reachability() {
    assert_reachable(&Automaton::build_graph(4, &[]), &[]);
}

#[test]
fn test_reachable() {
    assert_reachable(
        &Automaton::build_graph(4, &[
            (0, 0, "a".to_string()),
            (0, 2, "a".to_string()),
            (2, 3, "a".to_string()),
            (3, 1, "a".to_string()),
        ]),
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
}

#[test]
fn test_intersection_empty() {
    let empty = Automaton::build_graph(2, &[]);
    let not_empty = Automaton::build_graph(2, &[
        (0, 1, "a".to_string()),
        (1, 1, "a".to_string()),
    ]);

    let intersection = empty.intersection(&not_empty);
    assert_reachable(&intersection, &[]);
}

#[test]
fn test_regex() {
    let ab = Automaton::build_request("(a|b)+").unwrap();

    assert!(ab.accepts("aa"));
    assert!(ab.accepts("bb"));
    assert!(ab.accepts("abab"));
    assert!(!ab.accepts("cc"));
}

#[test]
fn test_intersection() {
    let ab = Automaton::build_request("(a|b)+").unwrap();
    let bc = Automaton::build_request("(c|b)+").unwrap();
    let bi = ab.intersection(&bc);

    assert!(bi.accepts("bb"));
    assert!(!bi.accepts("aa"));
    assert!(!bi.accepts("cc"));
}
