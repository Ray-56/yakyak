//! Domain result type

use super::error::DomainError;

/// Standard result type for domain operations
pub type Result<T> = std::result::Result<T, DomainError>;
