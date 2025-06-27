use crate::error::EngineError;

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub client: u16,
    pub tx_id: u32,
    pub kind: TransactionType,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Deserialize)]
pub struct CsvTransaction {
    #[serde(rename = "type")]
    pub kind: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<Decimal>,
}

impl TryFrom<CsvTransaction> for Transaction {
    type Error = EngineError;

    fn try_from(csv: CsvTransaction) -> Result<Self, Self::Error> {
        // Validate amount presence for deposit/withdrawal
        match csv.kind {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                if csv.amount.is_none() {
                    // TODO: probably should be a different error type
                    return Err(EngineError::InvalidTransaction {
                        message: format!("Missing amount for transaction {}", csv.tx),
                    });
                }
            }
            _ => {}
        }

        Ok(Transaction {
            kind: csv.kind,
            client: csv.client,
            tx_id: csv.tx,
            amount: csv.amount,
        })
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl Transaction {
    pub fn is_valid(&self) -> bool {
        match self.kind {
            TransactionType::Deposit | TransactionType::Withdrawal => {
                self.amount.is_some() && self.amount.unwrap() > Decimal::ZERO
            }
            TransactionType::Dispute { .. }
            | TransactionType::Resolve { .. }
            | TransactionType::Chargeback { .. } => self.amount.is_none(),
        }
    }

    pub fn new_deposit(client: u16, tx_id: u32, amount: Decimal) -> Self {
        if amount <= Decimal::ZERO {
            eprintln!("Deposit amount must be positive");
        }
        Self {
            client,
            tx_id,
            kind: TransactionType::Deposit,
            amount: Some(amount),
        }
    }

    pub fn new_withdrawal(client: u16, tx_id: u32, amount: Decimal) -> Self {
        if amount <= Decimal::ZERO {
            eprintln!("Withdrawal amount must be positive");
        }
        Self {
            client,
            tx_id,
            kind: TransactionType::Withdrawal,
            amount: Some(amount),
        }
    }

    pub fn new_dispute(client: u16, tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Dispute,
            amount: None,
        }
    }

    pub fn new_resolve(client: u16, tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Resolve,
            amount: None,
        }
    }

    pub fn new_chargeback(client: u16, tx_id: u32) -> Self {
        Self {
            client,
            tx_id,
            kind: TransactionType::Chargeback,
            amount: None,
        }
    }
}
