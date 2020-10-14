use std::env;
use std::str::FromStr;

use anyhow::Result;

use flat_practice_lib::dfa::Dfa;
use flat_practice_lib::graph::Graph;
use flat_practice_lib::measure::write_csv;

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

            let graph = Graph::read_from(graph_path)?;
            let regex = Dfa::read_regex_from(regex_path)?;
            println!("{:?}", graph.kronecker(&regex.graph).get_stats());
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
