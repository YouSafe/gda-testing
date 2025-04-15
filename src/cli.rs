use clap::ValueHint::{self};
use clap::{Args, Parser, Subcommand, arg};

#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommands,
}

#[derive(Debug, Subcommand)]
pub enum CliCommands {
    /// Compares two optimizers.
    Compare(CompareArgs),
    /// Runs your solver with a set of graphs
    Graphs {
        #[arg(value_hint=ValueHint::CommandString)]
        optimizer: String,
        /// Filter the input graphs
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Generates a plot for the leaderboard
    Leaderboard {},
    /// Generate evil graphs (WIP)
    Adversary {},
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

    #[arg(value_hint=ValueHint::CommandString)]
    pub optimizer1: String,

    #[arg(value_hint=ValueHint::CommandString)]
    pub optimizer2: String,
}
