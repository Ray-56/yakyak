//! Application layer - Use cases and application services
//!
//! This layer orchestrates domain objects to fulfill use cases.
//! It's responsible for:
//! - Transaction management
//! - Coordinating multiple aggregates
//! - Publishing domain events
//! - Converting between domain models and DTOs

pub mod call;
pub mod registration;
pub mod session;

// Placeholder modules
