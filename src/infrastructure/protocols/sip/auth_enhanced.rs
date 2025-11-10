/// Enhanced SIP authentication with SHA-256/SHA-512 and security features
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Supported digest algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestAlgorithm {
    MD5,
    Sha256,
    Sha512,
}

impl DigestAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            DigestAlgorithm::MD5 => "MD5",
            DigestAlgorithm::Sha256 => "SHA-256",
            DigestAlgorithm::Sha512 => "SHA-512",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "MD5" => Some(DigestAlgorithm::MD5),
            "SHA-256" => Some(DigestAlgorithm::Sha256),
            "SHA-512" => Some(DigestAlgorithm::Sha512),
            _ => None,
        }
    }
}

/// Enhanced digest authentication helper
pub struct EnhancedDigestAuth;

impl EnhancedDigestAuth {
    /// Calculate HA1 with specified algorithm
    pub fn calculate_ha1(
        username: &str,
        realm: &str,
        password: &str,
        algorithm: DigestAlgorithm,
    ) -> String {
        let data = format!("{}:{}:{}", username, realm, password);

        match algorithm {
            DigestAlgorithm::MD5 => {
                let digest = md5::compute(data);
                format!("{:x}", digest)
            }
            DigestAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            DigestAlgorithm::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        }
    }

    /// Calculate HA2
    pub fn calculate_ha2(method: &str, uri: &str, algorithm: DigestAlgorithm) -> String {
        let data = format!("{}:{}", method, uri);

        match algorithm {
            DigestAlgorithm::MD5 => {
                let digest = md5::compute(data);
                format!("{:x}", digest)
            }
            DigestAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            DigestAlgorithm::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        }
    }

    /// Calculate response digest
    pub fn calculate_response(
        ha1: &str,
        nonce: &str,
        ha2: &str,
        algorithm: DigestAlgorithm,
    ) -> String {
        let data = format!("{}:{}:{}", ha1, nonce, ha2);

        match algorithm {
            DigestAlgorithm::MD5 => {
                let digest = md5::compute(data);
                format!("{:x}", digest)
            }
            DigestAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            DigestAlgorithm::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        }
    }

    /// Calculate response digest with QoP
    pub fn calculate_response_qop(
        ha1: &str,
        nonce: &str,
        nc: &str,
        cnonce: &str,
        qop: &str,
        ha2: &str,
        algorithm: DigestAlgorithm,
    ) -> String {
        let data = format!("{}:{}:{}:{}:{}:{}", ha1, nonce, nc, cnonce, qop, ha2);

        match algorithm {
            DigestAlgorithm::MD5 => {
                let digest = md5::compute(data);
                format!("{:x}", digest)
            }
            DigestAlgorithm::Sha256 => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
            DigestAlgorithm::Sha512 => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(data.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        }
    }
}

/// Failed authentication attempt record
#[derive(Debug, Clone)]
struct FailedAttempt {
    timestamp: Instant,
    count: u32,
}

/// Brute force protection manager
pub struct BruteForceProtection {
    failed_attempts: Arc<RwLock<HashMap<String, FailedAttempt>>>,
    max_attempts: u32,
    lockout_duration: Duration,
    window_duration: Duration,
}

impl BruteForceProtection {
    /// Create new brute force protection manager
    pub fn new(max_attempts: u32, lockout_duration: Duration, window_duration: Duration) -> Self {
        Self {
            failed_attempts: Arc::new(RwLock::new(HashMap::new())),
            max_attempts,
            lockout_duration,
            window_duration,
        }
    }

    /// Create with default settings (5 attempts, 15 min lockout, 5 min window)
    pub fn default_settings() -> Self {
        Self::new(
            5,
            Duration::from_secs(15 * 60), // 15 minutes
            Duration::from_secs(5 * 60),   // 5 minutes
        )
    }

    /// Check if IP is currently locked out
    pub async fn is_locked_out(&self, ip: &str) -> bool {
        let attempts = self.failed_attempts.read().await;

        if let Some(attempt) = attempts.get(ip) {
            let elapsed = attempt.timestamp.elapsed();

            if attempt.count >= self.max_attempts {
                // Check if lockout period has expired
                if elapsed < self.lockout_duration {
                    warn!("IP {} is locked out ({} attempts)", ip, attempt.count);
                    return true;
                }
            }
        }

        false
    }

    /// Record a failed authentication attempt
    pub async fn record_failure(&self, ip: &str) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Instant::now();

        if let Some(attempt) = attempts.get_mut(ip) {
            let elapsed = attempt.timestamp.elapsed();

            if elapsed > self.window_duration {
                // Reset counter if window expired
                attempt.count = 1;
                attempt.timestamp = now;
                debug!("Reset failure count for IP {}", ip);
            } else {
                // Increment counter
                attempt.count += 1;
                attempt.timestamp = now;
                warn!("Failed auth attempt {} for IP {}", attempt.count, ip);

                if attempt.count >= self.max_attempts {
                    warn!(
                        "IP {} locked out after {} failed attempts",
                        ip, attempt.count
                    );
                }
            }
        } else {
            // First failure
            attempts.insert(
                ip.to_string(),
                FailedAttempt {
                    timestamp: now,
                    count: 1,
                },
            );
            debug!("First failed auth attempt for IP {}", ip);
        }
    }

    /// Record a successful authentication (resets counter)
    pub async fn record_success(&self, ip: &str) {
        let mut attempts = self.failed_attempts.write().await;
        attempts.remove(ip);
        debug!("Cleared failure record for IP {} after successful auth", ip);
    }

    /// Get failure count for an IP
    pub async fn get_failure_count(&self, ip: &str) -> u32 {
        let attempts = self.failed_attempts.read().await;
        attempts.get(ip).map(|a| a.count).unwrap_or(0)
    }

    /// Clean up expired entries
    pub async fn cleanup(&self) {
        let mut attempts = self.failed_attempts.write().await;
        let now = Instant::now();

        attempts.retain(|ip, attempt| {
            let elapsed = attempt.timestamp.elapsed();
            let should_keep = elapsed < self.lockout_duration;

            if !should_keep {
                debug!("Cleaned up expired entry for IP {}", ip);
            }

            should_keep
        });
    }

    /// Get total number of tracked IPs
    pub async fn count(&self) -> usize {
        let attempts = self.failed_attempts.read().await;
        attempts.len()
    }
}

/// Rate limiter for authentication requests
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    time_window: Duration,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(max_requests: usize, time_window: Duration) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            time_window,
        }
    }

    /// Create with default settings (10 requests per minute)
    pub fn default_settings() -> Self {
        Self::new(10, Duration::from_secs(60))
    }

    /// Check if request is allowed
    pub async fn is_allowed(&self, ip: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        // Get or create request history for this IP
        let history = requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        history.retain(|&timestamp| now.duration_since(timestamp) < self.time_window);

        // Check if under limit
        if history.len() < self.max_requests {
            history.push(now);
            true
        } else {
            warn!("Rate limit exceeded for IP {}", ip);
            false
        }
    }

    /// Clean up old entries
    pub async fn cleanup(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        requests.retain(|ip, history| {
            history.retain(|&timestamp| now.duration_since(timestamp) < self.time_window);
            let should_keep = !history.is_empty();

            if !should_keep {
                debug!("Cleaned up rate limiter entry for IP {}", ip);
            }

            should_keep
        });
    }

    /// Get request count for an IP
    pub async fn get_request_count(&self, ip: &str) -> usize {
        let requests = self.requests.read().await;
        requests.get(ip).map(|h| h.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_algorithms() {
        let username = "alice";
        let realm = "example.com";
        let password = "secret";

        // MD5
        let ha1_md5 = EnhancedDigestAuth::calculate_ha1(username, realm, password, DigestAlgorithm::MD5);
        assert_eq!(ha1_md5.len(), 32); // MD5 produces 32 hex characters

        // SHA-256
        let ha1_sha256 = EnhancedDigestAuth::calculate_ha1(username, realm, password, DigestAlgorithm::Sha256);
        assert_eq!(ha1_sha256.len(), 64); // SHA-256 produces 64 hex characters

        // SHA-512
        let ha1_sha512 = EnhancedDigestAuth::calculate_ha1(username, realm, password, DigestAlgorithm::Sha512);
        assert_eq!(ha1_sha512.len(), 128); // SHA-512 produces 128 hex characters

        // Verify algorithms are different
        assert_ne!(ha1_md5, ha1_sha256);
        assert_ne!(ha1_sha256, ha1_sha512);
    }

    #[tokio::test]
    async fn test_brute_force_protection() {
        let protection = BruteForceProtection::new(
            3,
            Duration::from_secs(60),
            Duration::from_secs(30),
        );

        let ip = "192.168.1.100";

        // First 2 attempts - should not be locked
        assert!(!protection.is_locked_out(ip).await);
        protection.record_failure(ip).await;
        assert!(!protection.is_locked_out(ip).await);
        protection.record_failure(ip).await;
        assert!(!protection.is_locked_out(ip).await);

        // 3rd attempt - should trigger lockout
        protection.record_failure(ip).await;
        assert!(protection.is_locked_out(ip).await);
        assert_eq!(protection.get_failure_count(ip).await, 3);

        // Successful auth should clear
        protection.record_success(ip).await;
        assert!(!protection.is_locked_out(ip).await);
        assert_eq!(protection.get_failure_count(ip).await, 0);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let ip = "192.168.1.100";

        // First 3 requests should be allowed
        assert!(limiter.is_allowed(ip).await);
        assert!(limiter.is_allowed(ip).await);
        assert!(limiter.is_allowed(ip).await);

        // 4th request should be denied
        assert!(!limiter.is_allowed(ip).await);

        assert_eq!(limiter.get_request_count(ip).await, 3);
    }

    #[test]
    fn test_algorithm_string_conversion() {
        assert_eq!(DigestAlgorithm::from_str("MD5"), Some(DigestAlgorithm::MD5));
        assert_eq!(DigestAlgorithm::from_str("SHA-256"), Some(DigestAlgorithm::Sha256));
        assert_eq!(DigestAlgorithm::from_str("sha-512"), Some(DigestAlgorithm::Sha512));
        assert_eq!(DigestAlgorithm::from_str("invalid"), None);

        assert_eq!(DigestAlgorithm::MD5.as_str(), "MD5");
        assert_eq!(DigestAlgorithm::Sha256.as_str(), "SHA-256");
        assert_eq!(DigestAlgorithm::Sha512.as_str(), "SHA-512");
    }

    #[test]
    fn test_ha2_calculation() {
        let method = "INVITE";
        let uri = "sip:bob@example.com";

        let ha2_md5 = EnhancedDigestAuth::calculate_ha2(method, uri, DigestAlgorithm::MD5);
        let ha2_sha256 = EnhancedDigestAuth::calculate_ha2(method, uri, DigestAlgorithm::Sha256);
        let ha2_sha512 = EnhancedDigestAuth::calculate_ha2(method, uri, DigestAlgorithm::Sha512);

        assert_eq!(ha2_md5.len(), 32);
        assert_eq!(ha2_sha256.len(), 64);
        assert_eq!(ha2_sha512.len(), 128);
    }
}
