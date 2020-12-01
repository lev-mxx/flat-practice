use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::time::Instant;

use anyhow::Result;

use graphblas::BaseTypeMatrix;

use crate::compute::dfa::*;
use crate::compute::graph::{ExtractPairs, Graph};

static LIST: &[(&str, fn(&mut BaseTypeMatrix<bool>))] = &[
    ("square", Graph::close_with_squaring),
    ("adj", Graph::close_with_adjacency_matrix),
];

pub fn write_csv(path: String, csv_path: String, iterations: u64) -> Result<()> {
    let csv_file = std::fs::File::create(csv_path)?;
    let mut csv = BufWriter::new(csv_file);

    for graph_dir in std::fs::read_dir(path)? {
        let graph_dir = graph_dir?;
        let graph_name = graph_dir.file_name().to_str().unwrap().to_string();
        let graph = Graph::read_from(graph_dir.path().join(format!("{}.txt", graph_name)))?;
        for class_dir in std::fs::read_dir(graph_dir.path().join("queries"))? {
            let class_dir = class_dir?;
            let class_name = class_dir.file_name().to_str().unwrap().to_string();

            for query_file in std::fs::read_dir(class_dir.path())? {
                let query_file = query_file?;
                let query_name = query_file.file_name().to_str().unwrap().to_string();
                let query = Dfa::read_query_from(query_file.path())?;

                for _ in 0..iterations {
                    let res = measure(&graph, &query);

                    let res = format!("{},{}/{},{},{},{},{},{},{},{}",
                                      graph_name, class_name, query_name,
                                      res.0,
                                      res.1["square"].0, res.1["square"].1, res.1["square"].2,
                                      res.1["adj"].0, res.1["adj"].1, res.1["adj"].2);
                    println!("{}", res);
                    csv.write(res.as_bytes())?;
                    csv.write("\n".as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

fn measure(graph: &Graph, request: &Dfa) -> (u128, HashMap<String, (u128, u128, usize)>) {
    let mut map = HashMap::<String, (u128, u128, usize)>::new();

    let time = Instant::now();
    let intersection = graph.kronecker(&request.graph);
    let adj = intersection.adjacency_matrix();
    let intersection_time = time.elapsed();

    for (name, func) in LIST {
        let mut closure = adj.clone();
        let time = Instant::now();
        func(&mut closure);
        let close_time = time.elapsed();

        let time = Instant::now();
        let pairs = closure.extract_pairs();
        let pairs_time = time.elapsed();
        let pairs_count = pairs.len();

        map.insert(name.to_string(), (close_time.as_nanos(), pairs_time.as_nanos(), pairs_count));
    }
    (intersection_time.as_nanos(), map)
}
