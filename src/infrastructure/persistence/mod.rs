//! Persistence implementations

pub mod memory;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub mod database;
#[cfg(feature = "postgres")]
pub mod user_repository;
#[cfg(feature = "postgres")]
pub mod cdr_repository;

#[cfg(feature = "postgres")]
pub use database::{create_pool, run_migrations, DatabaseConfig};
#[cfg(feature = "postgres")]
pub use user_repository::PgUserRepository;
#[cfg(feature = "postgres")]
pub use cdr_repository::PgCdrRepository;
