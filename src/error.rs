use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Account locked: {0}")]
    AccountLocked(u16),

    #[error("Invalid transaction: {message}")]
    InvalidTransaction { message: String },
}
