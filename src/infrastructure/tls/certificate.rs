/// Certificate and private key management
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Certificate type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CertificateType {
    /// Server certificate
    Server,
    /// Client certificate
    Client,
    /// CA certificate
    CA,
}

/// X.509 Certificate
#[derive(Debug, Clone)]
pub struct Certificate {
    pub id: Uuid,
    pub cert_type: CertificateType,
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
    pub fingerprint_sha256: String,
    pub public_key_algorithm: String,
    pub signature_algorithm: String,
    pub pem_data: String,
    pub san_dns_names: Vec<String>,
    pub san_ip_addresses: Vec<String>,
}

impl Certificate {
    /// Create a new certificate record
    pub fn new(cert_type: CertificateType, pem_data: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            cert_type,
            subject: String::new(),
            issuer: String::new(),
            serial_number: String::new(),
            not_before: Utc::now(),
            not_after: Utc::now(),
            fingerprint_sha256: String::new(),
            public_key_algorithm: String::new(),
            signature_algorithm: String::new(),
            pem_data,
            san_dns_names: Vec::new(),
            san_ip_addresses: Vec::new(),
        }
    }

    /// Check if certificate is currently valid
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        now >= self.not_before && now <= self.not_after
    }

    /// Check if certificate is self-signed
    pub fn is_self_signed(&self) -> bool {
        self.subject == self.issuer
    }

    /// Days until expiration
    pub fn days_until_expiration(&self) -> i64 {
        (self.not_after - Utc::now()).num_days()
    }

    /// Check if certificate will expire soon (within days)
    pub fn is_expiring_soon(&self, days: i64) -> bool {
        self.days_until_expiration() <= days
    }

    /// Get certificate fingerprint for SDP
    pub fn get_dtls_fingerprint(&self) -> String {
        // Format: "sha-256 AA:BB:CC:..."
        let formatted = self.fingerprint_sha256
            .as_bytes()
            .chunks(2)
            .map(|chunk| String::from_utf8_lossy(chunk))
            .collect::<Vec<_>>()
            .join(":");

        format!("sha-256 {}", formatted.to_uppercase())
    }
}

/// Private key
#[derive(Debug, Clone)]
pub struct PrivateKey {
    pub id: Uuid,
    pub algorithm: String,
    pub key_size: u32,
    pub pem_data: String,
    pub certificate_id: Option<Uuid>,
}

impl PrivateKey {
    pub fn new(algorithm: String, key_size: u32, pem_data: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            algorithm,
            key_size,
            pem_data,
            certificate_id: None,
        }
    }
}

/// Certificate manager
pub struct CertificateManager {
    certificates: Vec<Certificate>,
    private_keys: Vec<PrivateKey>,
}

impl CertificateManager {
    pub fn new() -> Self {
        Self {
            certificates: Vec::new(),
            private_keys: Vec::new(),
        }
    }

    /// Load certificate from PEM file
    pub fn load_certificate(&mut self, path: &Path, cert_type: CertificateType) -> Result<Uuid, String> {
        let pem_data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read certificate: {}", e))?;

        // In production, would parse with openssl/rustls
        let cert = Certificate::new(cert_type, pem_data);
        let id = cert.id;

        self.certificates.push(cert);
        Ok(id)
    }

    /// Load private key from PEM file
    pub fn load_private_key(&mut self, path: &Path) -> Result<Uuid, String> {
        let pem_data = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read private key: {}", e))?;

        // In production, would parse and validate
        let key = PrivateKey::new("RSA".to_string(), 2048, pem_data);
        let id = key.id;

        self.private_keys.push(key);
        Ok(id)
    }

    /// Get certificate by ID
    pub fn get_certificate(&self, id: Uuid) -> Option<&Certificate> {
        self.certificates.iter().find(|c| c.id == id)
    }

    /// Get private key by ID
    pub fn get_private_key(&self, id: Uuid) -> Option<&PrivateKey> {
        self.private_keys.iter().find(|k| k.id == id)
    }

    /// Get server certificates
    pub fn get_server_certificates(&self) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|c| c.cert_type == CertificateType::Server)
            .collect()
    }

    /// Get CA certificates
    pub fn get_ca_certificates(&self) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|c| c.cert_type == CertificateType::CA)
            .collect()
    }

    /// Check for expiring certificates
    pub fn get_expiring_certificates(&self, days: i64) -> Vec<&Certificate> {
        self.certificates
            .iter()
            .filter(|c| c.is_valid() && c.is_expiring_soon(days))
            .collect()
    }

    /// Remove certificate
    pub fn remove_certificate(&mut self, id: Uuid) -> Option<Certificate> {
        self.certificates
            .iter()
            .position(|c| c.id == id)
            .map(|pos| self.certificates.remove(pos))
    }

    /// Generate self-signed certificate (placeholder)
    pub fn generate_self_signed(
        &mut self,
        subject: String,
        days_valid: u32,
    ) -> Result<(Uuid, Uuid), String> {
        // In production, would use openssl/rustls to generate
        // For now, create placeholders

        let cert = Certificate::new(CertificateType::Server, "PLACEHOLDER_PEM".to_string());
        let key = PrivateKey::new("RSA".to_string(), 2048, "PLACEHOLDER_KEY".to_string());

        let cert_id = cert.id;
        let key_id = key.id;

        self.certificates.push(cert);
        self.private_keys.push(key);

        Ok((cert_id, key_id))
    }
}

impl Default for CertificateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_creation() {
        let cert = Certificate::new(CertificateType::Server, "pem_data".to_string());
        assert_eq!(cert.cert_type, CertificateType::Server);
        assert_eq!(cert.pem_data, "pem_data");
    }

    #[test]
    fn test_certificate_validity() {
        let mut cert = Certificate::new(CertificateType::Server, "pem_data".to_string());

        // Set valid dates
        cert.not_before = Utc::now() - chrono::Duration::days(1);
        cert.not_after = Utc::now() + chrono::Duration::days(365);

        assert!(cert.is_valid());
        assert!(!cert.is_expiring_soon(400));
        assert!(cert.is_expiring_soon(300));
    }

    #[test]
    fn test_self_signed_check() {
        let mut cert = Certificate::new(CertificateType::Server, "pem_data".to_string());
        cert.subject = "CN=example.com".to_string();
        cert.issuer = "CN=example.com".to_string();

        assert!(cert.is_self_signed());

        cert.issuer = "CN=ca.example.com".to_string();
        assert!(!cert.is_self_signed());
    }

    #[test]
    fn test_days_until_expiration() {
        let mut cert = Certificate::new(CertificateType::Server, "pem_data".to_string());
        cert.not_after = Utc::now() + chrono::Duration::days(30);

        let days = cert.days_until_expiration();
        assert!(days >= 29 && days <= 30);
    }

    #[test]
    fn test_private_key_creation() {
        let key = PrivateKey::new("RSA".to_string(), 2048, "key_data".to_string());
        assert_eq!(key.algorithm, "RSA");
        assert_eq!(key.key_size, 2048);
    }

    #[test]
    fn test_certificate_manager() {
        let mut manager = CertificateManager::new();

        let cert = Certificate::new(CertificateType::Server, "pem_data".to_string());
        let cert_id = cert.id;
        manager.certificates.push(cert);

        assert!(manager.get_certificate(cert_id).is_some());
        assert_eq!(manager.get_server_certificates().len(), 1);
    }

    #[test]
    fn test_expiring_certificates() {
        let mut manager = CertificateManager::new();

        let mut cert1 = Certificate::new(CertificateType::Server, "pem_data".to_string());
        cert1.not_before = Utc::now();
        cert1.not_after = Utc::now() + chrono::Duration::days(5);

        let mut cert2 = Certificate::new(CertificateType::Server, "pem_data2".to_string());
        cert2.not_before = Utc::now();
        cert2.not_after = Utc::now() + chrono::Duration::days(400);

        manager.certificates.push(cert1);
        manager.certificates.push(cert2);

        let expiring = manager.get_expiring_certificates(30);
        assert_eq!(expiring.len(), 1);
    }

    #[test]
    fn test_generate_self_signed() {
        let mut manager = CertificateManager::new();
        let result = manager.generate_self_signed("CN=test.local".to_string(), 365);

        assert!(result.is_ok());
        let (cert_id, key_id) = result.unwrap();

        assert!(manager.get_certificate(cert_id).is_some());
        assert!(manager.get_private_key(key_id).is_some());
    }
}
