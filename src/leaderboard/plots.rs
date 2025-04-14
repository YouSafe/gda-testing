use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use super::run_statistics::RunStatistics;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct LeaderboardData {
    all_runs: Vec<RunStatistics>,
    /// Graph IDs are implied
    graph_names: Vec<String>,
    /// Uses graph IDs for indexing
    best_crossing_values: Vec<u32>,
}

pub fn plot_runs(all_runs: Vec<RunStatistics>) -> std::io::Result<()> {
    let graph_names = get_graph_names(&all_runs);
    let graph_ids = make_graph_ids(&graph_names);
    let best_crossing_values = get_best_crossing_values(&all_runs, &graph_ids);

    let data = LeaderboardData {
        all_runs,
        graph_names,
        best_crossing_values,
    };

    let file = File::create("leaderboard.json")?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &data)?;
    writer.flush()?;

    println!("Wrote plots data, open ./index.html in a browser!");

    Ok(())
}

fn get_graph_names(all_runs: &[RunStatistics]) -> Vec<String> {
    let mut graph_names = all_runs
        .iter()
        .flat_map(|runs| {
            runs.runs
                .iter()
                .flat_map(|r| r.graphs.iter().map(|g| g.graph.clone()))
        })
        .collect::<Vec<_>>();

    // Sort for displaying
    // I could use
    // https://docs.rs/icu_collator/latest/icu_collator/
    // but that seemed heavyweight, so I went with an alternative
    numeric_sort::sort_unstable(&mut graph_names);

    graph_names
}

fn make_graph_ids(graph_names: &[String]) -> HashMap<String, usize> {
    graph_names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect()
}

fn get_best_crossing_values(
    all_runs: &[RunStatistics],
    graph_ids: &HashMap<String, usize>,
) -> Vec<u32> {
    let mut best_values = vec![u32::MAX; graph_ids.len()];
    for graph in all_runs
        .iter()
        .flat_map(|r| &r.runs)
        .flat_map(|r| &r.graphs)
    {
        let id = graph_ids[&graph.graph] as usize;
        best_values[id] = best_values[id].min(graph.max_per_edge());
    }
    best_values
}
