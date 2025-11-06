//! Domain layer - Core business logic and rules
//!
//! This layer contains:
//! - Aggregates: Consistency boundaries
//! - Entities: Objects with identity
//! - Value Objects: Immutable objects without identity
//! - Domain Services: Operations that don't fit in a single aggregate
//! - Repository Interfaces: Ports for persistence
//! - Domain Events: Things that happened in the domain

pub mod billing;
pub mod call;
pub mod cdr;
pub mod media;
pub mod registration;
pub mod routing;
pub mod session;
pub mod shared;
pub mod user;
pub mod voicemail;

// Re-export commonly used types
pub use shared::{DomainError, Result};
