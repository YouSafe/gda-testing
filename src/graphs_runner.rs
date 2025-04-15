use crate::{
    leaderboard::stats::{SingleRun, TeamStats, get_sys_time_in_secs},
    optimizer_protocol::{Optimizer, OptimizerResponse},
};
use smol::{fs, future, io};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

pub fn graphs_mode(
    command: String,
    filter: Option<String>,
) -> impl Future<Output = io::Result<TeamStats>> {
    println!("Starting {:?}", command);
    let graphs = collect_graphs(Path::new("./graphs"))
        .map(|g| filter_graphs(g, filter))
        .expect("./graphs folder should exist and be full of graphs");
    if graphs.is_empty() {
        panic!("No graphs found in the ./graphs folder");
    }

    async move {
        let unix_timestamp = get_sys_time_in_secs();
        let mut optimizer = Optimizer::new(&command, 1);
        let redirect_stderr = optimizer.redirect_stderr();

        let team_name = optimizer.read_start().await?;

        let run_optimizer = async move {
            let mut runs = vec![];

            for (graph_path, graph_name) in graphs {
                println!("\nOptimizing {}", graph_path.display());
                let graph = fs::read(graph_path)
                    .await?
                    .into_iter()
                    .map(|v| if v == b'\n' { b' ' } else { v })
                    .collect::<Vec<_>>();

                let start_time = Instant::now();
                optimizer.write_graph_bytes(&graph).await?;

                let mut graphs = vec![];
                let mut results = vec![];
                loop {
                    match optimizer.read_response().await? {
                        OptimizerResponse::Graph { text, graph } => {
                            let max_per_edge = graph.crossings().max_per_edge;
                            println!(
                                "Optimizer {text} produced a graph with {max_per_edge} crossings"
                            );
                            results.push(SingleRun {
                                optimizer: text,
                                graph: graph_name.clone(),
                                max_per_edge,
                                duration_ms: start_time.elapsed().as_millis() as u32,
                                unix_timestamp,
                            });
                            graphs.push(graph);
                        }
                        OptimizerResponse::Done => break,
                        other => panic!("Should receive graph but instead got {:?}", other),
                    }
                }
                if results.is_empty() {
                    panic!("Optimizer should have returned at least one graph")
                }

                for (graph, result) in graphs.iter().zip(results.iter()) {
                    match graph.is_valid() {
                        Ok(_) => {}
                        Err(e) => eprintln!("Graph {} was invalid! {}", result.graph, e),
                    }
                }

                runs.append(&mut results);
            }

            io::Result::Ok(runs)
        };

        let (runs, b) = future::zip(run_optimizer, redirect_stderr).await;
        let (runs, _) = (runs?, b?);
        Ok(TeamStats {
            name: team_name,
            runs,
        })
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
