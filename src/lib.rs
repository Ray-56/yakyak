//! YakYak - A modern PBX system built with Rust
//!
//! This is a Domain-Driven Design (DDD) implementation of a PBX system
//! that supports SIP, WebRTC, and modern communication protocols.

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod interface;

// Re-export commonly used types
pub use domain::shared::error::DomainError;
pub use domain::shared::result::Result;
