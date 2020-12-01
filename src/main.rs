#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use anyhow::Result;
use crate::compute::graph::Graph;
use crate::compute::dfa::Dfa;
use crate::measure::write_csv;

mod compute;
mod syntax;
mod measure;

static HELP: &'static str = "Arguments: (stats *path to graph file* *path to request file*) | (measure *path*) | (check *path*) | (dot *path*)";

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
        }
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
        "check" => {
            let path = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP);
            };
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            if syntax::check(content.as_str())? {
                println!("valid")
            } else {
                println!("invalid")
            }
        }
        "dot" => {
            let path = if let Some(arg) = args.next() {
                arg
            } else {
                panic!(HELP)
            };
            let mut file = File::open(path)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let ast = syntax::build_ast(content.as_str())?;
            let dot = syntax::to_dot(&ast);
            println!("{}", dot);
        }
        other => panic!("unknown command {}", other)
    }

    Ok(())
}
