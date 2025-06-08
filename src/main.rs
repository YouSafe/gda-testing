use clap::Parser;
use cli::Cli;
use comparer::compare_mode;
use graphs_runner::GraphsModeRunner;
use leaderboard::{plots::plot_leaderboard, stats::read_all_runs};
use smol::{channel, future, io};

pub mod cli;
pub mod comparer;
pub mod graph;
pub mod graphs_runner;
pub mod leaderboard;
pub mod optimizer_protocol;

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
        cli::CliCommands::Graphs {
            optimizer,
            filter,
            skip_to,
            save,
        } => smol::block_on(future::or(
            async move {
                is_interrupted.await;
                io::Result::Ok(())
            },
            async {
                _ = GraphsModeRunner {
                    command: optimizer,
                    filter,
                    skip_to,
                    save
                }
                .run()
                .await?;
                Ok(())
            },
        )),
        cli::CliCommands::Leaderboard {} => {
            plot_leaderboard(read_all_runs()?)?;
            Ok(())
        }
        cli::CliCommands::Adversary {} => {
            println!("Not yet implemented");
            Ok(())
        }
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
