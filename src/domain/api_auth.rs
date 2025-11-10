use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// API authentication token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    /// Access token for API requests
    Access,
    /// Refresh token for obtaining new access tokens
    Refresh,
    /// API key for service-to-service communication
    ApiKey,
}

/// JWT token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenClaims {
    /// Subject (user ID)
    pub sub: Uuid,
    /// Username
    pub username: String,
    /// Role ID
    pub role_id: Option<Uuid>,
    /// Token type
    pub token_type: TokenType,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issuer
    pub iss: String,
    /// Additional scopes/permissions
    pub scopes: Vec<String>,
}

impl TokenClaims {
    pub fn new(
        user_id: Uuid,
        username: String,
        role_id: Option<Uuid>,
        token_type: TokenType,
        expiry_seconds: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id,
            username,
            role_id,
            token_type,
            iat: now.timestamp(),
            exp: (now + Duration::seconds(expiry_seconds)).timestamp(),
            iss: "yakyak-pbx".to_string(),
            scopes: Vec::new(),
        }
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    pub fn expires_in(&self) -> i64 {
        self.exp - Utc::now().timestamp()
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }
}

/// API token (JWT representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: Option<String>,
}

impl ApiToken {
    pub fn new(access_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token: None,
            token_type: "Bearer".to_string(),
            expires_in,
            scope: None,
        }
    }

    pub fn with_refresh_token(mut self, refresh_token: String) -> Self {
        self.refresh_token = Some(refresh_token);
        self
    }

    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }
}

/// API key for service-to-service authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub name: String,
    pub description: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
}

impl ApiKey {
    pub fn new(name: String, scopes: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            key: Self::generate_key(),
            name,
            description: String::new(),
            scopes,
            enabled: true,
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            usage_count: 0,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    fn generate_key() -> String {
        // Generate a secure random API key (simplified)
        format!("yk_{}", Uuid::new_v4().to_string().replace("-", ""))
    }

    pub fn is_valid(&self) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return false;
            }
        }

        true
    }

    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    pub fn record_usage(&mut self) {
        self.usage_count += 1;
        self.last_used_at = Some(Utc::now());
    }
}

/// Authentication result
#[derive(Debug, Clone)]
pub enum AuthResult {
    Success(AuthContext),
    Failed(AuthError),
}

/// Authentication context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub username: String,
    pub role_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub auth_method: AuthMethod,
}

impl AuthContext {
    pub fn has_permission(&self, permission: &str) -> bool {
        // Check if user has specific permission
        // This would integrate with the RBAC system
        self.scopes.contains(&permission.to_string())
    }

    pub fn has_any_permission(&self, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[&str]) -> bool {
        permissions
            .iter()
            .all(|p| self.has_permission(p))
    }
}

/// Authentication method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    JwtToken,
    ApiKey,
    BasicAuth,
}

/// Authentication error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    InvalidToken,
    ExpiredToken,
    InvalidApiKey,
    InvalidCredentials,
    InsufficientPermissions,
    Unauthorized,
    RateLimitExceeded,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidToken => write!(f, "Invalid token"),
            AuthError::ExpiredToken => write!(f, "Token expired"),
            AuthError::InvalidApiKey => write!(f, "Invalid API key"),
            AuthError::InvalidCredentials => write!(f, "Invalid credentials"),
            AuthError::InsufficientPermissions => write!(f, "Insufficient permissions"),
            AuthError::Unauthorized => write!(f, "Unauthorized"),
            AuthError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
        }
    }
}

impl std::error::Error for AuthError {}

/// API authentication manager
pub struct ApiAuthManager {
    /// Secret key for signing JWT tokens
    secret_key: String,
    /// Access token expiry (seconds)
    access_token_expiry: i64,
    /// Refresh token expiry (seconds)
    refresh_token_expiry: i64,
    /// Active API keys
    api_keys: Arc<Mutex<HashMap<String, ApiKey>>>,
    /// Revoked tokens (blacklist)
    revoked_tokens: Arc<Mutex<Vec<String>>>,
    /// Rate limiting: token -> (request count, window start)
    rate_limits: Arc<Mutex<HashMap<String, (u32, DateTime<Utc>)>>>,
}

impl ApiAuthManager {
    pub fn new(secret_key: String) -> Self {
        Self {
            secret_key,
            access_token_expiry: 3600,      // 1 hour
            refresh_token_expiry: 2592000,  // 30 days
            api_keys: Arc::new(Mutex::new(HashMap::new())),
            revoked_tokens: Arc::new(Mutex::new(Vec::new())),
            rate_limits: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_token_expiry(mut self, access_expiry: i64, refresh_expiry: i64) -> Self {
        self.access_token_expiry = access_expiry;
        self.refresh_token_expiry = refresh_expiry;
        self
    }

    /// Generate JWT token (simplified - production should use jsonwebtoken crate)
    pub fn generate_token(
        &self,
        user_id: Uuid,
        username: String,
        role_id: Option<Uuid>,
        scopes: Vec<String>,
    ) -> Result<ApiToken, String> {
        let access_claims = TokenClaims::new(
            user_id,
            username.clone(),
            role_id,
            TokenType::Access,
            self.access_token_expiry,
        )
        .with_scopes(scopes.clone());

        let refresh_claims = TokenClaims::new(
            user_id,
            username,
            role_id,
            TokenType::Refresh,
            self.refresh_token_expiry,
        )
        .with_scopes(scopes);

        // Simplified token generation (production should use proper JWT library)
        let access_token = self.encode_token(&access_claims)?;
        let refresh_token = self.encode_token(&refresh_claims)?;

        Ok(ApiToken::new(access_token, self.access_token_expiry)
            .with_refresh_token(refresh_token))
    }

    /// Verify and decode JWT token
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, AuthError> {
        // Check if token is revoked
        if self.is_token_revoked(token) {
            return Err(AuthError::InvalidToken);
        }

        // Decode and verify token (simplified)
        let claims = self.decode_token(token)
            .map_err(|_| AuthError::InvalidToken)?;

        if claims.is_expired() {
            return Err(AuthError::ExpiredToken);
        }

        Ok(claims)
    }

    /// Refresh access token using refresh token
    pub fn refresh_token(&self, refresh_token: &str) -> Result<ApiToken, AuthError> {
        let claims = self.verify_token(refresh_token)?;

        if claims.token_type != TokenType::Refresh {
            return Err(AuthError::InvalidToken);
        }

        // Generate new access token
        let access_claims = TokenClaims::new(
            claims.sub,
            claims.username,
            claims.role_id,
            TokenType::Access,
            self.access_token_expiry,
        )
        .with_scopes(claims.scopes);

        let access_token = self.encode_token(&access_claims)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(ApiToken::new(access_token, self.access_token_expiry))
    }

    /// Revoke a token
    pub fn revoke_token(&self, token: &str) {
        self.revoked_tokens.lock().unwrap().push(token.to_string());
    }

    fn is_token_revoked(&self, token: &str) -> bool {
        self.revoked_tokens.lock().unwrap().contains(&token.to_string())
    }

    /// Create an API key
    pub fn create_api_key(&self, name: String, scopes: Vec<String>) -> ApiKey {
        let api_key = ApiKey::new(name, scopes);
        self.api_keys.lock().unwrap().insert(api_key.key.clone(), api_key.clone());
        api_key
    }

    /// Verify API key
    pub fn verify_api_key(&self, key: &str) -> Result<ApiKey, AuthError> {
        let mut api_keys = self.api_keys.lock().unwrap();

        if let Some(api_key) = api_keys.get_mut(key) {
            if !api_key.is_valid() {
                return Err(AuthError::InvalidApiKey);
            }

            api_key.record_usage();
            Ok(api_key.clone())
        } else {
            Err(AuthError::InvalidApiKey)
        }
    }

    /// Revoke an API key
    pub fn revoke_api_key(&self, key: &str) -> bool {
        if let Some(api_key) = self.api_keys.lock().unwrap().get_mut(key) {
            api_key.enabled = false;
            true
        } else {
            false
        }
    }

    /// Check rate limit
    pub fn check_rate_limit(&self, identifier: &str, max_requests: u32, window_seconds: i64) -> bool {
        let mut rate_limits = self.rate_limits.lock().unwrap();
        let now = Utc::now();

        if let Some((count, window_start)) = rate_limits.get_mut(identifier) {
            let elapsed = (now - *window_start).num_seconds();

            if elapsed > window_seconds {
                // Reset window
                *count = 1;
                *window_start = now;
                true
            } else if *count < max_requests {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            rate_limits.insert(identifier.to_string(), (1, now));
            true
        }
    }

    /// Authenticate request with JWT token
    pub fn authenticate_token(&self, token: &str) -> AuthResult {
        match self.verify_token(token) {
            Ok(claims) => {
                let context = AuthContext {
                    user_id: claims.sub,
                    username: claims.username,
                    role_id: claims.role_id,
                    scopes: claims.scopes,
                    auth_method: AuthMethod::JwtToken,
                };
                AuthResult::Success(context)
            }
            Err(e) => AuthResult::Failed(e),
        }
    }

    /// Authenticate request with API key
    pub fn authenticate_api_key(&self, key: &str) -> AuthResult {
        match self.verify_api_key(key) {
            Ok(api_key) => {
                // Create a context for API key authentication
                let context = AuthContext {
                    user_id: api_key.id, // Use API key ID as user ID
                    username: api_key.name.clone(),
                    role_id: None,
                    scopes: api_key.scopes.clone(),
                    auth_method: AuthMethod::ApiKey,
                };
                AuthResult::Success(context)
            }
            Err(e) => AuthResult::Failed(e),
        }
    }

    // Simplified token encoding/decoding (production should use jsonwebtoken crate)
    fn encode_token(&self, claims: &TokenClaims) -> Result<String, String> {
        // In production: use jsonwebtoken::encode
        let payload = serde_json::to_string(claims)
            .map_err(|e| format!("Failed to serialize claims: {}", e))?;

        // Simplified: just base64 encode (NOT SECURE - use proper JWT in production!)
        Ok(format!("{}::{}", base64_encode(&payload), base64_encode(&self.secret_key)))
    }

    fn decode_token(&self, token: &str) -> Result<TokenClaims, String> {
        // In production: use jsonwebtoken::decode
        let parts: Vec<&str> = token.split("::").collect();
        if parts.len() != 2 {
            return Err("Invalid token format".to_string());
        }

        let payload = base64_decode(parts[0])?;
        let signature = base64_decode(parts[1])?;

        // Verify signature
        if signature != self.secret_key {
            return Err("Invalid signature".to_string());
        }

        serde_json::from_str(&payload)
            .map_err(|e| format!("Failed to deserialize claims: {}", e))
    }
}

// Helper functions (simplified - use proper base64 crate in production)
fn base64_encode(data: &str) -> String {
    // Simplified encoding
    data.as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn base64_decode(data: &str) -> Result<String, String> {
    // Simplified decoding
    let bytes: Result<Vec<u8>, _> = (0..data.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&data[i..i + 2], 16))
        .collect();

    bytes
        .map_err(|e| format!("Decode error: {}", e))
        .and_then(|b| String::from_utf8(b).map_err(|e| format!("UTF-8 error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_claims_creation() {
        let claims = TokenClaims::new(
            Uuid::new_v4(),
            "testuser".to_string(),
            Some(Uuid::new_v4()),
            TokenType::Access,
            3600,
        );

        assert_eq!(claims.username, "testuser");
        assert_eq!(claims.token_type, TokenType::Access);
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_token_claims_expiry() {
        let mut claims = TokenClaims::new(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            TokenType::Access,
            3600,
        );

        assert!(!claims.is_expired());
        assert!(claims.expires_in() > 3500);

        // Simulate expiry
        claims.exp = Utc::now().timestamp() - 100;
        assert!(claims.is_expired());
    }

    #[test]
    fn test_token_claims_scopes() {
        let claims = TokenClaims::new(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            TokenType::Access,
            3600,
        )
        .with_scopes(vec!["read".to_string(), "write".to_string()]);

        assert!(claims.has_scope("read"));
        assert!(claims.has_scope("write"));
        assert!(!claims.has_scope("admin"));
    }

    #[test]
    fn test_api_key_creation() {
        let api_key = ApiKey::new(
            "Test Key".to_string(),
            vec!["api:read".to_string(), "api:write".to_string()],
        );

        assert!(api_key.key.starts_with("yk_"));
        assert!(api_key.is_valid());
        assert!(api_key.has_scope("api:read"));
    }

    #[test]
    fn test_api_key_expiry() {
        let mut api_key = ApiKey::new("Test".to_string(), vec![])
            .with_expiry(Utc::now() - Duration::hours(1));

        assert!(!api_key.is_valid());

        api_key.expires_at = Some(Utc::now() + Duration::hours(1));
        assert!(api_key.is_valid());
    }

    #[test]
    fn test_api_key_usage() {
        let mut api_key = ApiKey::new("Test".to_string(), vec![]);
        assert_eq!(api_key.usage_count, 0);
        assert!(api_key.last_used_at.is_none());

        api_key.record_usage();
        assert_eq!(api_key.usage_count, 1);
        assert!(api_key.last_used_at.is_some());
    }

    #[test]
    fn test_auth_manager_token_generation() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        let token = manager.generate_token(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            vec!["read".to_string()],
        );

        assert!(token.is_ok());
        let token = token.unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(token.refresh_token.is_some());
    }

    #[test]
    fn test_auth_manager_token_verification() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        let user_id = Uuid::new_v4();
        let token = manager.generate_token(
            user_id,
            "testuser".to_string(),
            None,
            vec!["read".to_string()],
        ).unwrap();

        let result = manager.verify_token(&token.access_token);
        assert!(result.is_ok());

        let claims = result.unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_auth_manager_token_revocation() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        let token = manager.generate_token(
            Uuid::new_v4(),
            "testuser".to_string(),
            None,
            vec![],
        ).unwrap();

        assert!(manager.verify_token(&token.access_token).is_ok());

        manager.revoke_token(&token.access_token);
        assert!(manager.verify_token(&token.access_token).is_err());
    }

    #[test]
    fn test_auth_manager_api_key() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        let api_key = manager.create_api_key(
            "Test Service".to_string(),
            vec!["service:access".to_string()],
        );

        let result = manager.verify_api_key(&api_key.key);
        assert!(result.is_ok());

        let verified_key = result.unwrap();
        assert_eq!(verified_key.usage_count, 1);
    }

    #[test]
    fn test_auth_manager_api_key_revocation() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        let api_key = manager.create_api_key("Test".to_string(), vec![]);
        assert!(manager.verify_api_key(&api_key.key).is_ok());

        manager.revoke_api_key(&api_key.key);
        assert!(manager.verify_api_key(&api_key.key).is_err());
    }

    #[test]
    fn test_auth_context_permissions() {
        let context = AuthContext {
            user_id: Uuid::new_v4(),
            username: "testuser".to_string(),
            role_id: None,
            scopes: vec!["read".to_string(), "write".to_string()],
            auth_method: AuthMethod::JwtToken,
        };

        assert!(context.has_permission("read"));
        assert!(context.has_permission("write"));
        assert!(!context.has_permission("admin"));

        assert!(context.has_any_permission(&["read", "admin"]));
        assert!(context.has_all_permissions(&["read", "write"]));
        assert!(!context.has_all_permissions(&["read", "write", "admin"]));
    }

    #[test]
    fn test_rate_limiting() {
        let manager = ApiAuthManager::new("test-secret".to_string());

        // First 5 requests should succeed
        for _ in 0..5 {
            assert!(manager.check_rate_limit("test-user", 5, 60));
        }

        // 6th request should fail
        assert!(!manager.check_rate_limit("test-user", 5, 60));
    }
}
