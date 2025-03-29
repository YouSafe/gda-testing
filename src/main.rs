use std::{
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use clap::Parser;
use cli::Cli;
use graph::{Edge, Graph, Node};
use petgraph::graph::UnGraph;
use petgraph_gen::random_gnp_graph;
use rand::{Rng, SeedableRng};
use sprt::{SPRT, elo_wld};

pub mod cli;
pub mod graph;
pub mod sprt;

fn main() -> anyhow::Result<()> {
    let stop = Arc::new(AtomicBool::new(false));

    let stop_clone = stop.clone();
    ctrlc::set_handler(move || {
        stop_clone.store(true, Ordering::SeqCst);
    })?;

    let cli = Cli::parse();

    let sprt = SPRT::new(cli.elo0, cli.elo1, cli.alpha, cli.beta);

    let mut wins = 0;
    let mut draws = 0;
    let mut losses = 0;

    let mut thread_rng = rand::thread_rng();
    let seed = match cli.seed {
        Some(seed) => seed,
        None => thread_rng.r#gen(),
    };

    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);

    println!("seed: {seed}");

    let mut current_instance = 0;

    while current_instance < cli.max_games && !stop.load(Ordering::SeqCst) {
        let nodes = rng.gen_range(10..200);
        let probability = rng.gen_range(0.1..0.8);

        let graph: UnGraph<_, _, usize> = random_gnp_graph(&mut rng, nodes, probability);
        println!(
            "[{current_instance}] #node: {}, #edge: {}",
            graph.node_count(),
            graph.edge_count()
        );

        let graph = Arc::new(Graph {
            nodes: (0..graph.node_count())
                .into_iter()
                .map(|id| Node {
                    id,
                    x: rng.gen_range(0..1000),
                    y: rng.gen_range(0..1000),
                })
                .collect(),
            points: vec![],
            edges: graph
                .raw_edges()
                .into_iter()
                .map(|e| Edge {
                    source: e.source().index(),
                    target: e.target().index(),
                })
                .collect(),
            width: 1_000_000,
            height: 1_000_000,
        });

        println!(
            "Started instance {} of {} ({} vs {})",
            current_instance,
            cli.max_games,
            cli.optimizer1.display(),
            cli.optimizer2.display()
        );

        let mut optimizer1 = Command::new(&cli.optimizer1)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute optimizer");

        let mut optimizer2 = Command::new(&cli.optimizer2)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to execute optimizer");

        let stdin1 = optimizer1.stdin.take().expect("failed to get child stdin");
        let stdin2 = optimizer2.stdin.take().expect("failed to get child stdin");

        let handle1 = thread::spawn({
            let graph_clone = graph.clone();

            move || {
                serde_json::to_writer(stdin1, graph_clone.as_ref()).unwrap();
            }
        });

        let handle2 = thread::spawn({
            let graph_clone = graph.clone();

            move || {
                serde_json::to_writer(stdin2, graph_clone.as_ref()).unwrap();
            }
        });

        let handle3 = thread::spawn(move || {
            let output = optimizer1.wait_with_output().unwrap();
            let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
            let graph: Graph = serde_json::from_str(&output).unwrap();

            let num_edge_crossings = graph.crossings();
            println!("max edge crossing: {}", num_edge_crossings.max_per_edge);

            num_edge_crossings
        });

        let handle4 = thread::spawn(move || {
            let output = optimizer2.wait_with_output().unwrap();
            let output = std::str::from_utf8(output.stdout.as_slice()).unwrap();
            let graph: Graph = serde_json::from_str(&output).unwrap();

            let num_edge_crossings = graph.crossings();
            println!("max edge crossing: {}", num_edge_crossings.max_per_edge);

            num_edge_crossings
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
        let join1 = handle3.join();
        let join2 = handle4.join();
        let crossings1 = join1.unwrap();
        let crossings2 = join2.unwrap();

        match crossings1.max_per_edge.cmp(&crossings2.max_per_edge) {
            std::cmp::Ordering::Less => wins += 1,
            std::cmp::Ordering::Equal => draws += 1,
            std::cmp::Ordering::Greater => losses += 1,
        }

        let (e1, e2, e3) = elo_wld(wins, losses, draws);
        println!("ELO: {e2:.3} +- {:.3} [{e1:.3}, {e3:.3}]", (e3 - e1) / 2.0);

        let status = sprt.status(wins, losses, draws);
        match status.result {
            sprt::SPRTResult::AcceptH0 => break,
            sprt::SPRTResult::AcceptH1 => break,
            sprt::SPRTResult::Continue => (),
        }

        println!(
            "LLR: {:.3} [{}, {}] ({:.3}, {:.3})",
            status.llr,
            sprt.elo0(),
            sprt.elo1(),
            sprt.lower(),
            sprt.upper()
        );

        current_instance += 1;
    }

    Ok(())
}
