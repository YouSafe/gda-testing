use clap::Parser;
use cli::Cli;
use smol::{channel, future, io};

pub mod cli;
pub mod compare_mode;
pub mod graph;
pub mod leaderboard_mode;
pub mod optimizer_protocol;
pub mod sprt;

fn main() -> io::Result<()> {
    let is_interrupted = get_ctrl_c();

    let cli = Cli::parse();

    match cli.command {
        cli::CliCommands::Compare(compare_args) => smol::block_on(async {
            future::or(
                async move {
                    is_interrupted.await;
                    io::Result::Ok(())
                },
                compare_mode::compare_mode(compare_args),
            )
            .await
        }),
        cli::CliCommands::Leaderboard { optimizer } => smol::block_on(async {
            future::or(
                async move {
                    is_interrupted.await;
                    io::Result::Ok(())
                },
                leaderboard_mode::leaderboard_mode(optimizer),
            )
            .await
        }),
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
