use crate::{
    graph::Graph,
    leaderboard::stats::{GraphStats, ResultsWriter, RunStats},
    optimizer_protocol::{Optimizer, OptimizerResponse},
};
use smol::{fs, future, io};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

pub struct GraphsModeRunner {
    pub command: String,
    pub filter: Option<String>,
    // skip_seen: bool
}

impl GraphsModeRunner {
    /// Loads graphs from the filesystem
    /// Starts optimizer
    /// Sends graphs, gets results
    /// Validates results
    /// Restarts optimizer on crashes (goes to the next graph)
    pub fn run(&self) -> impl Future<Output = io::Result<RunStats>> {
        println!("Starting {:?}", self.command);
        let graphs = collect_graphs(Path::new("./graphs"))
            .map(|g| filter_graphs(g, self.filter.as_deref()))
            .expect("./graphs folder should exist and be full of graphs");
        if graphs.is_empty() {
            panic!("No graphs found in the ./graphs folder");
        }
        let graphs_count = graphs.len();

        async move {
            let mut optimizer = Optimizer::new(&self.command, 1);
            let redirect_stderr = optimizer.redirect_stderr();

            let run_optimizer = async move {
                let team_name = optimizer.read_start().await?;
                let mut results_file = ResultsWriter::new(&team_name)?;
                let mut runs = vec![];

                for (graph_index, (graph_path, graph_name)) in graphs.into_iter().enumerate() {
                    println!(
                        "\nOptimizing {} ({graph_index}/{graphs_count} graphs)",
                        graph_path.display(),
                    );
                    let graph_bytes = fs::read(graph_path)
                        .await?
                        .into_iter()
                        .map(|v| if v == b'\n' { b' ' } else { v })
                        .collect::<Vec<_>>();

                    let input_graph: Graph = serde_json::from_slice(&graph_bytes)?;

                    optimizer.read_graph_request().await?;

                    let start_time = Instant::now();
                    optimizer.write_graph_bytes(&graph_bytes).await?;

                    let (graph, mut result) = match optimizer.read_response().await? {
                        OptimizerResponse::Graph { graph } => {
                            let duration_ms = start_time.elapsed().as_millis() as u32;
                            let max_per_edge = graph.crossings().max_per_edge;
                            println!("Optimizer produced a graph with {max_per_edge} crossings");
                            (
                                graph,
                                GraphStats {
                                    graph: graph_name.clone(),
                                    max_per_edge: Some(max_per_edge),
                                    duration_ms,
                                },
                            )
                        }
                        OptimizerResponse::Done => {
                            eprintln!("No graph was returned! Did the optimizer crash?");
                            // TODO: Recreate the optimizer
                            continue;
                        }
                        response => {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("expected graph, but got {:?}", response),
                            ));
                        }
                    };

                    if let Err(e) = graph.is_valid() {
                        result.max_per_edge = None;
                        eprintln!("Graph {} was invalid! {}", result.graph, e);
                    }

                    if input_graph.nodes.len() != graph.nodes.len() {
                        result.max_per_edge = None;
                        eprintln!(
                            "Output graph doesn't have the same number of nodes! Input has {} nodes. Output has {} nodes.",
                            input_graph.nodes.len(),
                            graph.nodes.len(),
                        );
                    }

                    if input_graph.edges.len() != graph.edges.len() {
                        result.max_per_edge = None;
                        eprintln!(
                            "Output graph doesn't have the same number of edges! Input has {} edges. Output has {} edges",
                            input_graph.edges.len(),
                            graph.edges.len(),
                        );
                    }

                    if !input_graph.is_isomorphic(&graph) {
                        eprintln!(
                            "Warning: Output graph did not trivially match the input graph. Did the nodes get relabeled, or did something worse happen?",
                        );
                    }

                    results_file.write_single_run(&result)?;

                    runs.push(result);
                }

                results_file.flush()?;

                io::Result::Ok(RunStats {
                    name: team_name,
                    runs,
                })
            };

            let (runs, b) = future::zip(run_optimizer, redirect_stderr).await;
            let (runs, _) = (runs?, b?);
            Ok(runs)
        }
    }
}

fn filter_graphs(graphs: Vec<(PathBuf, String)>, filter: Option<&str>) -> Vec<(PathBuf, String)> {
    if let Some(filter) = filter {
        graphs
            .into_iter()
            .filter(|(_, name)| name.contains(filter))
            .collect()
    } else {
        graphs
    }
}

/// Collects all graphs for this run, and returns them in a sorted order
fn collect_graphs(dir: &Path) -> std::io::Result<Vec<(PathBuf, String)>> {
    fn collect_graphs_rec(
        dir: &Path,
        name: &str,
        graphs: &mut Vec<(PathBuf, String)>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = format!(
                "{name}/{}",
                entry
                    .file_name()
                    .into_string()
                    .expect("Paths should not have non UTF-8 characters")
            );
            if path.is_dir() {
                collect_graphs_rec(&path, &name, graphs)?;
            } else if path.is_file() {
                graphs.push((path, name));
            }
        }
        Ok(())
    }

    let mut graphs = Vec::new();
    collect_graphs_rec(dir, "", &mut graphs)?;
    graphs.sort();
    Ok(graphs)
}
