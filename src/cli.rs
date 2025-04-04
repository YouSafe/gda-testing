use std::path::PathBuf;

use clap::ValueHint::ExecutablePath;
use clap::{Args, Parser, Subcommand, arg};

#[derive(Debug, Parser)]
pub struct Cli {
    #[clap(long, short)]
    pub raw_mode: bool,

    #[command(subcommand)]
    pub command: CliCommands,
}

#[derive(Debug, Subcommand)]
pub enum CliCommands {
    /// Compares two optimizers.
    Compare(CompareArgs),
    /// Shows where your optimizer ranks in the leaderboard.
    Leaderboard {
        #[arg(value_hint=ExecutablePath)]
        optimizer: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct CompareArgs {
    #[clap(long, default_value = "0.05")]
    pub alpha: f32,

    #[clap(long, default_value = "0.05")]
    pub beta: f32,

    #[clap(long, default_value = "0")]
    pub elo0: u32,

    #[clap(long, default_value = "10")]
    pub elo1: u32,

    #[clap(long, default_value = "2")]
    pub max_games: u32,

    #[clap(long, default_value = "1")]
    pub rounds: u32,

    #[clap(long, short)]
    pub seed: Option<u64>,

    #[arg(value_hint=ExecutablePath)]
    pub optimizer1: PathBuf,

    #[arg(value_hint=ExecutablePath)]
    pub optimizer2: PathBuf,
}
