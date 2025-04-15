use std::sync::Arc;

use petgraph::graph::UnGraph;
use petgraph_gen::random_gnp_graph;
use rand::{Rng, SeedableRng};
use smol::{future, io};

use crate::{
    cli::CompareArgs,
    comparer::sprt::{self, SPRT, elo_wld},
    graph::{Edge, Graph, Node},
    optimizer_protocol::{AllOk, Optimizer},
};

pub fn compare_mode(cli: CompareArgs) -> impl Future<Output = io::Result<()>> {
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

    let mut optimizer1 = Optimizer::new(&cli.optimizer1, 1);
    let mut optimizer2 = Optimizer::new(&cli.optimizer2, 2);

    let redirect_stderr = future::zip(optimizer1.redirect_stderr(), optimizer2.redirect_stderr());

    println!("seed: {seed}");

    let run_optimizers = async move {
        let (name1, name2) = future::zip(optimizer1.read_start(), optimizer2.read_start()).await;
        let (name1, name2) = (name1?, name2?);

        let mut current_instance = 0;
        while current_instance < cli.max_games {
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
                "Started instance {} of {} ({:?} vs {:?})",
                current_instance, cli.max_games, name1, name2
            );

            _ = future::zip(
                optimizer1.write_graph(&graph),
                optimizer2.write_graph(&graph),
            )
            .await
            .all_ok()?;

            let (graphs1, graphs2) =
                future::zip(optimizer1.read_graphs(), optimizer2.read_graphs()).await;
            let (graphs1, graphs2) = (graphs1?, graphs2?);

            let crossings1 = graphs1
                .iter()
                .map(|(_, g)| g.crossings().max_per_edge)
                .max()
                .expect("Optimizer 1 returned no graphs");
            let crossings2 = graphs2
                .iter()
                .map(|(_, g)| g.crossings().max_per_edge)
                .max()
                .expect("Optimizer 2 returned no graphs");
            println!("{} max edge crossing: {}", name1, crossings1);
            println!("{} max edge crossing: {}", name2, crossings2);

            match crossings1.cmp(&crossings2) {
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
        io::Result::Ok(())
    };

    async {
        let (a, (b, c)) = future::zip(run_optimizers, redirect_stderr).await;
        a?;
        b?;
        c?;
        Ok(())
    }
}
