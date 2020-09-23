
use std::env;
use flat_practice_lib::automaton::Automaton;
use std::io::{Result, ErrorKind, Error};

fn main() -> Result<()> {
    let mut args = env::args();
    args.next();

    let graph_path = args.next().ok_or_else(|| Error::from(ErrorKind::InvalidInput))?;
    let regex_path = args.next().ok_or_else(|| Error::from(ErrorKind::InvalidInput))?;

    let graph = Automaton::read_graph(graph_path)?;
    let regex = Automaton::read_regex(regex_path)?;

    match args.next().unwrap().as_str() {
        "stats" => {
            let intersection = graph.intersection(&regex);
            intersection.print_stats();
        },
        _ => {}
    }

    Ok(())
}
