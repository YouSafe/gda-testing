use std::process::Stdio;

use clap::builder::styling::Style;
use smol::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    stream::StreamExt,
};

use crate::graph::Graph;

pub struct Optimizer {
    name: String,
    /// The child process.
    /// It is not terminated when the optimizer is dropped (see smol docs).
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Optimizer {
    pub fn new(command: &str, name: String) -> Self {
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
            name,
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
        let name = self.name.clone();
        async move {
            while let Some(line) = lines.next().await {
                eprintln!("{log_info_style}[{}] {}{log_info_style:#}", name, line?);
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

            if line == "DONE" {
                return Ok(OptimizerResponse::Done);
            }

            let graph = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(e) => {
                    panic!("Failed to parse {:?} because of {}", line, e);
                }
            };

            Ok(OptimizerResponse::Graph(graph))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub enum OptimizerResponse {
    Graph(Graph),
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
