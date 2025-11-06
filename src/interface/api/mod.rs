//! API interface implementations

pub mod calls_handler;
pub mod cdr_dto;
pub mod cdr_handler;
pub mod conference;
pub mod jsonrpc;
pub mod metrics_handler;
pub mod monitoring;
pub mod rest;
pub mod router;
pub mod user_dto;
pub mod user_handler;
pub mod user_import;
pub mod voicemail;
pub mod websocket;
pub mod ws_handler;

pub use conference::{conference_router, ConferenceApiState};
pub use metrics_handler::{init_metrics, update_active_calls, update_registered_users};
pub use monitoring::{MetricsCollector, SystemHealth};
pub use router::build_router;
pub use user_handler::AppState;
pub use user_import::{import_users_csv, import_users_json};
pub use voicemail::{voicemail_router, VoicemailApiState};
pub use websocket::EventBroadcaster;
pub use ws_handler::EventBroadcaster as LegacyEventBroadcaster;
