#![allow(dead_code)]
#![feature(fn_traits)]

#[macro_use]
extern crate lalrpop_util;

use std::env;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use anyhow::Result;

use crate::compute::dfa::Dfa;
use crate::compute::graph::Graph;
use crate::measure::write_csv;

mod compute;
mod syntax;
mod measure;

static HELP: &'static str = concat!("Arguments: (stats *path to graph file* *path to request file*)\n",
    "\t| (measure *path*)\n",
    "\t| (check *path*)\n",
    "\t| (dot *path*)\n",
);

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let mut arg = || {
        if let Some(arg) = args.next() { arg } else { panic!(HELP) }
    };

    let cmd = arg();

    match cmd.as_str() {
        "stats" => {
            let graph_path = arg();
            let regex_path = arg();

            let graph = Graph::read_from(graph_path)?;
            let regex = Dfa::read_regex_from(regex_path)?;
            println!("{:?}", graph.kronecker(&regex.graph).get_stats());
        }
        "measure" => {
            let path = arg();
            let csv = arg();
            let iterations = u64::from_str(arg().as_str())?;
            write_csv(path, csv, iterations)?;
        }
        "check" => {
            let path = arg();
            let string = match path.as_str() {
                "-" => read_stdin(),
                _ => {
                    let mut file = File::open(path)?;
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    content
                }
            };

            if syntax::check(string.as_str())? {
                println!("valid")
            } else {
                println!("invalid")
            }
        }
        "dot" => {
            let path = arg();
            let string = match path.as_str() {
                "-" => read_stdin(),
                _ => {
                    let mut file = File::open(path)?;
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    content
                }
            };
            let ast = syntax::build_ast(string.as_str())?;
            let dot = syntax::to_dot(&ast);

            println!("{}", dot);
        }
        other => panic!("unknown command {}", other)
    }

    Ok(())
}

fn read_stdin() -> String {
    let mut str = String::new();
    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                match input.as_str() {
                    "end\n" => break,
                    _ => str.push_str(input.as_str()),
                }
            }
            Err(error) => panic!("error: {}", error),
        }
    }
    str
}
