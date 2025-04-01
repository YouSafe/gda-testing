use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use clap::Parser;
use cli::Cli;

pub mod cli;
pub mod compare_mode;
pub mod graph;
pub mod leaderboard_mode;
pub mod sprt;

fn main() -> anyhow::Result<()> {
    let stop = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler({
        let stop = stop.clone();
        move || {
            stop.store(true, Ordering::SeqCst);
        }
    })?;

    let cli = Cli::parse();

    match cli.command {
        cli::CliCommands::Compare(compare_args) => compare_mode::compare_mode(compare_args, stop),
        cli::CliCommands::Leaderboard { optimizer } => todo!(),
    }

    Ok(())
}
