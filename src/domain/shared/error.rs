//! Domain errors

use thiserror::Error;

/// Domain result type
pub type Result<T> = std::result::Result<T, DomainError>;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
