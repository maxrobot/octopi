pub mod account;
pub mod engine;
pub mod error;
pub mod transaction;

use crate::transaction::CsvTransaction;
use csv::ReaderBuilder;
use std::fs::File;

pub fn stream_transactions(
    path: &str,
) -> Result<impl Iterator<Item = CsvTransaction>, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let rdr = ReaderBuilder::new().trim(csv::Trim::All).from_reader(file);

    // Filter out invalid records and return only valid CsvTransactions
    Ok(rdr
        .into_deserialize::<CsvTransaction>()
        .filter_map(|result| match result {
            Ok(tx) => Some(tx),
            Err(e) => {
                eprintln!("Skipping invalid CSV line: {}", e);
                None
            }
        }))
}
