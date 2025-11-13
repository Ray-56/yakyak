//! API interface implementations

// Temporarily disabled - under development
// pub mod call_queue;
pub mod calls_handler;
pub mod cdr_dto;
pub mod cdr_handler;
// pub mod conference;
pub mod conference_handler;
pub mod jsonrpc;
pub mod metrics_handler;
pub mod monitoring;
pub mod rest;
pub mod router;
// pub mod sip_trunk;
// pub mod tenant;
pub mod user_dto;
pub mod user_handler;
// pub mod user_import;
// pub mod voicemail;
// pub mod webrtc_signaling;
pub mod websocket;
pub mod ws_handler;

// pub use call_queue::{call_queue_router, CallQueueApiState};
// pub use conference::{conference_router, ConferenceApiState};
pub use metrics_handler::{init_metrics, update_active_calls, update_registered_users};
pub use monitoring::{MetricsCollector, SystemHealth};
pub use router::build_router;
// pub use sip_trunk::{sip_trunk_router, SipTrunkApiState};
// pub use tenant::{tenant_router, TenantApiState};
pub use user_handler::AppState;
// pub use user_import::{import_users_csv, import_users_json};
// pub use voicemail::{voicemail_router, VoicemailApiState};
// pub use webrtc_signaling::{webrtc_signaling_router, SignalingState};
pub use websocket::EventBroadcaster;
pub use ws_handler::EventBroadcaster as LegacyEventBroadcaster;
