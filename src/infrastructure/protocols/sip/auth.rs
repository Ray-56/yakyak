//! SIP Digest Authentication (RFC 2617, RFC 3261)

use super::message::{SipError, SipRequest};
use async_trait::async_trait;
use rand::Rng;
use rsip::Header;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// SIP authenticator trait
#[async_trait]
pub trait SipAuthenticator: Send + Sync {
    /// Generate an authentication challenge
    async fn create_challenge(&self) -> AuthChallenge;

    /// Verify authentication for a request
    async fn verify_request(&self, request: &SipRequest, method: &str) -> Result<String, SipError>;
}

/// User credentials for authentication
#[derive(Debug, Clone)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
    pub realm: String,
}

/// Authentication challenge
#[derive(Debug, Clone)]
pub struct AuthChallenge {
    pub realm: String,
    pub nonce: String,
    pub algorithm: String,
    pub qop: Option<String>,
}

impl AuthChallenge {
    /// Create a new authentication challenge
    pub fn new(realm: &str) -> Self {
        Self {
            realm: realm.to_string(),
            nonce: Self::generate_nonce(),
            algorithm: "MD5".to_string(),
            qop: Some("auth".to_string()),
        }
    }

    /// Generate a random nonce
    fn generate_nonce() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        hex::encode(random_bytes)
    }

    /// Format as WWW-Authenticate header value
    pub fn to_header_value(&self) -> String {
        if let Some(qop) = &self.qop {
            format!(
                r#"Digest realm="{}", nonce="{}", algorithm={}, qop="{}""#,
                self.realm, self.nonce, self.algorithm, qop
            )
        } else {
            format!(
                r#"Digest realm="{}", nonce="{}", algorithm={}"#,
                self.realm, self.nonce, self.algorithm
            )
        }
    }
}

/// Parsed Authorization header
#[derive(Debug, Clone)]
pub struct AuthorizationHeader {
    pub username: String,
    pub realm: String,
    pub nonce: String,
    pub uri: String,
    pub response: String,
    pub algorithm: Option<String>,
    pub qop: Option<String>,
    pub nc: Option<String>,
    pub cnonce: Option<String>,
}

impl AuthorizationHeader {
    /// Parse Authorization header from request
    pub fn from_request(request: &SipRequest) -> Result<Self, SipError> {
        // Find Authorization or Proxy-Authorization header
        let auth_value = request
            .headers()
            .iter()
            .find_map(|h| match h {
                Header::Authorization(auth) => Some(auth.to_string()),
                Header::ProxyAuthorization(auth) => Some(auth.to_string()),
                _ => None,
            })
            .ok_or_else(|| SipError::Authentication("No Authorization header found".to_string()))?;

        debug!("Parsing Authorization header: {}", auth_value);

        // Parse the Digest parameters
        let params = Self::parse_digest_params(&auth_value)?;

        Ok(Self {
            username: params
                .get("username")
                .ok_or_else(|| {
                    SipError::Authentication("Missing username in Authorization".to_string())
                })?
                .to_string(),
            realm: params
                .get("realm")
                .ok_or_else(|| {
                    SipError::Authentication("Missing realm in Authorization".to_string())
                })?
                .to_string(),
            nonce: params
                .get("nonce")
                .ok_or_else(|| {
                    SipError::Authentication("Missing nonce in Authorization".to_string())
                })?
                .to_string(),
            uri: params
                .get("uri")
                .ok_or_else(|| {
                    SipError::Authentication("Missing uri in Authorization".to_string())
                })?
                .to_string(),
            response: params
                .get("response")
                .ok_or_else(|| {
                    SipError::Authentication("Missing response in Authorization".to_string())
                })?
                .to_string(),
            algorithm: params.get("algorithm").map(|s| s.to_string()),
            qop: params.get("qop").map(|s| s.to_string()),
            nc: params.get("nc").map(|s| s.to_string()),
            cnonce: params.get("cnonce").map(|s| s.to_string()),
        })
    }

    /// Parse Digest authentication parameters
    fn parse_digest_params(auth_value: &str) -> Result<HashMap<String, String>, SipError> {
        let mut params = HashMap::new();

        // Remove "Digest " prefix
        let digest_str = auth_value
            .strip_prefix("Digest ")
            .unwrap_or(auth_value)
            .trim();

        // Simple parser for key="value" pairs
        for part in digest_str.split(',') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');
                params.insert(key.to_string(), value.to_string());
            }
        }

        Ok(params)
    }
}

/// Digest authentication manager
pub struct DigestAuth {
    realm: String,
    users: Arc<RwLock<HashMap<String, UserCredentials>>>,
    active_nonces: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

impl DigestAuth {
    /// Create a new digest authentication manager
    pub fn new(realm: &str) -> Self {
        Self {
            realm: realm.to_string(),
            users: Arc::new(RwLock::new(HashMap::new())),
            active_nonces: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a user
    pub async fn add_user(&self, username: &str, password: &str) {
        let credentials = UserCredentials {
            username: username.to_string(),
            password: password.to_string(),
            realm: self.realm.clone(),
        };

        let mut users = self.users.write().await;
        users.insert(username.to_string(), credentials);
        info!("Added user: {}", username);
    }

    /// Generate an authentication challenge
    pub async fn create_challenge(&self) -> AuthChallenge {
        let challenge = AuthChallenge::new(&self.realm);

        // Store the nonce
        let mut nonces = self.active_nonces.write().await;
        nonces.insert(challenge.nonce.clone(), std::time::Instant::now());

        debug!("Created auth challenge with nonce: {}", challenge.nonce);
        challenge
    }

    /// Verify authentication
    pub async fn verify_request(
        &self,
        request: &SipRequest,
        method: &str,
    ) -> Result<String, SipError> {
        // Parse Authorization header
        let auth = AuthorizationHeader::from_request(request)?;

        // Verify nonce exists and is not too old
        {
            let nonces = self.active_nonces.read().await;
            let nonce_time = nonces.get(&auth.nonce).ok_or_else(|| {
                SipError::Authentication("Invalid or expired nonce".to_string())
            })?;

            // Check nonce age (5 minutes max)
            if nonce_time.elapsed().as_secs() > 300 {
                return Err(SipError::Authentication("Nonce expired".to_string()));
            }
        }

        // Get user credentials
        let users = self.users.read().await;
        let credentials = users.get(&auth.username).ok_or_else(|| {
            warn!("Authentication failed: unknown user {}", auth.username);
            SipError::Authentication(format!("Unknown user: {}", auth.username))
        })?;

        // Verify realm matches
        if auth.realm != self.realm {
            warn!("Realm mismatch: expected {}, got {}", self.realm, auth.realm);
            return Err(SipError::Authentication("Realm mismatch".to_string()));
        }

        // Calculate expected response
        let expected_response = self.calculate_response(
            &auth.username,
            &credentials.password,
            &auth.realm,
            &auth.nonce,
            method,
            &auth.uri,
            auth.qop.as_deref(),
            auth.nc.as_deref(),
            auth.cnonce.as_deref(),
        );

        // Verify response
        if auth.response != expected_response {
            warn!("Authentication failed for user {}: response mismatch", auth.username);
            return Err(SipError::Authentication("Invalid credentials".to_string()));
        }

        info!("Authentication successful for user: {}", auth.username);
        Ok(auth.username)
    }

    /// Calculate digest response
    fn calculate_response(
        &self,
        username: &str,
        password: &str,
        realm: &str,
        nonce: &str,
        method: &str,
        uri: &str,
        qop: Option<&str>,
        nc: Option<&str>,
        cnonce: Option<&str>,
    ) -> String {
        // HA1 = MD5(username:realm:password)
        let ha1 = {
            let digest = md5::compute(format!("{}:{}:{}", username, realm, password));
            format!("{:x}", digest)
        };

        // HA2 = MD5(method:uri)
        let ha2 = {
            let digest = md5::compute(format!("{}:{}", method, uri));
            format!("{:x}", digest)
        };

        // Response = MD5(HA1:nonce:HA2) or MD5(HA1:nonce:nc:cnonce:qop:HA2)
        let response = if let Some(qop_value) = qop {
            let nc_value = nc.unwrap_or("00000001");
            let cnonce_value = cnonce.unwrap_or("");
            let digest = md5::compute(format!(
                "{}:{}:{}:{}:{}:{}",
                ha1, nonce, nc_value, cnonce_value, qop_value, ha2
            ));
            format!("{:x}", digest)
        } else {
            let digest = md5::compute(format!("{}:{}:{}", ha1, nonce, ha2));
            format!("{:x}", digest)
        };

        debug!("Calculated response for user {}: {}", username, response);
        response
    }

    /// Clean up old nonces
    pub async fn cleanup_nonces(&self) {
        let mut nonces = self.active_nonces.write().await;
        nonces.retain(|_, time| time.elapsed().as_secs() < 300);
    }
}

#[async_trait]
impl SipAuthenticator for DigestAuth {
    async fn create_challenge(&self) -> AuthChallenge {
        self.create_challenge().await
    }

    async fn verify_request(&self, request: &SipRequest, method: &str) -> Result<String, SipError> {
        self.verify_request(request, method).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_challenge() {
        let auth = DigestAuth::new("test.com");
        let challenge = auth.create_challenge().await;

        assert_eq!(challenge.realm, "test.com");
        assert_eq!(challenge.algorithm, "MD5");
        assert!(!challenge.nonce.is_empty());
    }

    #[tokio::test]
    async fn test_add_user() {
        let auth = DigestAuth::new("test.com");
        auth.add_user("alice", "secret123").await;

        let users = auth.users.read().await;
        assert!(users.contains_key("alice"));
    }

    #[test]
    fn test_parse_digest_params() {
        let auth_value = r#"Digest username="alice", realm="test.com", nonce="abc123", uri="sip:bob@test.com", response="def456""#;
        let params = AuthorizationHeader::parse_digest_params(auth_value).unwrap();

        assert_eq!(params.get("username").unwrap(), "alice");
        assert_eq!(params.get("realm").unwrap(), "test.com");
        assert_eq!(params.get("nonce").unwrap(), "abc123");
    }

    #[test]
    fn test_calculate_response() {
        let auth = DigestAuth::new("test.com");
        let response = auth.calculate_response(
            "alice",
            "secret",
            "test.com",
            "dcd98b7102dd2f0e8b11d0f600bfb0c093",
            "REGISTER",
            "sip:test.com",
            None,
            None,
            None,
        );

        // Response should be a 32-character hex string
        assert_eq!(response.len(), 32);
    }
}
