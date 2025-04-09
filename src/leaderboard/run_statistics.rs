use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Seek},
    path::PathBuf,
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RunStatistics {
    /// Human readable name of the optimizer
    pub name: String,
    pub runs: Vec<SingleRun>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleRun {
    /// Unix timestamp in seconds
    pub unix_seconds: u64,
    pub graphs: Vec<GraphStatistics>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphStatistics {
    pub graph: String,
    pub crossings: Vec<CrossingStatistic>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct CrossingStatistic {
    pub max_per_edge: u32,
    pub duration: Duration,
}

impl SingleRun {
    pub fn new() -> Self {
        Self {
            unix_seconds: get_sys_time_in_secs(),
            graphs: vec![],
        }
    }

    pub fn new_graph(&mut self, graph: String) -> &mut GraphStatistics {
        self.graphs.push(GraphStatistics {
            graph,
            crossings: vec![],
        });
        self.graphs.last_mut().unwrap()
    }
}

fn get_sys_time_in_secs() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

// let mut statistics = read_statistics(name.clone()).expect("statistics file should be readable");
pub fn write_run(name: String, run: SingleRun) -> std::io::Result<RunStatistics> {
    let mut path = PathBuf::from("./leaderboard");
    path.push(name.clone());
    path.set_extension("json");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let mut run_statistics = if file_contents.trim() == "" {
        // Only set the name on the first run. After that, humans can modify it
        RunStatistics { name, runs: vec![] }
    } else {
        serde_json::from_str(&file_contents).expect("statics file must be valid RunStatistics JSON")
    };
    if run.graphs.len() > 0 {
        run_statistics.runs.push(run);
    }
    file.set_len(0)?;
    file.rewind()?;
    // Use pretty writing for easier git diffs
    serde_json::to_writer_pretty(file, &run_statistics)?;
    Ok(run_statistics)
}
