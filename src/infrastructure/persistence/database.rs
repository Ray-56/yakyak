//! Database connection management

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/yakyak".to_string(),
            max_connections: 10,
            min_connections: 2,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),      // 10 minutes
            max_lifetime: Duration::from_secs(1800),     // 30 minutes
        }
    }
}

impl DatabaseConfig {
    /// Create from environment variable or use default
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/yakyak".to_string());

        Self {
            url,
            ..Default::default()
        }
    }
}

/// Create a new database connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    info!("Creating database connection pool");
    info!("Database URL: {}", mask_password(&config.url));
    info!("Max connections: {}", config.max_connections);
    info!("Min connections: {}", config.min_connections);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .max_lifetime(Some(config.max_lifetime))
        .connect(&config.url)
        .await?;

    info!("Database connection pool created successfully");

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running database migrations");

    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;

    info!("Database migrations completed successfully");
    Ok(())
}

/// Mask password in database URL for logging
fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(colon_pos) = url[..at_pos].rfind(':') {
            let mut masked = url.to_string();
            masked.replace_range(colon_pos + 1..at_pos, "****");
            return masked;
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let url = "postgres://user:password@localhost/db";
        let masked = mask_password(url);
        assert_eq!(masked, "postgres://user:****@localhost/db");
    }

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }
}
