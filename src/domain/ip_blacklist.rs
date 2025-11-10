use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// IP blacklist entry reason
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlacklistReason {
    /// Manual block by administrator
    Manual,
    /// Too many authentication failures
    AuthenticationFailures,
    /// Too many requests (rate limit exceeded)
    RateLimitExceeded,
    /// Suspicious activity detected
    SuspiciousActivity,
    /// Malformed requests
    MalformedRequests,
    /// Port scanning detected
    PortScanning,
    /// Brute force attack
    BruteForce,
    /// Custom reason
    Custom(String),
}

impl std::fmt::Display for BlacklistReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlacklistReason::Manual => write!(f, "Manual block"),
            BlacklistReason::AuthenticationFailures => write!(f, "Too many auth failures"),
            BlacklistReason::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            BlacklistReason::SuspiciousActivity => write!(f, "Suspicious activity"),
            BlacklistReason::MalformedRequests => write!(f, "Malformed requests"),
            BlacklistReason::PortScanning => write!(f, "Port scanning"),
            BlacklistReason::BruteForce => write!(f, "Brute force attack"),
            BlacklistReason::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// IP blacklist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistEntry {
    pub id: Uuid,
    pub ip_address: IpAddr,
    pub reason: BlacklistReason,
    pub description: String,
    pub added_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub added_by: Option<String>,
}

impl BlacklistEntry {
    pub fn new(ip_address: IpAddr, reason: BlacklistReason) -> Self {
        Self {
            id: Uuid::new_v4(),
            ip_address,
            reason,
            description: String::new(),
            added_at: Utc::now(),
            expires_at: None,
            added_by: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_expiry(mut self, duration: Duration) -> Self {
        self.expires_at = Some(Utc::now() + duration);
        self
    }

    pub fn with_added_by(mut self, added_by: String) -> Self {
        self.added_by = Some(added_by);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn is_permanent(&self) -> bool {
        self.expires_at.is_none()
    }
}

/// IP whitelist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub id: Uuid,
    pub ip_address: IpAddr,
    pub description: String,
    pub added_at: DateTime<Utc>,
    pub added_by: Option<String>,
}

impl WhitelistEntry {
    pub fn new(ip_address: IpAddr) -> Self {
        Self {
            id: Uuid::new_v4(),
            ip_address,
            description: String::new(),
            added_at: Utc::now(),
            added_by: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    pub fn with_added_by(mut self, added_by: String) -> Self {
        self.added_by = Some(added_by);
        self
    }
}

/// Request tracking for rate limiting
#[derive(Debug, Clone)]
struct RequestTracker {
    timestamps: VecDeque<DateTime<Utc>>,
    window_seconds: i64,
    max_requests: usize,
}

impl RequestTracker {
    fn new(window_seconds: i64, max_requests: usize) -> Self {
        Self {
            timestamps: VecDeque::new(),
            window_seconds,
            max_requests,
        }
    }

    fn record_request(&mut self) {
        let now = Utc::now();
        self.timestamps.push_back(now);
        self.cleanup_old_requests();
    }

    fn cleanup_old_requests(&mut self) {
        let cutoff = Utc::now() - Duration::seconds(self.window_seconds);
        while let Some(&timestamp) = self.timestamps.front() {
            if timestamp < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    fn is_rate_limited(&self) -> bool {
        self.timestamps.len() >= self.max_requests
    }

    fn request_count(&self) -> usize {
        self.timestamps.len()
    }
}

/// Authentication failure tracking
#[derive(Debug, Clone)]
struct FailureTracker {
    failures: VecDeque<DateTime<Utc>>,
    window_seconds: i64,
}

impl FailureTracker {
    fn new(window_seconds: i64) -> Self {
        Self {
            failures: VecDeque::new(),
            window_seconds,
        }
    }

    fn record_failure(&mut self) {
        let now = Utc::now();
        self.failures.push_back(now);
        self.cleanup_old_failures();
    }

    fn cleanup_old_failures(&mut self) {
        let cutoff = Utc::now() - Duration::seconds(self.window_seconds);
        while let Some(&timestamp) = self.failures.front() {
            if timestamp < cutoff {
                self.failures.pop_front();
            } else {
                break;
            }
        }
    }

    fn failure_count(&self) -> usize {
        self.failures.len()
    }

    fn reset(&mut self) {
        self.failures.clear();
    }
}

/// IP blacklist and rate limiting manager
pub struct IpBlacklistManager {
    blacklist: Arc<Mutex<HashMap<IpAddr, BlacklistEntry>>>,
    whitelist: Arc<Mutex<HashMap<IpAddr, WhitelistEntry>>>,
    request_trackers: Arc<Mutex<HashMap<IpAddr, RequestTracker>>>,
    failure_trackers: Arc<Mutex<HashMap<IpAddr, FailureTracker>>>,
    config: BlacklistConfig,
}

/// Configuration for blacklist manager
#[derive(Debug, Clone)]
pub struct BlacklistConfig {
    /// Rate limit: max requests per window
    pub rate_limit_max_requests: usize,
    /// Rate limit: window in seconds
    pub rate_limit_window_seconds: i64,
    /// Auth failures: max failures before auto-block
    pub max_auth_failures: usize,
    /// Auth failures: tracking window in seconds
    pub auth_failure_window_seconds: i64,
    /// Auto-block duration for rate limit violations
    pub rate_limit_block_duration: Duration,
    /// Auto-block duration for auth failures
    pub auth_failure_block_duration: Duration,
    /// Enable auto-blocking
    pub auto_block_enabled: bool,
}

impl Default for BlacklistConfig {
    fn default() -> Self {
        Self {
            rate_limit_max_requests: 100,
            rate_limit_window_seconds: 60,
            max_auth_failures: 5,
            auth_failure_window_seconds: 300,
            rate_limit_block_duration: Duration::hours(1),
            auth_failure_block_duration: Duration::hours(24),
            auto_block_enabled: true,
        }
    }
}

impl IpBlacklistManager {
    pub fn new(config: BlacklistConfig) -> Self {
        Self {
            blacklist: Arc::new(Mutex::new(HashMap::new())),
            whitelist: Arc::new(Mutex::new(HashMap::new())),
            request_trackers: Arc::new(Mutex::new(HashMap::new())),
            failure_trackers: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Check if an IP is blocked
    pub fn is_blocked(&self, ip: &IpAddr) -> bool {
        // Whitelist takes precedence
        if self.whitelist.lock().unwrap().contains_key(ip) {
            return false;
        }

        if let Some(entry) = self.blacklist.lock().unwrap().get(ip) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Add IP to blacklist
    pub fn block_ip(&self, entry: BlacklistEntry) -> Uuid {
        let id = entry.id;
        let ip = entry.ip_address;
        self.blacklist.lock().unwrap().insert(ip, entry);
        id
    }

    /// Remove IP from blacklist
    pub fn unblock_ip(&self, ip: &IpAddr) -> bool {
        self.blacklist.lock().unwrap().remove(ip).is_some()
    }

    /// Add IP to whitelist
    pub fn whitelist_ip(&self, entry: WhitelistEntry) -> Uuid {
        let id = entry.id;
        let ip = entry.ip_address;
        self.whitelist.lock().unwrap().insert(ip, entry);
        id
    }

    /// Remove IP from whitelist
    pub fn remove_from_whitelist(&self, ip: &IpAddr) -> bool {
        self.whitelist.lock().unwrap().remove(ip).is_some()
    }

    /// Check if IP is whitelisted
    pub fn is_whitelisted(&self, ip: &IpAddr) -> bool {
        self.whitelist.lock().unwrap().contains_key(ip)
    }

    /// Record a request from an IP and check rate limit
    pub fn check_rate_limit(&self, ip: &IpAddr) -> Result<(), String> {
        // Skip rate limiting for whitelisted IPs
        if self.is_whitelisted(ip) {
            return Ok(());
        }

        // Check if already blocked
        if self.is_blocked(ip) {
            return Err("IP is blocked".to_string());
        }

        let mut trackers = self.request_trackers.lock().unwrap();
        let tracker = trackers.entry(*ip).or_insert_with(|| {
            RequestTracker::new(
                self.config.rate_limit_window_seconds,
                self.config.rate_limit_max_requests,
            )
        });

        tracker.cleanup_old_requests();

        if tracker.is_rate_limited() {
            // Auto-block if enabled
            if self.config.auto_block_enabled {
                drop(trackers); // Release lock before blocking
                let entry = BlacklistEntry::new(*ip, BlacklistReason::RateLimitExceeded)
                    .with_expiry(self.config.rate_limit_block_duration)
                    .with_description(format!(
                        "Auto-blocked: {} requests in {} seconds",
                        self.config.rate_limit_max_requests, self.config.rate_limit_window_seconds
                    ));
                self.block_ip(entry);
            }
            return Err("Rate limit exceeded".to_string());
        }

        tracker.record_request();
        Ok(())
    }

    /// Record authentication failure
    pub fn record_auth_failure(&self, ip: &IpAddr) -> Result<(), String> {
        // Skip for whitelisted IPs
        if self.is_whitelisted(ip) {
            return Ok(());
        }

        let mut trackers = self.failure_trackers.lock().unwrap();
        let tracker = trackers
            .entry(*ip)
            .or_insert_with(|| FailureTracker::new(self.config.auth_failure_window_seconds));

        tracker.cleanup_old_failures();
        tracker.record_failure();

        if tracker.failure_count() >= self.config.max_auth_failures {
            // Auto-block if enabled
            if self.config.auto_block_enabled {
                drop(trackers); // Release lock before blocking
                let entry = BlacklistEntry::new(*ip, BlacklistReason::BruteForce)
                    .with_expiry(self.config.auth_failure_block_duration)
                    .with_description(format!(
                        "Auto-blocked: {} auth failures in {} seconds",
                        self.config.max_auth_failures, self.config.auth_failure_window_seconds
                    ));
                self.block_ip(entry);
                return Err("Too many authentication failures - IP blocked".to_string());
            }
        }

        Ok(())
    }

    /// Record successful authentication (reset failure counter)
    pub fn record_auth_success(&self, ip: &IpAddr) {
        if let Some(tracker) = self.failure_trackers.lock().unwrap().get_mut(ip) {
            tracker.reset();
        }
    }

    /// Get blacklist entry for an IP
    pub fn get_blacklist_entry(&self, ip: &IpAddr) -> Option<BlacklistEntry> {
        self.blacklist.lock().unwrap().get(ip).cloned()
    }

    /// Get whitelist entry for an IP
    pub fn get_whitelist_entry(&self, ip: &IpAddr) -> Option<WhitelistEntry> {
        self.whitelist.lock().unwrap().get(ip).cloned()
    }

    /// List all blacklisted IPs
    pub fn list_blacklist(&self) -> Vec<BlacklistEntry> {
        self.blacklist
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// List all whitelisted IPs
    pub fn list_whitelist(&self) -> Vec<WhitelistEntry> {
        self.whitelist
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Cleanup expired blacklist entries
    pub fn cleanup_expired(&self) -> usize {
        let mut blacklist = self.blacklist.lock().unwrap();
        let expired: Vec<IpAddr> = blacklist
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(ip, _)| *ip)
            .collect();

        for ip in &expired {
            blacklist.remove(ip);
        }

        expired.len()
    }

    /// Get statistics
    pub fn get_statistics(&self) -> BlacklistStatistics {
        let blacklist = self.blacklist.lock().unwrap();
        let whitelist = self.whitelist.lock().unwrap();
        let request_trackers = self.request_trackers.lock().unwrap();
        let failure_trackers = self.failure_trackers.lock().unwrap();

        let active_blocks = blacklist.values().filter(|e| !e.is_expired()).count();
        let permanent_blocks = blacklist.values().filter(|e| e.is_permanent()).count();
        let temporary_blocks = active_blocks - permanent_blocks;

        BlacklistStatistics {
            total_blacklisted: blacklist.len(),
            active_blocks,
            permanent_blocks,
            temporary_blocks,
            whitelisted_ips: whitelist.len(),
            tracked_ips: request_trackers.len(),
            ips_with_failures: failure_trackers.len(),
        }
    }

    /// Get request count for an IP
    pub fn get_request_count(&self, ip: &IpAddr) -> usize {
        self.request_trackers
            .lock()
            .unwrap()
            .get(ip)
            .map(|t| t.request_count())
            .unwrap_or(0)
    }

    /// Get failure count for an IP
    pub fn get_failure_count(&self, ip: &IpAddr) -> usize {
        self.failure_trackers
            .lock()
            .unwrap()
            .get(ip)
            .map(|t| t.failure_count())
            .unwrap_or(0)
    }
}

/// Blacklist statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlacklistStatistics {
    pub total_blacklisted: usize,
    pub active_blocks: usize,
    pub permanent_blocks: usize,
    pub temporary_blocks: usize,
    pub whitelisted_ips: usize,
    pub tracked_ips: usize,
    pub ips_with_failures: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn test_ip() -> IpAddr {
        IpAddr::from_str("192.168.1.100").unwrap()
    }

    #[test]
    fn test_blacklist_entry_creation() {
        let ip = test_ip();
        let entry = BlacklistEntry::new(ip, BlacklistReason::Manual)
            .with_description("Test block".to_string())
            .with_added_by("admin".to_string());

        assert_eq!(entry.ip_address, ip);
        assert_eq!(entry.reason, BlacklistReason::Manual);
        assert!(entry.is_permanent());
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_blacklist_entry_expiry() {
        let ip = test_ip();
        let mut entry = BlacklistEntry::new(ip, BlacklistReason::RateLimitExceeded)
            .with_expiry(Duration::seconds(1));

        assert!(!entry.is_permanent());
        assert!(!entry.is_expired());

        // Simulate expiry
        entry.expires_at = Some(Utc::now() - Duration::seconds(1));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_whitelist_entry() {
        let ip = test_ip();
        let entry = WhitelistEntry::new(ip)
            .with_description("Trusted IP".to_string())
            .with_added_by("admin".to_string());

        assert_eq!(entry.ip_address, ip);
        assert_eq!(entry.description, "Trusted IP");
    }

    #[test]
    fn test_ip_blocking() {
        let manager = IpBlacklistManager::new(BlacklistConfig::default());
        let ip = test_ip();

        assert!(!manager.is_blocked(&ip));

        let entry = BlacklistEntry::new(ip, BlacklistReason::Manual);
        manager.block_ip(entry);

        assert!(manager.is_blocked(&ip));

        manager.unblock_ip(&ip);
        assert!(!manager.is_blocked(&ip));
    }

    #[test]
    fn test_ip_whitelisting() {
        let manager = IpBlacklistManager::new(BlacklistConfig::default());
        let ip = test_ip();

        // Block IP
        let entry = BlacklistEntry::new(ip, BlacklistReason::Manual);
        manager.block_ip(entry);

        // Whitelist should override blacklist
        let whitelist_entry = WhitelistEntry::new(ip);
        manager.whitelist_ip(whitelist_entry);

        assert!(!manager.is_blocked(&ip));
        assert!(manager.is_whitelisted(&ip));
    }

    #[test]
    fn test_rate_limiting() {
        let mut config = BlacklistConfig::default();
        config.rate_limit_max_requests = 3;
        config.auto_block_enabled = false;

        let manager = IpBlacklistManager::new(config);
        let ip = test_ip();

        // First 3 requests should succeed
        for _ in 0..3 {
            assert!(manager.check_rate_limit(&ip).is_ok());
        }

        // 4th request should fail
        assert!(manager.check_rate_limit(&ip).is_err());
    }

    #[test]
    fn test_auth_failure_tracking() {
        let mut config = BlacklistConfig::default();
        config.max_auth_failures = 3;
        config.auto_block_enabled = true;

        let manager = IpBlacklistManager::new(config);
        let ip = test_ip();

        // Record failures
        for _ in 0..2 {
            assert!(manager.record_auth_failure(&ip).is_ok());
        }

        // 3rd failure should trigger auto-block
        assert!(manager.record_auth_failure(&ip).is_err());
        assert!(manager.is_blocked(&ip));
    }

    #[test]
    fn test_auth_success_resets_failures() {
        let mut config = BlacklistConfig::default();
        config.max_auth_failures = 3;

        let manager = IpBlacklistManager::new(config);
        let ip = test_ip();

        // Record some failures
        manager.record_auth_failure(&ip).unwrap();
        manager.record_auth_failure(&ip).unwrap();
        assert_eq!(manager.get_failure_count(&ip), 2);

        // Success should reset
        manager.record_auth_success(&ip);
        assert_eq!(manager.get_failure_count(&ip), 0);
    }

    #[test]
    fn test_cleanup_expired() {
        let manager = IpBlacklistManager::new(BlacklistConfig::default());

        // Add expired entry
        let ip1 = IpAddr::from_str("192.168.1.100").unwrap();
        let mut entry1 = BlacklistEntry::new(ip1, BlacklistReason::Manual);
        entry1.expires_at = Some(Utc::now() - Duration::hours(1));
        manager.block_ip(entry1);

        // Add active entry
        let ip2 = IpAddr::from_str("192.168.1.101").unwrap();
        let entry2 = BlacklistEntry::new(ip2, BlacklistReason::Manual);
        manager.block_ip(entry2);

        let removed = manager.cleanup_expired();
        assert_eq!(removed, 1);
        assert!(!manager.is_blocked(&ip1));
        assert!(manager.is_blocked(&ip2));
    }

    #[test]
    fn test_statistics() {
        let manager = IpBlacklistManager::new(BlacklistConfig::default());

        // Add some entries
        let ip1 = IpAddr::from_str("192.168.1.100").unwrap();
        let ip2 = IpAddr::from_str("192.168.1.101").unwrap();

        manager.block_ip(BlacklistEntry::new(ip1, BlacklistReason::Manual));
        manager.whitelist_ip(WhitelistEntry::new(ip2));

        let stats = manager.get_statistics();
        assert_eq!(stats.total_blacklisted, 1);
        assert_eq!(stats.whitelisted_ips, 1);
        assert_eq!(stats.permanent_blocks, 1);
    }

    #[test]
    fn test_whitelist_bypasses_rate_limit() {
        let mut config = BlacklistConfig::default();
        config.rate_limit_max_requests = 2;

        let manager = IpBlacklistManager::new(config);
        let ip = test_ip();

        // Whitelist the IP
        manager.whitelist_ip(WhitelistEntry::new(ip));

        // Should not be rate limited even with many requests
        for _ in 0..10 {
            assert!(manager.check_rate_limit(&ip).is_ok());
        }
    }

    #[test]
    fn test_reason_display() {
        assert_eq!(BlacklistReason::Manual.to_string(), "Manual block");
        assert_eq!(
            BlacklistReason::BruteForce.to_string(),
            "Brute force attack"
        );
        assert_eq!(
            BlacklistReason::Custom("Test".to_string()).to_string(),
            "Test"
        );
    }
}
