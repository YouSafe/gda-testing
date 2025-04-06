use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use clap::Parser;
use cli::Cli;
use smol::{channel, io};

pub mod cli;
pub mod compare_mode;
pub mod graph;
pub mod leaderboard_mode;
pub mod sprt;

fn main() -> io::Result<()> {
    let is_interrupted = get_ctrl_c();

    let cli = Cli::parse();

    match cli.command {
        cli::CliCommands::Compare(compare_args) => {
            smol::block_on(compare_mode::compare_mode(compare_args, is_interrupted))
        }
        cli::CliCommands::Leaderboard { optimizer } => smol::block_on(
            leaderboard_mode::leaderboard_mode(optimizer, is_interrupted),
        ),
    }
}

fn get_ctrl_c() -> impl Future<Output = ()> {
    let (s, ctrl_c) = channel::bounded(10);
    let handle = move || {
        s.try_send(()).ok();
    };
    ctrlc::set_handler(handle).unwrap();

    async move {
        while !ctrl_c.recv().await.is_ok() {
            // Wait
        }
    }
}
