pub mod engine;
pub mod error;

use crate::engine::transaction::CsvTransaction;
use csv::ReaderBuilder;
use std::error::Error;
use std::fs::File;

pub fn stream_transactions(
    path: &str,
) -> Result<impl Iterator<Item = Result<CsvTransaction, csv::Error>>, Box<dyn Error>> {
    let file = File::open(path)?;
    let rdr = ReaderBuilder::new().trim(csv::Trim::All).from_reader(file);

    // Into an iterator of Result<CsvTransaction>
    Ok(rdr.into_deserialize())
}
