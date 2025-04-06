use std::{
    path::{Path, PathBuf},
    process::Stdio,
};

use smol::{
    fs, future,
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
    stream::StreamExt,
};

use crate::graph::Graph;

pub fn leaderboard_mode(
    optimizer: &[String],
    is_interrupted: impl Future<Output = ()>,
) -> impl Future<Output = io::Result<()>> {
    println!("Starting {} with args {:?}", &optimizer[0], &optimizer[1..]);
    let mut optimizer = Command::new(&optimizer[0])
        .args(optimizer[1..].iter().map(|v| std::ffi::OsStr::new(v)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Log child error m
        .spawn()
        .expect("failed to execute optimizer");

    let mut child_stdin = optimizer.stdin.take().expect("failed to get child stdin");
    let child_stdout = optimizer.stdout.take().expect("failed to get child stdout");
    let child_stderr = optimizer.stderr.take().expect("failed to get child stdout");

    let redirect_stderr = async {
        let mut lines = BufReader::new(child_stderr).lines();
        while let Some(line) = lines.next().await {
            eprintln!("[Optimizer] {}", line?);
        }
        io::Result::Ok(())
    };

    let graphs = collect_graphs(Path::new("./graphs"))
        .expect("./graphs folder should exist and be full of graphs");
    let read_graphs = async move {
        let mut child_stdout = BufReader::new(child_stdout)
            .lines()
            .filter(|v| !matches!(v.as_deref(), Ok("")));
        for graph_path in graphs {
            println!("Processing: {}", graph_path.display());
            let graph = fs::read(graph_path)
                .await?
                .into_iter()
                .map(|v| if v == b'\n' { b' ' } else { v })
                .collect::<Vec<_>>();

            child_stdin.write_all(&graph).await?;
            child_stdin.write(b"\n").await?;
            child_stdin.flush().await?;

            let line = child_stdout
                .next()
                .await
                .expect("Expected child stream to be open")
                .expect("Expected child to return an optimized graph");

            let graph: Graph = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    panic!("Failed to parse {:?} because of {}", line, e);
                }
            };

            let num_edge_crossings = graph.crossings();
            println!("Max edge crossing: {}", num_edge_crossings.max_per_edge);
        }

        Ok(())
    };

    async {
        let is_interrupted = async move {
            _ = is_interrupted.await;
            (io::Result::Ok(()), io::Result::Ok(()))
        };
        let (looper, redirecter) =
            future::or(is_interrupted, future::zip(read_graphs, redirect_stderr)).await;
        looper?;
        redirecter?;
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
