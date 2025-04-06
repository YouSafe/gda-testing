use std::{
    path::{Path, PathBuf},
    process::Stdio,
    thread,
};

use smol::{
    channel::Receiver,
    future,
    io::{self, AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    spawn,
    stream::StreamExt,
};

pub fn leaderboard_mode(
    optimizer: PathBuf,
    is_interrupted: impl Future<Output = ()>,
) -> impl Future<Output = io::Result<()>> {
    let mut optimizer = Command::new(&optimizer)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // Log child error m
        .spawn()
        .expect("failed to execute optimizer");

    let child_stdin = optimizer.stdin.take().expect("failed to get child stdin");
    let child_stdout = optimizer.stdout.take().expect("failed to get child stdout");
    let child_stderr = optimizer.stderr.take().expect("failed to get child stdout");

    let redirect_stderr = async {
        let mut lines = BufReader::new(child_stderr).lines();
        while let Some(line) = lines.next().await {
            eprintln!("Child process reported {}", line?);
        }
        io::Result::Ok(())
    };

    let graphs = collect_graphs(Path::new("./graphs"))
        .expect("./graphs folder should exist and be full of graphs");
    loop {
        break;
    }

    async {
        let is_interrupted = async move {
            _ = is_interrupted.await;
            (io::Result::Ok(()), io::Result::Ok(()))
        };
        let (looper, redirecter) = future::or(
            is_interrupted,
            future::zip(async { Ok(()) }, redirect_stderr),
        )
        .await;
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
