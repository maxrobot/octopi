use octopi::stream_transactions;
use octopi::{engine::Engine, transaction::Transaction};

use std::env;
use std::error::Error;
use std::io::stdout;
use std::path::Path;
use tokio::sync::mpsc;

const DEFAULT_CHANNEL_SIZE: usize = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let csv_path = parse_args();

    validate_csv_file(&csv_path);
    process_transactions(&csv_path).await
}

fn parse_args() -> String {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => "transactions.csv".to_string(),
        2 => args[1].clone(),
        _ => {
            eprintln!("Usage: {} [csv_file]", args[0]);
            eprintln!("  csv_file: Path to CSV file (default: transactions.csv)");
            std::process::exit(1);
        }
    }
}

fn validate_csv_file(path: &str) {
    if !Path::new(path).exists() {
        eprintln!("Error: File '{}' does not exist", path);
        std::process::exit(1);
    }

    if !path.to_lowercase().ends_with(".csv") {
        eprintln!("Error: File '{}' is not a CSV file", path);
        std::process::exit(1);
    }
}

async fn process_transactions(csv_path: &str) -> Result<(), Box<dyn Error>> {
    let txs = stream_transactions(csv_path)?;

    // Create a channel to send transactions to the engine
    // NOTE: if we wanted to have multiple senders then we could clone the channel and
    // have many threads sending to the same recevier `rx`
    let (tx_channel, mut rx) = mpsc::channel::<Transaction>(DEFAULT_CHANNEL_SIZE);

    // Spawn engine task
    let engine_handle = tokio::spawn(async move {
        let mut engine = Engine::default();

        while let Some(tx) = rx.recv().await {
            if let Err(e) = engine.apply_transaction(tx) {
                eprintln!("Engine error: {:?}", e);
            }
        }

        engine.dump_accounts(stdout());
    });

    // Process CSV transactions
    for csv_tx in txs {
        match csv_tx.try_into() {
            Ok(parsed_tx) => {
                tx_channel.send(parsed_tx).await.expect("Receiver dropped");
            }
            Err(e) => {
                eprintln!("Transaction conversion error: {:?}", e);
            }
        }
    }

    // Close the channel to signal the engine task to finish
    drop(tx_channel);

    // Wait for the engine task to complete
    engine_handle.await?;

    Ok(())
}
