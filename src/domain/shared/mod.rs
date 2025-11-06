//! Shared kernel - Common types and utilities used across all bounded contexts

pub mod error;
pub mod events;
pub mod result;
pub mod value_objects;

pub use error::DomainError;
pub use result::Result;
pub use value_objects::*;
