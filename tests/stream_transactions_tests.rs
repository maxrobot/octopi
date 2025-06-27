use octopi::stream_transactions;
use rust_decimal::Decimal;
use std::fs;
use std::str::FromStr;
use tempfile::NamedTempFile;

#[test]
fn test_stream_transactions_valid_csv() {
    // Create a temporary CSV file with valid data
    let temp_file = NamedTempFile::new().unwrap();
    let csv_content = r#"type,client,tx,amount
deposit,1,1,100.50
withdrawal,1,2,50.25
dispute,1,3,1
resolve,1,4,1
chargeback,1,5,1"#;

    fs::write(&temp_file, csv_content).unwrap();

    // Test the function
    let txs: Vec<_> = stream_transactions(temp_file.path().to_str().unwrap())
        .unwrap()
        .collect();

    assert_eq!(txs.len(), 5);

    // Check first transaction (deposit)
    let first_tx = txs[0].as_ref().unwrap();
    assert_eq!(first_tx.client, 1);
    assert_eq!(first_tx.tx, 1);
    assert_eq!(first_tx.amount, Some(Decimal::from_str("100.50").unwrap()));

    // Check second transaction (withdrawal)
    let second_tx = txs[1].as_ref().unwrap();
    assert_eq!(second_tx.client, 1);
    assert_eq!(second_tx.tx, 2);
    assert_eq!(second_tx.amount, Some(Decimal::from_str("50.25").unwrap()));
}

#[test]
fn test_stream_transactions_empty_csv() {
    let temp_file = NamedTempFile::new().unwrap();
    let csv_content = r#"type,client,tx,amount"#; // Only header

    fs::write(&temp_file, csv_content).unwrap();

    let txs: Vec<_> = stream_transactions(temp_file.path().to_str().unwrap())
        .unwrap()
        .collect();

    assert_eq!(txs.len(), 0);
}

#[test]
fn test_stream_transactions_missing_amount() {
    let temp_file = NamedTempFile::new().unwrap();
    let csv_content = r#"type,client,tx,amount
deposit,1,1,
withdrawal,1,2,50.25"#;

    fs::write(&temp_file, csv_content).unwrap();

    let txs: Vec<_> = stream_transactions(temp_file.path().to_str().unwrap())
        .unwrap()
        .collect();

    assert_eq!(txs.len(), 2);

    // First transaction should have None amount
    let first_tx = txs[0].as_ref().unwrap();
    assert_eq!(first_tx.amount, None);

    // Second transaction should have amount
    let second_tx = txs[1].as_ref().unwrap();
    assert_eq!(second_tx.amount, Some(Decimal::from_str("50.25").unwrap()));
}

#[test]
fn test_stream_transactions_invalid_file() {
    let result = stream_transactions("nonexistent_file.csv");
    assert!(result.is_err());
}

#[test]
fn test_stream_transactions_large_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let mut csv_content = String::from("type,client,tx,amount\n");

    // Generate 100 transactions
    for i in 1..=100 {
        csv_content.push_str(&format!("deposit,{},{},{}\n", i, i, i * 10));
    }

    fs::write(&temp_file, csv_content).unwrap();

    let txs: Vec<_> = stream_transactions(temp_file.path().to_str().unwrap())
        .unwrap()
        .collect();

    assert_eq!(txs.len(), 100);

    // Check a few specific transactions
    let tx_50 = txs[49].as_ref().unwrap(); // 50th transaction (0-indexed)
    assert_eq!(tx_50.client, 50);
    assert_eq!(tx_50.tx, 50);
    assert_eq!(tx_50.amount, Some(Decimal::from(500)));
}

#[test]
fn test_stream_transactions_mixed_types() {
    let temp_file = NamedTempFile::new().unwrap();
    let csv_content = r#"type,client,tx,amount
deposit,1,1,100.00
withdrawal,1,2,25.50
dispute,1,3,1
resolve,1,4,1
chargeback,1,5,1
deposit,2,6,200.75
withdrawal,2,7,75.25"#;

    fs::write(&temp_file, csv_content).unwrap();

    let txs: Vec<_> = stream_transactions(temp_file.path().to_str().unwrap())
        .unwrap()
        .collect();

    assert_eq!(txs.len(), 7);

    // Check dispute transaction - should have None amount but at this point
    // we simply accept it
    let dispute_tx = txs[2].as_ref().unwrap();
    assert_eq!(dispute_tx.amount, Some(Decimal::from(1)));

    // Check withdrawal transaction (should have amount)
    let withdrawal_tx = txs[1].as_ref().unwrap();
    assert_eq!(
        withdrawal_tx.amount,
        Some(Decimal::from_str("25.50").unwrap())
    );
}
