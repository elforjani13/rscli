use clap::{crate_authors, Parser};
use csv::{ReaderBuilder, StringRecord};
use rand::Rng;
use std::{
    cmp::Ordering,
    collections::BinaryHeap,
    io::{Error, ErrorKind},
    path::PathBuf,
};

#[derive(clap::Parser, Debug)]
#[clap(
    author=crate_authors!(),
    about="Command line tool for random data sampling! 🔀",
    )]
struct Cli {
    /// input tree file
    #[clap(short, long, help = "Input tree file")]
    file: PathBuf,
    /// the number of samples we'd like to get
    #[clap(short, long, help = "The number of samples we'd like to get")]
    sample_count: usize,
    /// the column with the weights
    #[clap(short, long, help = "The column with the weights")]
    weights: Option<String>,
    /// include these rows - names by Id column
    #[clap(long, help = "Include these rows - names by Id column")]
    include: Option<Vec<String>>,
    /// exclude these rows - names by Id column
    #[clap(long, help = "Exclude these rows - names by Id column")]
    exclude: Option<Vec<String>>,
    /// id column -  default is the first one
    #[clap(long, help = "Id column - default is the first one")]
    id_col: Option<String>,
}

#[derive(Debug, PartialEq)]
struct Line {
    record: StringRecord,
    weight: f64,
    randomness: f64,
    position_index: f64,
    tie_breaker: usize,
}

struct DataProc {
    args: Cli,
}

impl Eq for Line {}

impl Ord for Line {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.position_index.partial_cmp(&self.position_index) {
            Some(ordering) => ordering,
            None => {
                log::warn!("Warning: Unprecedented precision challenge detected in float indexes. Resolving tie randomly.");

                let mut rng = rand::thread_rng();
                if rng.gen::<f64>() > 0.5 {
                    other.tie_breaker.cmp(&self.tie_breaker)
                } else {
                    self.tie_breaker.cmp(&other.tie_breaker)
                }
            }
        }
    }
}

impl PartialOrd for Line {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl DataProc {
    fn new() -> Self {
        Self { args: Cli::parse() }
    }

    fn setup_logging() {
        env_logger::init();
        log::trace!("Starting up");
    }

    fn process_args(&self) {
        log::debug!("{:?}", self.args);
    }

    fn setup_writer(&self) -> csv::Writer<std::io::Stdout> {
        csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(std::io::stdout())
    }

    fn setup_reader(&self, delim: u8) -> csv::Reader<std::fs::File> {
        csv::ReaderBuilder::new()
            .delimiter(delim)
            .from_path(&self.args.file)
            .expect("Failed to read the file")
    }

    fn process_data(&self) -> Result<(), Error> {
        let mut rng = rand::thread_rng();
        let mut wtr = self.setup_writer();
        let mut reader = self.setup_reader(b'\t');

        let weight_col =
            match &self.args.weights {
                Some(w) => Some(reader.headers()?.iter().position(|r| r == w).ok_or_else(
                    || Error::new(ErrorKind::NotFound, format!("Column '{}' not found.", w)),
                )?),
                None => None,
            };

        let id_col = match &self.args.id_col {
            Some(w) => reader
                .headers()?
                .iter()
                .position(|r| r == w)
                .expect("Id column not found"),
            None => 0,
        };

        let include: Vec<String> = match &self.args.include {
            Some(tips) => tips.clone(),
            None => vec![],
        };

        let exclude: Vec<String> = match &self.args.exclude {
            Some(tips) => tips.clone(),
            None => vec![],
        };

        reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_path(&self.args.file)
            .expect("Failed to read the input file");

        let mut heap = BinaryHeap::new();
        let mut i = 0;
        for record in reader.records() {
            let record = record?;
            if !exclude.contains(&String::from(record.get(id_col).unwrap())) {
                let weight: f64 = get_weight(weight_col, &record);
                let rng = rng.gen::<f64>();
                let index = if include.contains(&String::from(record.get(id_col).unwrap())) {
                    f64::INFINITY
                } else {
                    (1.0 / weight) * rng.log2()
                };
                if index == 0.0 {
                    panic!("Non-zero weights required for numerical precision.");
                }
                let line = Line {
                    record: record.clone(),
                    weight,
                    randomness: rng,
                    position_index: index,
                    tie_breaker: i,
                };
                log::trace!("pushing line {:?}", &line);
                heap.push(line);
                i += 1;
                if heap.len() > self.args.sample_count {
                    let smallest = heap.pop();
                    if let Some(poor_soul) = smallest {
                        log::trace!("removing line  {:?}", poor_soul)
                    }
                }
            }
        }

        wtr.write_record(reader.headers()?)?;
        for line in heap.iter() {
            wtr.write_record(&line.record)?;
        }
        wtr.flush()?;

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let data_proc = DataProc::new();
    DataProc::setup_logging();
    data_proc.process_args();
    match data_proc.process_data() {
        Ok(_) => Ok(()),

        Err(err) => {
            eprintln!("Error processing data: {}", err);
            Err(err)
        }
    }
}

fn get_weight(column: Option<usize>, record: &StringRecord) -> f64 {
    match column {
        Some(i) => record
            .get(i)
            .and_then(|w| w.parse().ok())
            .unwrap_or(f64::NEG_INFINITY),
        None => 1.0,
    }
}
