use std::path::{Path, PathBuf};

use smol::{fs, future, io};

use crate::{
    graph::Graph,
    optimizer_protocol::{AllOk, Optimizer, OptimizerResponse},
};

pub fn leaderboard_mode(command: String) -> impl Future<Output = io::Result<()>> {
    println!("Starting {:?}", command);
    let mut optimizer = Optimizer::new(&command, "Optimizer".to_string());
    let redirect_stderr = optimizer.redirect_stderr();

    let graphs = collect_graphs(Path::new("./graphs"))
        .expect("./graphs folder should exist and be full of graphs");
    let run_optimizer = async move {
        for graph_path in graphs {
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
            }
        }

        Ok(())
    };

    async {
        _ = future::zip(run_optimizer, redirect_stderr).await.all_ok()?;
        Ok(())
    }
}

/// Collects all graphs for this run, and returns them in a sorted order
fn collect_graphs(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    fn collect_graphs_rec(dir: &Path, graphs: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_graphs_rec(&path, graphs)?;
            } else if path.is_file() {
                graphs.push(path);
            }
        }
        Ok(())
    }

    let mut graphs = Vec::new();
    collect_graphs_rec(dir, &mut graphs)?;
    graphs.sort();
    Ok(graphs)
}
