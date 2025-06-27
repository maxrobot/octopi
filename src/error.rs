use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Account locked: {0}")]
    AccountLocked(u16),

    #[error("Invalid transaction_id {0} is a duplicate")]
    DuplicateTransaction(u32),

    #[error("Invalid client {0} does not match referenced client {1}")]
    InvalidClient(u16, u16),

    #[error("Invalid dispute operation on withdrawal")]
    InvalidOperationOnWithdrawal,

    #[error("Invalid client {0} does not exist")]
    NonExistentClient(u16),

    #[error("Invalid transaction_id {0} does not exist")]
    NonExistentTransaction(u32),

    #[error("Invalid transaction_id {0} has zero amount")]
    ZeroAmount(u32),

    #[error("Invalid transaction: {message}")]
    InvalidTransaction { message: String },
}
