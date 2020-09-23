
use rustomaton::regex::Regex;
use rustomaton::dfa::ToDfa;

#[test]
fn automatons_test() {
    let alphabet = ['a', 'b', 'c'];
    let ab = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "(a+|b+)").unwrap().to_dfa();
    let bc = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "(c+|b+)").unwrap().to_dfa();
    let b = Regex::parse_with_alphabet(alphabet.iter().cloned().collect(), "bb*").unwrap().to_dfa();
    let bi = ab.intersect(bc);
    assert_eq!(bi, b);
}
