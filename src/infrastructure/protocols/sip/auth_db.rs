//! Database-backed SIP Digest Authentication

use super::auth::{AuthChallenge, AuthorizationHeader, SipAuthenticator};
use super::message::{SipError, SipRequest};
use async_trait::async_trait;
use crate::domain::user::UserRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Digest authentication manager with database backend
pub struct DigestAuthDb {
    realm: String,
    user_repository: Arc<dyn UserRepository>,
    active_nonces: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

impl DigestAuthDb {
    /// Create a new digest authentication manager with database backend
    pub fn new(realm: String, user_repository: Arc<dyn UserRepository>) -> Self {
        Self {
            realm,
            user_repository,
            active_nonces: Arc::new(RwLock::new(HashMap::new())),
        }
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

        // Verify realm matches
        if auth.realm != self.realm {
            warn!("Realm mismatch: expected {}, got {}", self.realm, auth.realm);
            return Err(SipError::Authentication("Realm mismatch".to_string()));
        }

        // Get user from database
        let user = self
            .user_repository
            .find_by_username_and_realm(&auth.username, &auth.realm)
            .await
            .map_err(|e| {
                warn!("Database error while looking up user {}: {}", auth.username, e);
                SipError::Internal(format!("Database error: {}", e))
            })?
            .ok_or_else(|| {
                warn!("Authentication failed: unknown user {}", auth.username);
                SipError::Authentication(format!("Unknown user: {}", auth.username))
            })?;

        // Check if user is enabled
        if !user.is_enabled() {
            warn!("User {} is disabled", auth.username);
            return Err(SipError::Authentication("User is disabled".to_string()));
        }

        // Get SIP HA1 from user record
        let ha1 = user.sip_ha1.ok_or_else(|| {
            warn!("User {} has no SIP HA1 hash stored", auth.username);
            SipError::Internal("SIP HA1 not configured for user".to_string())
        })?;

        debug!("Using stored SIP HA1 for user {}", auth.username);

        let expected_response = Self::calculate_response_from_ha1(
            &ha1,
            &auth.nonce,
            method,
            &auth.uri,
            auth.qop.as_deref(),
            auth.nc.as_deref(),
            auth.cnonce.as_deref(),
        );

        // Verify response
        if auth.response != expected_response {
            warn!(
                "Authentication failed for user {}: response mismatch",
                auth.username
            );
            return Err(SipError::Authentication("Invalid credentials".to_string()));
        }

        info!("Authentication successful for user: {}", auth.username);
        Ok(auth.username)
    }

    /// Calculate digest response from HA1
    fn calculate_response_from_ha1(
        ha1: &str,
        nonce: &str,
        method: &str,
        uri: &str,
        qop: Option<&str>,
        nc: Option<&str>,
        cnonce: Option<&str>,
    ) -> String {
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

        debug!("Calculated response from HA1");
        response
    }

    /// Clean up old nonces
    pub async fn cleanup_nonces(&self) {
        let mut nonces = self.active_nonces.write().await;
        nonces.retain(|_, time| time.elapsed().as_secs() < 300);
    }
}

#[async_trait]
impl SipAuthenticator for DigestAuthDb {
    async fn create_challenge(&self) -> AuthChallenge {
        self.create_challenge().await
    }

    async fn verify_request(&self, request: &SipRequest, method: &str) -> Result<String, SipError> {
        self.verify_request(request, method).await
    }
}
