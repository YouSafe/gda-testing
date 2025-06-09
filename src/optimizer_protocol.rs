use std::process::Stdio;

use clap::builder::styling::{self, Style};
use smol::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    stream::StreamExt,
};

use crate::graph::Graph;

pub struct Optimizer {
    id: u32,
    command: Vec<String>,
    /// The child process.
    /// It is not terminated when the optimizer is dropped (see smol docs).
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

pub static LOG_INFO: Style = Style::new().dimmed();
pub static LOG_WARN: Style = Style::new()
    .dimmed()
    .fg_color(Some(styling::Color::Ansi(styling::AnsiColor::Yellow)));
pub static LOG_ERROR: Style =
    Style::new().fg_color(Some(styling::Color::Ansi(styling::AnsiColor::Red)));

impl Optimizer {
    pub fn new(command: &str, id: u32) -> Self {
        #[cfg(target_os = "windows")] // For Windows with its backslashes
        let command = winsplit::split(&command);
        #[cfg(not(target_os = "windows"))] // For sane OSes
        let command = &shlex::split(&command).unwrap();

        Self::from_command(id, command)
    }

    fn from_command(id: u32, command: Vec<String>) -> Self {
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
            command,
            process,
            stdin,
            stdout,
        }
    }

    pub fn restart(&mut self) -> impl Future<Output = io::Result<()>> + use<> {
        let id = self.id;
        let command = std::mem::take(&mut self.command);
        let mut opt = Self::from_command(id, command);
        std::mem::swap(self, &mut opt);

        async move {
            // Explicitly close the stdin of the old optimizer
            opt.stdin.close().await?;
            // And kill it
            _ = opt
                .process
                .kill()
                .inspect_err(|e| eprintln!("Killing the old optimizer failed {:?}", e));

            // And the stderr will get killed after a while anyways
            Ok(())
        }
    }

    /// Redirects stderr to this process's stdout
    pub fn take_stderr(&mut self) -> ChildStderr {
        self.process
            .stderr
            .take()
            .expect("failed to get child stderr")
    }

    #[must_use]
    pub fn redirect_stderr(&mut self) -> impl Future<Output = io::Result<()>> + Send + use<> {
        let mut lines = BufReader::new(self.take_stderr()).lines();
        let id = self.id;
        async move {
            while let Some(line) = lines.next().await {
                eprintln!("{LOG_INFO}[Optimizer {}] {}{LOG_INFO:#}", id, line?);
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
        let id = self.id;
        async move {
            loop {
                let mut line = String::new();
                self.stdout.read_line(&mut line).await?;
                if line.len() == 0 {
                    if !matches!(self.process.try_status(), Ok(None)) {
                        return Ok(OptimizerResponse::Done);
                    }
                    continue;
                }

                if let Some(rest) = starts_with(&line, "START") {
                    return Ok(OptimizerResponse::Start {
                        name: rest.trim_ascii().to_string(),
                    });
                } else if line.starts_with("GRAPH") {
                    return Ok(OptimizerResponse::GraphRequest);
                } else if line.starts_with("{") {
                    let graph = match serde_json::from_str(&line) {
                        Ok(v) => v,
                        Err(e) => {
                            panic!("Failed to parse {:?} because of {}", line, e);
                        }
                    };
                    return Ok(OptimizerResponse::Graph { graph });
                } else {
                    // Optimizers shouldn't print to stdout, but whatever
                    eprintln!(
                        "{LOG_WARN}[Optimizer to stdout {}] {}{LOG_WARN:#}",
                        id, line
                    );
                }
            }
        }
    }

    #[must_use]
    pub fn read_start(&mut self) -> impl Future<Output = io::Result<String>> {
        async {
            match self.read_response().await? {
                OptimizerResponse::Start { name } => Ok(name),
                response => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("expected start, but got {:?}", response),
                )),
            }
        }
    }

    #[must_use]
    pub fn read_graph(&mut self) -> impl Future<Output = io::Result<Graph>> {
        async {
            match self.read_response().await? {
                OptimizerResponse::Graph { graph } => Ok(graph),
                response => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("expected graph, but got {:?}", response),
                )),
            }
        }
    }

    #[must_use]
    pub fn read_graph_request(&mut self) -> impl Future<Output = io::Result<()>> {
        async {
            match self.read_response().await? {
                OptimizerResponse::GraphRequest => Ok(()),
                response => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("expected graph request, but got {:?}", response),
                )),
            }
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
    GraphRequest,
    Graph { graph: Graph },
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
