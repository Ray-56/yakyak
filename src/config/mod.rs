//! Configuration management

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub sip: SipConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SipConfig {
    pub bind_address: String,
    pub bind_port: u16,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
            },
            sip: SipConfig {
                bind_address: "0.0.0.0".to_string(),
                bind_port: 5060,
                domain: "localhost".to_string(),
            },
            database: DatabaseConfig {
                url: "postgres://postgres@localhost/yakyak".to_string(),
            },
        }
    }
}
