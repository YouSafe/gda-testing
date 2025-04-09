use crate::{
    graph::Graph,
    leaderboard::run_statistics::{CrossingStatistic, SingleRun},
    optimizer_protocol::{Optimizer, OptimizerResponse},
};
use smol::{fs, future, io};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

pub fn leaderboard_mode(
    command: String,
    filter: Option<String>,
) -> impl Future<Output = io::Result<SingleRun>> {
    println!("Starting {:?}", command);
    let graphs = collect_graphs(Path::new("./graphs"))
        .map(|g| filter_graphs(g, filter))
        .expect("./graphs folder should exist and be full of graphs");

    async move {
        if graphs.is_empty() {
            return Ok(SingleRun::new());
        }

        let mut optimizer = Optimizer::new(&command, "Optimizer".to_string());
        let redirect_stderr = optimizer.redirect_stderr();

        let run_optimizer = async move {
            let mut run = SingleRun::new();

            for (graph_path, graph_name) in graphs {
                let graph_statistics = run.new_graph(graph_name);
                let start_time = Instant::now();
                println!("\nOptimizing {}", graph_path.display());
                let graph = fs::read(graph_path)
                    .await?
                    .into_iter()
                    .map(|v| if v == b'\n' { b' ' } else { v })
                    .collect::<Vec<_>>();

                optimizer.write_graph_bytes(&graph).await?;

                loop {
                    let graph: Graph = match optimizer.read_response().await? {
                        OptimizerResponse::Graph(graph) => graph,
                        OptimizerResponse::Done => break,
                    };
                    let num_edge_crossings = graph.crossings();
                    println!("Max edge crossing: {}", num_edge_crossings.max_per_edge);
                    graph_statistics.crossings.push(CrossingStatistic {
                        max_per_edge: num_edge_crossings.max_per_edge,
                        duration: start_time.elapsed(),
                    });
                }
            }

            io::Result::Ok(run)
        };

        let (run, b) = future::zip(run_optimizer, redirect_stderr).await;
        let (run, _) = (run?, b?);
        Ok(run)
    }
}

fn filter_graphs(graphs: Vec<(PathBuf, String)>, filter: Option<String>) -> Vec<(PathBuf, String)> {
    if let Some(filter) = filter {
        graphs
            .into_iter()
            .filter(|(_, name)| name.contains(&filter))
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
