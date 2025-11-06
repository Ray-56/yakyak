//! Call bounded context - manages the lifecycle of calls

pub mod aggregate;
pub mod entity;
pub mod event;
pub mod repository;
pub mod service;
pub mod value_object;

pub use aggregate::Call;
pub use entity::Participant;
pub use event::CallEvent;
pub use repository::CallRepository;
pub use service::CallDomainService;
pub use value_object::{CallDirection, CallState, EndReason};
