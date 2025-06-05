use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct RunStats {
    /// Optimizer name plus version plus parameters
    pub name: String,
    pub runs: Vec<GraphStats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphStats {
    /// Name of the graph
    pub graph: String,
    /// Crossing number. Empty if it was invalid
    pub max_per_edge: Option<u32>,
    /// How long this run took
    pub duration_ms: u32,
}

pub struct ResultsWriter(csv::Writer<File>);

impl ResultsWriter {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let mut path = PathBuf::from("./stats");
        path.push(name);
        path.set_extension("csv");

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(path)?;

        let file_size = file.metadata().map(|v| v.len()).unwrap_or_default();

        let writer = csv::WriterBuilder::new()
            // Only add headers to empty files
            .has_headers(file_size == 0)
            .from_writer(file);
        Ok(Self(writer))
    }

    pub fn write_single_run(&mut self, run: &GraphStats) -> std::io::Result<()> {
        self.0.serialize(run)?;
        // Makes for a much better user experience, at the cost of some useless disk flushes
        self.flush()
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        Ok(self.0.flush()?)
    }
}

pub fn read_all_runs() -> std::io::Result<Vec<RunStats>> {
    let mut all_runs: Vec<RunStats> = vec![];
    for entry in std::fs::read_dir("./stats")? {
        let entry = entry?;
        let reader = File::open(entry.path())?;
        all_runs.push(match read_runs(reader) {
            Ok(runs) => RunStats {
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
fn read_runs<R: std::io::Read>(rdr: R) -> csv::Result<Vec<GraphStats>> {
    let mut results = vec![];
    let mut rdr = csv::Reader::from_reader(rdr);
    for result in rdr.deserialize() {
        let record: GraphStats = result?;
        results.push(record);
    }
    Ok(results)
}

// algorithm,
