
use rustgraphblas::*;
use rustomaton::regex::Regex;
use rustomaton::dfa::ToDfa;
use rustomaton::automaton::Automata;

#[test]
fn link_test() {
    SparseMatrix::<bool>::empty((3, 3));
}

#[test]
fn automatons_test() {
    let ab = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "(a+|b+)").unwrap().to_dfa();
    let bc = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "(c+|b+)").unwrap().to_dfa();
    let b = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "bb*").unwrap().to_dfa();
    let bi = ab.intersect(bc);
    assert_eq!(bi, b);
}

#[test]
fn matrices_test() {
    let mut a = SparseMatrix::<u32>::empty((2, 2));
    a.load(&[1, 2, 3, 5], &[0, 0, 1, 1], &[0, 1, 0, 1]);
    let mut b = SparseMatrix::<u32>::empty((2, 2));
    b.load(&[5, 3, 2, 1], &[0, 0, 1, 1], &[0, 1, 0, 1]);

    let monoid = SparseMonoid::new(BinaryOp::<u32, u32, u32>::plus(), 0);
    let semiring = Semiring::new(&monoid, BinaryOp::<u32, u32, u32>::times());

    let c = a.mxm(Option::<&SparseMatrix<bool>>::None, Option::None, &b, semiring, &Descriptor::default());
    
    assert_eq!(c.get((0, 0)), Some(9));
    assert_eq!(c.get((0, 1)), Some(5));
    assert_eq!(c.get((1, 0)), Some(25));
    assert_eq!(c.get((1, 1)), Some(14));
}
