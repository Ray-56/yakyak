/// TLS configuration for SIP and media encryption
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TLS operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsMode {
    /// TLS is disabled
    Disabled,
    /// TLS is optional (opportunistic)
    Optional,
    /// TLS is required for all connections
    Required,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// TLS mode
    pub mode: TlsMode,

    /// Path to certificate file (PEM format)
    pub certificate_path: Option<PathBuf>,

    /// Path to private key file (PEM format)
    pub private_key_path: Option<PathBuf>,

    /// Path to CA certificate bundle for verification
    pub ca_bundle_path: Option<PathBuf>,

    /// Verify peer certificates
    pub verify_peer: bool,

    /// Verify peer hostname
    pub verify_hostname: bool,

    /// Minimum TLS version (e.g., "1.2", "1.3")
    pub min_version: String,

    /// Maximum TLS version
    pub max_version: Option<String>,

    /// Allowed cipher suites
    pub cipher_suites: Vec<String>,

    /// ALPN protocols (for HTTP/2, etc.)
    pub alpn_protocols: Vec<String>,

    /// Client certificate required
    pub require_client_cert: bool,

    /// Session timeout in seconds
    pub session_timeout: u64,

    /// Enable session resumption
    pub session_resumption: bool,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            mode: TlsMode::Optional,
            certificate_path: None,
            private_key_path: None,
            ca_bundle_path: None,
            verify_peer: true,
            verify_hostname: true,
            min_version: "1.2".to_string(),
            max_version: None,
            cipher_suites: Self::default_cipher_suites(),
            alpn_protocols: vec![],
            require_client_cert: false,
            session_timeout: 3600,
            session_resumption: true,
        }
    }
}

impl TlsConfig {
    /// Create disabled TLS config
    pub fn disabled() -> Self {
        Self {
            mode: TlsMode::Disabled,
            ..Default::default()
        }
    }

    /// Create required TLS config
    pub fn required() -> Self {
        Self {
            mode: TlsMode::Required,
            ..Default::default()
        }
    }

    /// Set certificate and key paths
    pub fn with_certificate(mut self, cert_path: PathBuf, key_path: PathBuf) -> Self {
        self.certificate_path = Some(cert_path);
        self.private_key_path = Some(key_path);
        self
    }

    /// Set CA bundle path
    pub fn with_ca_bundle(mut self, ca_path: PathBuf) -> Self {
        self.ca_bundle_path = Some(ca_path);
        self
    }

    /// Set minimum TLS version
    pub fn with_min_version(mut self, version: String) -> Self {
        self.min_version = version;
        self
    }

    /// Enable mutual TLS (mTLS)
    pub fn with_mutual_tls(mut self) -> Self {
        self.require_client_cert = true;
        self.verify_peer = true;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.mode == TlsMode::Disabled {
            return Ok(());
        }

        // Check certificate files exist if TLS is enabled
        if self.mode != TlsMode::Disabled {
            if self.certificate_path.is_none() {
                return Err("Certificate path required when TLS is enabled".to_string());
            }
            if self.private_key_path.is_none() {
                return Err("Private key path required when TLS is enabled".to_string());
            }

            // Check files exist
            if let Some(ref cert_path) = self.certificate_path {
                if !cert_path.exists() {
                    return Err(format!("Certificate file not found: {:?}", cert_path));
                }
            }

            if let Some(ref key_path) = self.private_key_path {
                if !key_path.exists() {
                    return Err(format!("Private key file not found: {:?}", key_path));
                }
            }
        }

        // Validate TLS version
        if !["1.0", "1.1", "1.2", "1.3"].contains(&self.min_version.as_str()) {
            return Err(format!("Invalid TLS version: {}", self.min_version));
        }

        Ok(())
    }

    /// Get default cipher suites (strong ciphers only)
    fn default_cipher_suites() -> Vec<String> {
        vec![
            // TLS 1.3 ciphers
            "TLS_AES_256_GCM_SHA384".to_string(),
            "TLS_CHACHA20_POLY1305_SHA256".to_string(),
            "TLS_AES_128_GCM_SHA256".to_string(),

            // TLS 1.2 ciphers
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256".to_string(),
        ]
    }
}

/// DTLS configuration for SRTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DtlsConfig {
    /// DTLS enabled
    pub enabled: bool,

    /// Certificate fingerprint algorithm
    pub fingerprint_algorithm: String,

    /// DTLS role (active, passive, actpass)
    pub role: DtlsRole,

    /// SRTP profiles
    pub srtp_profiles: Vec<String>,

    /// Retransmission timeout (milliseconds)
    pub retransmission_timeout: u64,

    /// Maximum retransmissions
    pub max_retransmissions: u32,
}

/// DTLS connection role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DtlsRole {
    /// Client (initiates DTLS handshake)
    Active,
    /// Server (waits for DTLS handshake)
    Passive,
    /// Can be either client or server
    Actpass,
}

impl DtlsRole {
    pub fn to_string(&self) -> &str {
        match self {
            DtlsRole::Active => "active",
            DtlsRole::Passive => "passive",
            DtlsRole::Actpass => "actpass",
        }
    }
}

impl Default for DtlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fingerprint_algorithm: "sha-256".to_string(),
            role: DtlsRole::Actpass,
            srtp_profiles: vec![
                "SRTP_AES128_CM_SHA1_80".to_string(),
                "SRTP_AES128_CM_SHA1_32".to_string(),
            ],
            retransmission_timeout: 1000,
            max_retransmissions: 5,
        }
    }
}

impl DtlsConfig {
    /// Create disabled DTLS config
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Set DTLS role
    pub fn with_role(mut self, role: DtlsRole) -> Self {
        self.role = role;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert_eq!(config.mode, TlsMode::Optional);
        assert!(config.verify_peer);
        assert_eq!(config.min_version, "1.2");
    }

    #[test]
    fn test_tls_config_disabled() {
        let config = TlsConfig::disabled();
        assert_eq!(config.mode, TlsMode::Disabled);
    }

    #[test]
    fn test_tls_config_required() {
        let config = TlsConfig::required();
        assert_eq!(config.mode, TlsMode::Required);
    }

    #[test]
    fn test_tls_config_with_certificate() {
        let config = TlsConfig::default()
            .with_certificate(
                PathBuf::from("/path/to/cert.pem"),
                PathBuf::from("/path/to/key.pem"),
            );

        assert!(config.certificate_path.is_some());
        assert!(config.private_key_path.is_some());
    }

    #[test]
    fn test_tls_config_mutual_tls() {
        let config = TlsConfig::default().with_mutual_tls();
        assert!(config.require_client_cert);
        assert!(config.verify_peer);
    }

    #[test]
    fn test_tls_config_validation() {
        let config = TlsConfig::disabled();
        assert!(config.validate().is_ok());

        let config = TlsConfig::required();
        assert!(config.validate().is_err()); // Missing certificate paths
    }

    #[test]
    fn test_dtls_config_default() {
        let config = DtlsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.fingerprint_algorithm, "sha-256");
        assert_eq!(config.role, DtlsRole::Actpass);
    }

    #[test]
    fn test_dtls_role_string() {
        assert_eq!(DtlsRole::Active.to_string(), "active");
        assert_eq!(DtlsRole::Passive.to_string(), "passive");
        assert_eq!(DtlsRole::Actpass.to_string(), "actpass");
    }

    #[test]
    fn test_dtls_config_with_role() {
        let config = DtlsConfig::default().with_role(DtlsRole::Active);
        assert_eq!(config.role, DtlsRole::Active);
    }
}
