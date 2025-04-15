use std::process::Stdio;

use clap::builder::styling::Style;
use smol::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    stream::StreamExt,
};

use crate::graph::Graph;

pub struct Optimizer {
    id: u32,
    /// The child process.
    /// It is not terminated when the optimizer is dropped (see smol docs).
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Optimizer {
    pub fn new(command: &str, id: u32) -> Self {
        #[cfg(target_os = "windows")] // For Windows with its backslashes
        let command = winsplit::split(&command);
        #[cfg(not(target_os = "windows"))] // For sane OSes
        let command = &shlex::split(&command).unwrap();

        let mut process = Command::new(&command[0])
            .args(command[1..].iter().map(|v| std::ffi::OsStr::new(v)))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute optimizer");

        let stdin = process.stdin.take().expect("failed to get child stdin");
        let stdout = BufReader::new(process.stdout.take().expect("failed to get child stdout"));

        Self {
            id,
            process,
            stdin,
            stdout,
        }
    }

    /// Redirects stderr to this process's stdout
    #[must_use]
    pub fn redirect_stderr(&mut self) -> impl Future<Output = io::Result<()>> + use<> {
        let child_stderr = self
            .process
            .stderr
            .take()
            .expect("failed to get child stderr");
        let log_info_style = Style::new().dimmed();
        let mut lines = BufReader::new(child_stderr).lines();
        let id = self.id;
        async move {
            while let Some(line) = lines.next().await {
                eprintln!(
                    "{log_info_style}[Optimizer {}] {}{log_info_style:#}",
                    id, line?
                );
            }
            io::Result::Ok(())
        }
    }

    /// Writes a graph to the child
    #[must_use]
    pub fn write_graph(&mut self, graph: &Graph) -> impl Future<Output = io::Result<()>> {
        let graph_bytes = serde_json::to_vec(graph).unwrap();
        async move { self.write_graph_bytes(&graph_bytes).await }
    }

    /// Writes a graph to the child
    #[must_use]
    pub fn write_graph_bytes(&mut self, graph: &[u8]) -> impl Future<Output = io::Result<()>> {
        async move {
            self.stdin.write_all(&graph).await?;
            self.stdin.write_all(b"\n").await?;
            self.stdin.flush().await?;
            Ok(())
        }
    }

    /// Reads a response from the optimizer
    #[must_use]
    pub fn read_response(&mut self) -> impl Future<Output = io::Result<OptimizerResponse>> {
        async {
            let mut line = String::new();
            // Skip empty lines
            while line.trim_end().len() == 0 {
                self.stdout.read_line(&mut line).await?;
            }

            let line = line.trim_end();

            if let Some(rest) = starts_with(line, "START") {
                Ok(OptimizerResponse::Start {
                    name: rest.trim_ascii().to_string(),
                })
            } else if let Some(rest) = starts_with(line, "GRAPH") {
                let text = rest.trim_ascii().to_string();
                let mut line = String::new();
                self.stdout.read_line(&mut line).await?;
                let graph = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        panic!("Failed to parse {:?} because of {}", line, e);
                    }
                };
                Ok(OptimizerResponse::Graph { text, graph })
            } else if line == "DONE" {
                Ok(OptimizerResponse::Done)
            } else {
                panic!("Unknown response {}", line)
            }
        }
    }

    #[must_use]
    pub fn read_start(&mut self) -> impl Future<Output = io::Result<String>> {
        async {
            let response = self.read_response().await?;
            if let OptimizerResponse::Start { name } = response {
                Ok(name)
            } else {
                panic!("Expected ready, but got {:?}", response)
            }
        }
    }

    #[must_use]
    pub fn read_graphs(&mut self) -> impl Future<Output = io::Result<Vec<(String, Graph)>>> {
        async {
            let mut results = vec![];
            loop {
                match self.read_response().await? {
                    OptimizerResponse::Graph { text, graph } => results.push((text, graph)),
                    OptimizerResponse::Done => break,
                    other => panic!(
                        "{} should have returned a graph but instead returned {:?}",
                        self.id, other
                    ),
                }
            }
            if results.is_empty() {
                panic!("{} should have returned at least one graph", self.id)
            }
            Ok(results)
        }
    }
}

/// Checks if text starts with a pattern, and returns the remaining text
fn starts_with<'a>(text: &'a str, pattern: &str) -> Option<&'a str> {
    if text.starts_with(pattern) {
        Some(&text[pattern.len()..])
    } else {
        None
    }
}

#[derive(Debug)]
pub enum OptimizerResponse {
    Start { name: String },
    Graph { text: String, graph: Graph },
    Done,
}

pub trait AllOk<T, E> {
    type TOut;
    /// Does the same as
    /// ```rs
    /// let (a, b) = my_result;
    /// return (a?, b?);
    /// ```
    fn all_ok(self) -> Result<Self::TOut, E>;
}

impl<T, E> AllOk<T, E> for (Result<T, E>, Result<T, E>) {
    type TOut = (T, T);

    fn all_ok(self) -> Result<Self::TOut, E> {
        match self {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}
