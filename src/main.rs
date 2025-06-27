mod engine;
mod error;

use octopi::{
    engine::{engine::Engine, transaction::Transaction},
    stream_transactions,
};

use std::env;
use std::error::Error;
use std::io::stdout;
use std::path::Path;
use tokio::sync::mpsc;

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
    println!("Processing transactions from: {}", csv_path);
    let txs = stream_transactions(csv_path)?;

    // Create a channel to send transactions to the engine
    let (tx_channel, mut rx) = mpsc::channel::<Transaction>(100);

    // Spawn engine task
    tokio::spawn(async move {
        let mut engine = Engine::new();

        while let Some(tx) = rx.recv().await {
            if let Err(e) = engine.apply_transaction(tx) {
                eprintln!("Engine error: {:?}", e);
            }
        }

        engine.dump_accounts(stdout());
    });

    // Process CSV transactions
    for tx_result in txs {
        match tx_result {
            Ok(tx) => {
                let parsed_tx = tx.try_into()?;
                tx_channel.send(parsed_tx).await.expect("Receiver dropped");
            }
            Err(e) => {
                eprintln!("CSV parsing error: {}", e);
            }
        }
    }

    Ok(())
}
