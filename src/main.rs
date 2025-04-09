use clap::Parser;
use cli::Cli;
use leaderboard::{plots::plot_runs, run_statistics::write_run, subcommand::leaderboard_mode};
use smol::{channel, future, io};

pub mod cli;
pub mod compare_mode;
pub mod graph;
pub mod leaderboard;
pub mod optimizer_protocol;
pub mod sprt;

// For faster compile times, we could
// - Use the Clap builder API
// - Use a different library, see https://github.com/rosetta-rs/argparse-rosetta-rs
// - Investigate a different Serde library, like nanoserde

fn main() -> io::Result<()> {
    let is_interrupted = get_ctrl_c();
    let cli = Cli::parse();

    match cli.command {
        cli::CliCommands::Compare(compare_args) => smol::block_on(future::or(
            async move {
                is_interrupted.await;
                io::Result::Ok(())
            },
            compare_mode::compare_mode(compare_args),
        )),
        cli::CliCommands::Leaderboard {
            name,
            optimizer,
            filter,
        } => smol::block_on(future::or(
            async move {
                is_interrupted.await;
                io::Result::Ok(())
            },
            async {
                let run = leaderboard_mode(optimizer, filter).await?;
                let run_statistics = write_run(name, run)?;
                plot_runs(run_statistics);
                Ok(())
            },
        )),
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
