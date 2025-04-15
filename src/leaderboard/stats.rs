use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamStats {
    /// Team name
    pub name: String,
    pub runs: Vec<SingleRun>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleRun {
    /// Name and parameters of the optimizer
    pub optimizer: String,
    /// Name of the graph
    pub graph: String,
    pub max_per_edge: u32,
    /// How long this run took
    pub duration_ms: u32,
    /// Unix timestamp in seconds of this set of runs
    pub unix_timestamp: u64,
}

pub fn get_sys_time_in_secs() -> u64 {
    match std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

// let mut statistics = read_statistics(name.clone()).expect("statistics file should be readable");
pub fn write_runs(name: String, mut new_runs: Vec<SingleRun>) -> std::io::Result<TeamStats> {
    let mut path = PathBuf::from("./stats");
    path.push(name.clone());
    path.set_extension("csv");

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;
    let mut run_statistics = if file_contents.trim() == "" {
        vec![]
    } else {
        read_runs(file_contents.as_bytes()).expect("statics file must be valid RunStatistics CSV")
    };
    run_statistics.append(&mut new_runs);
    file.set_len(0)?;
    file.rewind()?;
    let mut wtr = csv::Writer::from_writer(file);
    for run in &run_statistics {
        wtr.serialize(run)?;
    }
    wtr.flush()?;
    Ok(TeamStats {
        name,
        runs: run_statistics,
    })
}

pub fn read_all_runs() -> std::io::Result<Vec<TeamStats>> {
    let mut all_runs: Vec<TeamStats> = vec![];
    for entry in std::fs::read_dir("./stats")? {
        let entry = entry?;
        let reader = File::open(entry.path())?;
        all_runs.push(match read_runs(reader) {
            Ok(runs) => TeamStats {
                name: entry
                    .path()
                    .file_stem()
                    .expect("File name needs to exist")
                    .to_string_lossy()
                    .into_owned(),
                runs,
            },
            Err(e) => {
                panic!("Parsing {:?} failed {}", entry, e);
            }
        });
    }
    all_runs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(all_runs)
}

/// For analysis, just point a pivot table at the data.
fn read_runs<R: std::io::Read>(rdr: R) -> csv::Result<Vec<SingleRun>> {
    let mut results = vec![];
    let mut rdr = csv::Reader::from_reader(rdr);
    for result in rdr.deserialize() {
        let record: SingleRun = result?;
        results.push(record);
    }
    Ok(results)
}

// algorithm,
