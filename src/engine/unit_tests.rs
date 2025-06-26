use crate::engine::account::Account;
use crate::engine::engine::{chargeback, deposit, dispute, resolve, withdraw};
use crate::engine::transaction::Transaction;
use crate::error::EngineError;
use rust_decimal::Decimal;
use std::str::FromStr;

#[cfg(test)]
mod deposit_tests {
    use super::*;

    #[test]
    fn test_deposit_basic_functionality() {
        let mut account = Account::new(1);
        let result = deposit(&mut account, Decimal::from(100));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.total, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
        assert!(!account.locked);
    }

    #[test]
    fn test_deposit_zero_amount() {
        let mut account = Account::new(1);
        let result = deposit(&mut account, Decimal::ZERO);

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_negative_amount() {
        let mut account = Account::new(1);
        let result = deposit(&mut account, Decimal::from(-50));

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Total balance is negative");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }

    #[test]
    fn test_deposit_negative_amount_on_existing_balance() {
        let mut account = Account::new(1);
        account.available = Decimal::from(25);
        account.total = Decimal::from(25);

        let result = deposit(&mut account, Decimal::from(-50));

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Total balance is negative");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }

    #[test]
    fn test_deposit_existing_account() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.total = Decimal::from(50);

        let result = deposit(&mut account, Decimal::from(25));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(75));
        assert_eq!(account.total, Decimal::from(75));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_preserves_held_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.held = Decimal::from(50);
        account.total = Decimal::from(150);

        let result = deposit(&mut account, Decimal::from(25));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(125));
        assert_eq!(account.held, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(175));
    }

    #[test]
    fn test_deposit_preserves_locked_status() {
        let mut account = Account::new(1);
        account.locked = true;

        let result = deposit(&mut account, Decimal::from(100));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.total, Decimal::from(100));
        assert!(account.locked); // Should remain locked
    }

    #[test]
    fn test_deposit_large_amount() {
        let mut account = Account::new(1);
        let large_amount = Decimal::from(1_000_000_000);

        let result = deposit(&mut account, large_amount);

        assert!(result.is_ok());
        assert_eq!(account.available, large_amount);
        assert_eq!(account.total, large_amount);
    }

    #[test]
    fn test_deposit_decimal_amount() {
        let mut account = Account::new(1);
        let decimal_amount = Decimal::from_str("123.45").unwrap();

        let result = deposit(&mut account, decimal_amount);

        assert!(result.is_ok());
        assert_eq!(account.available, decimal_amount);
        assert_eq!(account.total, decimal_amount);
    }

    #[test]
    fn test_deposit_multiple_operations() {
        let mut account = Account::new(1);

        // First deposit
        let result1 = deposit(&mut account, Decimal::from(50));
        assert!(result1.is_ok());

        // Second deposit
        let result2 = deposit(&mut account, Decimal::from(75));
        assert!(result2.is_ok());

        // Third deposit
        let result3 = deposit(&mut account, Decimal::from(25));
        assert!(result3.is_ok());

        assert_eq!(account.available, Decimal::from(150));
        assert_eq!(account.total, Decimal::from(150));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_negative_amount_exactly_balances_existing() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = deposit(&mut account, Decimal::from(-100));

        assert!(result.is_ok()); // Should be exactly 0, not negative
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
    }

    #[test]
    fn test_deposit_negative_amount_slightly_less_than_balance() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = deposit(&mut account, Decimal::from_str("-99.99").unwrap());

        assert!(result.is_ok()); // Should still be positive
        assert_eq!(account.available, Decimal::from_str("0.01").unwrap());
        assert_eq!(account.total, Decimal::from_str("0.01").unwrap());
    }

    #[test]
    fn test_deposit_negative_amount_slightly_more_than_balance() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = deposit(&mut account, Decimal::from_str("-100.01").unwrap());

        assert!(result.is_err()); // Should be negative
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Total balance is negative");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }
}

#[cfg(test)]
mod withdraw_tests {
    use super::*;

    #[test]
    fn test_withdraw_basic_functionality() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = withdraw(&mut account, Decimal::from(50));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(50));
        assert_eq!(account.held, Decimal::ZERO);
    }

    #[test]
    fn test_withdraw_insufficient_funds() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.total = Decimal::from(50);

        let result = withdraw(&mut account, Decimal::from(100));

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Insufficient funds");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }

    #[test]
    fn test_withdraw_exact_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = withdraw(&mut account, Decimal::from(100));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
    }

    #[test]
    fn test_withdraw_zero_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let result = withdraw(&mut account, Decimal::ZERO);

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.total, Decimal::from(100));
    }

    #[test]
    fn test_withdraw_preserves_held_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.held = Decimal::from(50);
        account.total = Decimal::from(150);

        let result = withdraw(&mut account, Decimal::from(25));

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(75));
        assert_eq!(account.held, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(125));
    }
}

#[cfg(test)]
mod dispute_tests {
    use super::*;

    #[test]
    fn test_dispute_basic_functionality() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let tx = Transaction::new_deposit(1, 1, Decimal::from(50));

        let result = dispute(&mut account, &tx);

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.held, Decimal::from(50));
        assert_eq!(account.total, Decimal::from(100));
    }

    #[test]
    fn test_dispute_insufficient_funds() {
        let mut account = Account::new(1);
        account.available = Decimal::from(25);
        account.total = Decimal::from(25);

        let tx = Transaction::new_deposit(1, 1, Decimal::from(50));

        let result = dispute(&mut account, &tx);

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Insufficient funds");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }

    #[test]
    fn test_dispute_transaction_without_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(100);
        account.total = Decimal::from(100);

        let tx = Transaction::new_dispute(1, 1, 1);

        let result = dispute(&mut account, &tx);

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Transaction has no amount");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    #[test]
    fn test_resolve_basic_functionality() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.held = Decimal::from(50);
        account.total = Decimal::from(100);

        let tx = Transaction::new_deposit(1, 1, Decimal::from(50));

        let result = resolve(&mut account, &tx);

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(100));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::from(100));
    }

    #[test]
    fn test_resolve_transaction_without_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.held = Decimal::from(50);
        account.total = Decimal::from(100);

        let tx = Transaction::new_resolve(1, 1, 1);

        let result = resolve(&mut account, &tx);

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Transaction has no amount");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }
}

#[cfg(test)]
mod chargeback_tests {
    use super::*;

    #[test]
    fn test_chargeback_basic_functionality() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.held = Decimal::from(50);
        account.total = Decimal::from(100);

        let tx = Transaction::new_deposit(1, 1, Decimal::from(50));

        let result = chargeback(&mut account, &tx);

        assert!(result.is_ok());
        assert_eq!(account.available, Decimal::from(50));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::from(50));
        assert!(account.locked);
    }

    #[test]
    fn test_chargeback_transaction_without_amount() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.held = Decimal::from(50);
        account.total = Decimal::from(100);

        let tx = Transaction::new_chargeback(1, 1, 1);

        let result = chargeback(&mut account, &tx);

        assert!(result.is_err());
        match result {
            Err(EngineError::InvalidTransaction { message }) => {
                assert_eq!(message, "Transaction has no amount");
            }
            _ => panic!("Expected InvalidTransaction error"),
        }
    }

    #[test]
    fn test_chargeback_locks_account() {
        let mut account = Account::new(1);
        account.available = Decimal::from(50);
        account.held = Decimal::from(50);
        account.total = Decimal::from(100);
        account.locked = false;

        let tx = Transaction::new_deposit(1, 1, Decimal::from(50));

        let result = chargeback(&mut account, &tx);

        assert!(result.is_ok());
        assert!(account.locked);
    }
}
