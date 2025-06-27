use crate::engine::account::Account;
use crate::engine::transaction::{Transaction, TransactionType};
use crate::error::EngineError;

use rust_decimal::Decimal;
use std::collections::HashMap;
use std::io::Write;
use std::str::FromStr;

pub struct Engine {
    accounts: HashMap<u16, Account>,
    transactions: HashMap<u32, Transaction>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    pub fn apply_transaction(&mut self, tx: Transaction) -> Result<(), EngineError> {
        // Retrieve the account else create it
        let entry = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client));

        let tx_already_exists = self.transactions.contains_key(&tx.tx_id);

        if !entry.is_available() {
            return Err(EngineError::AccountLocked(tx.client));
        }

        match tx.kind {
            TransactionType::Deposit => {
                if tx_already_exists {
                    return Err(EngineError::InvalidTransaction {
                        message: "Transaction already exists".to_string(),
                    });
                }
                let amount = tx.amount.ok_or_else(|| EngineError::InvalidTransaction {
                    message: "Deposit must have an amount".to_string(),
                })?;
                deposit(entry, amount)?;
                self.transactions.insert(tx.tx_id, tx);
            }
            TransactionType::Withdrawal => {
                if tx_already_exists {
                    return Err(EngineError::InvalidTransaction {
                        message: "Transaction already exists".to_string(),
                    });
                }
                let amount = tx.amount.ok_or_else(|| EngineError::InvalidTransaction {
                    message: "Withdrawal must have an amount".to_string(),
                })?;
                withdraw(entry, amount)?;
                self.transactions.insert(tx.tx_id, tx);
            }
            TransactionType::Dispute | TransactionType::Resolve | TransactionType::Chargeback => {
                let original = self.transactions.get(&tx.tx_id).ok_or_else(|| {
                    EngineError::InvalidTransaction {
                        message: "Referenced transaction does not exist".to_string(),
                    }
                })?;

                if original.client != tx.client {
                    return Err(EngineError::InvalidTransaction {
                        message: "Referenced transaction does not belong to the same client"
                            .to_string(),
                    });
                }

                if original.kind == TransactionType::Withdrawal {
                    return Err(EngineError::InvalidTransaction {
                        message: "Withdrawal cannot be disputed".to_string(),
                    });
                }

                match tx.kind {
                    TransactionType::Dispute => dispute(entry, original)?,
                    TransactionType::Resolve => resolve(entry, original)?,
                    TransactionType::Chargeback => chargeback(entry, original)?,
                    _ => unreachable!(),
                }
            }
        }

        Ok(())
    }

    pub fn dump_accounts<W: Write>(&self, mut writer: W) {
        // Print CSV header
        writeln!(&mut writer, "client,available,held,total,locked").unwrap();

        for (client, account) in self.accounts.iter() {
            writeln!(
                &mut writer,
                "{},{},{},{},{}",
                client,
                account.available.round_dp(4),
                account.held.round_dp(4),
                account.total.round_dp(4),
                account.locked
            )
            .unwrap();
        }
    }
}

pub fn deposit(account: &mut Account, amount: Decimal) -> Result<(), EngineError> {
    // TODO: check this doesn't overflow
    account.available += amount;
    account.total += amount;

    if account.total < Decimal::ZERO {
        return Err(EngineError::InvalidTransaction {
            message: "Total balance is negative".to_string(),
        });
    }

    Ok(())
}

pub fn withdraw(account: &mut Account, amount: Decimal) -> Result<(), EngineError> {
    if account.available < amount {
        return Err(EngineError::InvalidTransaction {
            message: "Insufficient funds".to_string(),
        });
    }

    account.available -= amount;
    account.total -= amount;

    Ok(())
}

pub fn dispute(account: &mut Account, tx: &Transaction) -> Result<(), EngineError> {
    let mut amount = tx.amount.ok_or(EngineError::InvalidTransaction {
        message: "Transaction has no amount".to_string(),
    })?;

    if account.available < amount {
        amount = account.available;
    }

    account.held += amount;
    account.available -= amount;

    Ok(())
}

pub fn resolve(account: &mut Account, tx: &Transaction) -> Result<(), EngineError> {
    let mut amount = tx.amount.ok_or(EngineError::InvalidTransaction {
        message: "Transaction has no amount".to_string(),
    })?;

    if account.held < amount {
        amount = account.held;
    }

    account.held -= amount;
    account.available += amount;

    Ok(())
}

pub fn chargeback(account: &mut Account, tx: &Transaction) -> Result<(), EngineError> {
    let mut amount = tx.amount.ok_or(EngineError::InvalidTransaction {
        message: "Transaction has no amount".to_string(),
    })?;

    if account.held < amount {
        amount = account.held;
    }

    account.held -= amount;
    account.total -= amount;

    account.locked = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::transaction::Transaction;

    mod apply_transaction_tests {
        use super::*;

        #[test]
        fn test_apply_deposit_transaction() {
            let mut engine = Engine::new();
            let tx = Transaction::new_deposit(1, 1, Decimal::from(100));

            assert!(engine.apply_transaction(tx).is_ok());

            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.total, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
            assert!(!account.locked);

            // Verify transaction was stored
            assert!(engine.transactions.contains_key(&1));
        }

        #[test]
        fn test_apply_withdrawal_transaction() {
            let mut engine = Engine::new();

            // First deposit some money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // Then withdraw
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(50));

            assert!(engine.apply_transaction(withdraw_tx).is_ok());

            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(50));
            assert_eq!(account.total, Decimal::from(50));
            assert_eq!(account.held, Decimal::ZERO);
        }

        #[test]
        fn test_apply_withdrawal_insufficient_funds() {
            let mut engine = Engine::new();

            // Deposit some money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(50));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // Try to withdraw more than available
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(100));

            let result = engine.apply_transaction(withdraw_tx);
            assert!(result.is_err());

            // Account should remain unchanged
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(50));
            assert_eq!(account.total, Decimal::from(50));
        }

        #[test]
        fn test_apply_dispute_transaction() {
            let mut engine = Engine::new();

            // First deposit some money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // Then dispute the transaction
            let dispute_tx = Transaction::new_dispute(1, 1); // Dispute the first transaction

            assert!(engine.apply_transaction(dispute_tx).is_ok());

            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::from(100));
            assert_eq!(account.total, Decimal::from(100));
        }

        #[test]
        fn test_apply_resolve_transaction() {
            let mut engine = Engine::new();

            // First deposit some money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // Then dispute the transaction
            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            // Then resolve the dispute
            let resolve_tx = Transaction::new_resolve(1, 1);

            assert!(engine.apply_transaction(resolve_tx).is_ok());

            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::from(100));
        }

        #[test]
        fn test_apply_chargeback_transaction() {
            let mut engine = Engine::new();

            // First deposit some money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // Then dispute the transaction
            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            // Then chargeback
            let chargeback_tx = Transaction::new_chargeback(1, 1);

            assert!(engine.apply_transaction(chargeback_tx).is_ok());

            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::ZERO);
            assert!(account.locked);
        }

        #[test]
        fn test_duplicate_transaction_id() {
            let mut engine = Engine::new();

            let tx1 = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(tx1).is_ok());

            let tx2 = Transaction::new_deposit(1, 1, Decimal::from(200)); // Same tx_id

            let result = engine.apply_transaction(tx2);
            assert!(result.is_err());

            // Account should only reflect the first transaction
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(100));
            assert_eq!(account.total, Decimal::from(100));
        }

        #[test]
        fn test_account_locked() {
            let mut engine = Engine::new();

            // First deposit and chargeback to lock the account
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            let chargeback_tx = Transaction::new_chargeback(1, 1);
            assert!(engine.apply_transaction(chargeback_tx).is_ok());

            // Try to apply another transaction to locked account
            let new_tx = Transaction::new_deposit(1, 4, Decimal::from(50));

            let result = engine.apply_transaction(new_tx);
            assert!(result.is_err());
            match result {
                Err(EngineError::AccountLocked(client)) => {
                    assert_eq!(client, 1);
                }
                _ => panic!("Expected AccountLocked error"),
            }
        }

        #[test]
        fn test_dispute_nonexistent_transaction() {
            let mut engine = Engine::new();

            let dispute_tx = Transaction::new_dispute(1, 999); // Non-existent transaction

            let result = engine.apply_transaction(dispute_tx);
            assert!(result.is_err());
            match result {
                Err(EngineError::InvalidTransaction { message }) => {
                    assert_eq!(message, "Referenced transaction does not exist");
                }
                _ => panic!("Expected InvalidTransaction error"),
            }
        }

        #[test]
        fn test_resolve_nonexistent_transaction() {
            let mut engine = Engine::new();

            let resolve_tx = Transaction::new_resolve(1, 999); // Non-existent transaction

            let result = engine.apply_transaction(resolve_tx);
            assert!(result.is_err());
            match result {
                Err(EngineError::InvalidTransaction { message }) => {
                    assert_eq!(message, "Referenced transaction does not exist");
                }
                _ => panic!("Expected InvalidTransaction error"),
            }
        }

        #[test]
        fn test_chargeback_nonexistent_transaction() {
            let mut engine = Engine::new();

            let chargeback_tx = Transaction::new_chargeback(1, 999); // Non-existent transaction

            let result = engine.apply_transaction(chargeback_tx);
            assert!(result.is_err());
            match result {
                Err(EngineError::InvalidTransaction { message }) => {
                    assert_eq!(message, "Referenced transaction does not exist");
                }
                _ => panic!("Expected InvalidTransaction error"),
            }
        }

        #[test]
        fn test_multiple_clients() {
            let mut engine = Engine::new();

            // Client 1 deposit
            let tx1 = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(engine.apply_transaction(tx1).is_ok());

            // Client 2 deposit
            let tx2 = Transaction::new_deposit(2, 2, Decimal::from(200));
            assert!(engine.apply_transaction(tx2).is_ok());

            // Verify both accounts exist and are separate
            let account1 = engine.accounts.get(&1).unwrap();
            let account2 = engine.accounts.get(&2).unwrap();

            assert_eq!(account1.available, Decimal::from(100));
            assert_eq!(account1.total, Decimal::from(100));
            assert_eq!(account2.available, Decimal::from(200));
            assert_eq!(account2.total, Decimal::from(200));
        }

        #[test]
        fn test_transaction_storage() {
            let mut engine = Engine::new();

            let tx = Transaction::new_deposit(1, 42, Decimal::from(100));

            assert!(engine.apply_transaction(tx).is_ok());

            // Verify transaction was stored
            assert!(engine.transactions.contains_key(&42));
            let stored_tx = engine.transactions.get(&42).unwrap();
            assert_eq!(stored_tx.client, 1);
            assert_eq!(stored_tx.tx_id, 42);
        }

        #[test]
        fn test_negative_deposit() {
            // This should panic due to validation in constructor
            let result = std::panic::catch_unwind(|| {
                Transaction::new_deposit(1, 1, Decimal::from(-50));
            });
            assert!(result.is_err());
        }

        #[test]
        fn test_zero_deposit() {
            // This should panic due to validation in constructor
            let result = std::panic::catch_unwind(|| {
                Transaction::new_deposit(1, 1, Decimal::ZERO);
            });
            assert!(result.is_err());
        }

        #[test]
        fn test_complex_workflow() {
            let mut engine = Engine::new();

            // 1. Deposit money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(1500));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // 2. Withdraw some money
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(300));
            assert!(engine.apply_transaction(withdraw_tx).is_ok());

            // 3. Deposit money
            let deposit_tx = Transaction::new_deposit(1, 3, Decimal::from(500));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // 4. Dispute the withdrawal
            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            // Verify dispute state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(200));
            assert_eq!(account.held, Decimal::from(1500));
            assert_eq!(account.total, Decimal::from(1700));
            assert!(!account.locked);

            // 5. Resolve the dispute
            let resolve_tx = Transaction::new_resolve(1, 1);
            assert!(engine.apply_transaction(resolve_tx).is_ok());

            // Verify final state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(1700));
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::from(1700));
            assert!(!account.locked);

            // Verify all transactions were stored
            assert_eq!(engine.transactions.len(), 3);
            assert!(engine.transactions.contains_key(&1));
            assert!(engine.transactions.contains_key(&2));
            assert!(engine.transactions.contains_key(&3));
        }

        #[test]
        fn test_dispute_withdrawal() {
            let mut engine = Engine::new();

            // 1. Deposit money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(1500));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // 2. Withdraw some money
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(300));
            assert!(engine.apply_transaction(withdraw_tx).is_ok());

            // 3. Dispute the withdrawal
            let dispute_tx = Transaction::new_dispute(1, 2);
            assert!(engine.apply_transaction(dispute_tx).is_err());

            // Verify final state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(1200));
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::from(1200));
            assert!(!account.locked);

            // Verify all transactions were stored
            assert_eq!(engine.transactions.len(), 2);
            assert!(engine.transactions.contains_key(&1));
            assert!(engine.transactions.contains_key(&2));
        }

        #[test]
        fn test_resolve_dispute_post_withdrawal() {
            let mut engine = Engine::new();

            // 1. Deposit money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(500));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // 2. Withdraw some money
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(250));
            assert!(engine.apply_transaction(withdraw_tx).is_ok());

            // 3. Dispute the withdrawal
            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            // Verify dispute state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::from(250));
            assert_eq!(account.total, Decimal::from(250));
            assert!(!account.locked);

            // 4. Resolve the dispute
            let resolve_tx = Transaction::new_resolve(1, 1);
            assert!(engine.apply_transaction(resolve_tx).is_ok());

            // Verify final state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::from(250));
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::from(250));
            assert!(!account.locked);

            // Verify all transactions were stored
            assert_eq!(engine.transactions.len(), 2);
            assert!(engine.transactions.contains_key(&1));
            assert!(engine.transactions.contains_key(&2));
        }

        #[test]
        fn test_chargeback_dispute_post_withdrawal() {
            let mut engine = Engine::new();

            // 1. Deposit money
            let deposit_tx = Transaction::new_deposit(1, 1, Decimal::from(500));
            assert!(engine.apply_transaction(deposit_tx).is_ok());

            // 2. Withdraw some money
            let withdraw_tx = Transaction::new_withdrawal(1, 2, Decimal::from(250));
            assert!(engine.apply_transaction(withdraw_tx).is_ok());

            // 3. Dispute the withdrawal
            let dispute_tx = Transaction::new_dispute(1, 1);
            assert!(engine.apply_transaction(dispute_tx).is_ok());

            // Verify dispute state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::from(250));
            assert_eq!(account.total, Decimal::from(250));
            assert!(!account.locked);

            // 4. Chargeback the dispute
            let chargeback_tx = Transaction::new_chargeback(1, 1);
            assert!(engine.apply_transaction(chargeback_tx).is_ok());

            // Verify final state
            let account = engine.accounts.get(&1).unwrap();
            assert_eq!(account.available, Decimal::ZERO);
            assert_eq!(account.held, Decimal::ZERO);
            assert_eq!(account.total, Decimal::ZERO);
            assert!(account.locked);

            // Verify all transactions were stored
            assert_eq!(engine.transactions.len(), 2);
            assert!(engine.transactions.contains_key(&1));
            assert!(engine.transactions.contains_key(&2));
        }

        #[test]
        fn test_transaction_validation() {
            // Test that constructor methods enforce validation
            let result = std::panic::catch_unwind(|| {
                Transaction::new_deposit(1, 1, Decimal::from(-10));
            });
            assert!(result.is_err());

            let result = std::panic::catch_unwind(|| {
                Transaction::new_withdrawal(1, 1, Decimal::ZERO);
            });
            assert!(result.is_err());

            // Test that valid transactions are created correctly
            let deposit = Transaction::new_deposit(1, 1, Decimal::from(100));
            assert!(deposit.is_valid());
            assert_eq!(deposit.amount, Some(Decimal::from(100)));

            let dispute = Transaction::new_dispute(1, 1);
            assert!(dispute.is_valid());
            assert_eq!(dispute.amount, None);
        }

        #[test]
        fn test_dump_accounts_output() {
            use crate::engine::transaction::Transaction;
            use rust_decimal::Decimal;

            let mut engine = Engine::new();
            let tx1 = Transaction::new_deposit(1, 1, Decimal::from(100));
            let tx2 = Transaction::new_deposit(2, 2, Decimal::from(200));
            engine.apply_transaction(tx1).unwrap();
            engine.apply_transaction(tx2).unwrap();

            // Capture output in a buffer
            let mut buf = Vec::new();
            engine.dump_accounts(&mut buf);
            let output = String::from_utf8(buf).unwrap();

            // Check CSV header
            assert!(output.contains("client,available,held,total,locked"));
            // Check CSV data
            assert!(output.contains("1,100,0,100,false"));
            assert!(output.contains("2,200,0,200,false"));
        }

        #[test]
        fn test_dump_accounts_short() {
            use crate::engine::transaction::Transaction;
            use rust_decimal::Decimal;

            let mut engine = Engine::new();
            let tx = Transaction::new_deposit(1, 1, Decimal::from_str("42.0001").unwrap());
            engine.apply_transaction(tx).unwrap();

            let mut buf = Vec::new();
            engine.dump_accounts(&mut buf);
            let output = String::from_utf8(buf).unwrap();

            // Check CSV header
            assert!(output.contains("client,available,held,total,locked"));
            // Check CSV data
            assert!(output.contains("1,42.0001,0,42.0001,false"));
        }

        #[test]
        fn test_dump_accounts_long() {
            use crate::engine::transaction::Transaction;
            use rust_decimal::Decimal;

            let mut engine = Engine::new();
            for i in 1..=20 {
                let tx = Transaction::new_deposit(i, i as u32, Decimal::from(i * 10));
                engine.apply_transaction(tx).unwrap();
            }

            let mut buf = Vec::new();
            engine.dump_accounts(&mut buf);
            let output = String::from_utf8(buf).unwrap();

            // Check CSV header
            assert!(output.contains("client,available,held,total,locked"));

            // Check CSV data for each account
            for i in 1..=20 {
                let expected = format!("{},{},0,{},false", i, i * 10, i * 10);
                assert!(output.contains(&expected), "Missing: {}", expected);
            }
        }
    }
}
