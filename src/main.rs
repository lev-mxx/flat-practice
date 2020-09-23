
use std::env;
use flat_practice_lib::automaton::Automaton;
use std::error::Error;

static HELP: &'static str = "Arguments: (path to graph file) (path to request file) stats";

fn main() -> Result<(), std::io::Error> {
    let mut args = env::args();
    args.next();

    let graph_path = if let Some(graph_path) = args.next() {
        graph_path
    } else {
        panic!(HELP);
    };

    let regex_path = if let Some(regex_path) = args.next() {
        regex_path
    } else {
        panic!(HELP);
    };

    let cmd = if let Some(cmd) = args.next() {
        cmd
    } else {
        panic!(HELP);
    };

    match cmd.as_str() {
        "stats" => {
            let graph = Automaton::read_graph(graph_path)?;
            let regex = Automaton::read_regex(regex_path)?;
            let intersection = graph.intersection(&regex);
            intersection.print_stats();
        },
        other => panic!("unknown command {}", other)
    }

    Ok(())
}
