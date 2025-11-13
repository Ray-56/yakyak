//! Domain layer - Core business logic and rules
//!
//! This layer contains:
//! - Aggregates: Consistency boundaries
//! - Entities: Objects with identity
//! - Value Objects: Immutable objects without identity
//! - Domain Services: Operations that don't fit in a single aggregate
//! - Repository Interfaces: Ports for persistence
//! - Domain Events: Things that happened in the domain

pub mod api_auth;
pub mod audio;
pub mod billing;
pub mod call;
pub mod call_announcer;
pub mod call_forwarding;
pub mod call_manager;
pub mod call_parking;
pub mod call_pickup;
pub mod call_quality;
pub mod call_queue;
pub mod call_queue_engine;
pub mod call_recording;
pub mod cdr;
pub mod conference;
pub mod conference_manager;
pub mod conference_recording;
pub mod dnd;
pub mod instant_messaging;
pub mod ip_blacklist;
pub mod media;
pub mod music_on_hold;
pub mod mwi;
pub mod presence;
pub mod registration;
pub mod routing;
pub mod security;
pub mod session;
pub mod shared;
pub mod sip_trunk;
pub mod tenant;
pub mod user;
pub mod voicemail;
pub mod voicemail_ivr;
pub mod voicemail_service;

// Re-export commonly used types
pub use shared::{DomainError, Result};
