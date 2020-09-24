
use std::env;
use flat_practice_lib::automaton::Automaton;
use anyhow::Result;
use flat_practice_lib::measure::write_csv;
use std::str::FromStr;

static HELP: &'static str = "Arguments: (stats *path to graph file* *path to request file*) | (measure *path*)";

fn main() -> Result<()> {
    let mut args = env::args().skip(1);

    let cmd = if let Some(cmd) = args.next() {
        cmd
    } else {
        panic!(HELP);
    };

    match cmd.as_str() {
        "stats" => {
            let graph_path = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP);
            };

            let regex_path = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP);
            };

            let graph = Automaton::read_graph(graph_path)?;
            let regex = Automaton::read_regex(regex_path)?;
            let intersection = graph.intersection(&regex);
            println!("{:?}", intersection.get_stats());
        },
        "measure" => {
            let path = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP);
            };
            let csv = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP);
            };
            let iterations = if let Some(arg) = args.next() {
                u64::from_str(arg.as_str())?
            } else {
                panic!(HELP);
            };
            write_csv(path, csv, iterations)?;
        }
        other => panic!("unknown command {}", other)
    }


    Ok(())
}
