use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
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
}

pub struct ResultsWriter(csv::Writer<File>);

impl ResultsWriter {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let mut path = PathBuf::from("./stats");
        path.push(name);
        path.set_extension("csv");

        let file = OpenOptions::new().append(true).create(true).open(path)?;
        let writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        Ok(Self(writer))
    }

    pub fn write_single_run(&mut self, run: &SingleRun) -> std::io::Result<()> {
        Ok(self.0.serialize(run)?)
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        Ok(self.0.flush()?)
    }
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
