//! Shared value objects used across multiple bounded contexts

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Call identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CallId(Uuid);

impl CallId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CallId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Endpoint identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EndpointId(Uuid);

impl EndpointId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for EndpointId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SIP URI value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SipUri {
    user: String,
    host: String,
    port: Option<u16>,
}

impl SipUri {
    pub fn new(user: String, host: String, port: Option<u16>) -> Self {
        Self { user, host, port }
    }

    pub fn parse(uri: &str) -> Result<Self, String> {
        // Simple parsing for now, can be enhanced
        if !uri.starts_with("sip:") {
            return Err("URI must start with 'sip:'".to_string());
        }

        let uri = &uri[4..]; // Remove "sip:" prefix
        let parts: Vec<&str> = uri.split('@').collect();

        if parts.len() != 2 {
            return Err("Invalid SIP URI format".to_string());
        }

        let user = parts[0].to_string();
        let host_port: Vec<&str> = parts[1].split(':').collect();
        let host = host_port[0].to_string();
        let port = if host_port.len() > 1 {
            host_port[1].parse().ok()
        } else {
            None
        };

        Ok(Self { user, host, port })
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }
}

impl fmt::Display for SipUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(port) = self.port {
            write!(f, "sip:{}@{}:{}", self.user, self.host, port)
        } else {
            write!(f, "sip:{}@{}", self.user, self.host)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sip_uri_parse() {
        let uri = SipUri::parse("sip:alice@example.com").unwrap();
        assert_eq!(uri.user(), "alice");
        assert_eq!(uri.host(), "example.com");
        assert_eq!(uri.port(), None);

        let uri_with_port = SipUri::parse("sip:bob@example.com:5060").unwrap();
        assert_eq!(uri_with_port.user(), "bob");
        assert_eq!(uri_with_port.host(), "example.com");
        assert_eq!(uri_with_port.port(), Some(5060));
    }

    #[test]
    fn test_sip_uri_display() {
        let uri = SipUri::new("alice".to_string(), "example.com".to_string(), None);
        assert_eq!(uri.to_string(), "sip:alice@example.com");

        let uri_with_port = SipUri::new("bob".to_string(), "example.com".to_string(), Some(5060));
        assert_eq!(uri_with_port.to_string(), "sip:bob@example.com:5060");
    }
}
