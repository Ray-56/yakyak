//! Enhanced security features for authentication and authorization
//!
//! Provides password policies, strength validation, security auditing,
//! and advanced protection mechanisms for enterprise security requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PasswordStrength {
    /// Very weak password (score 0-20)
    VeryWeak,
    /// Weak password (score 21-40)
    Weak,
    /// Fair password (score 41-60)
    Fair,
    /// Strong password (score 61-80)
    Strong,
    /// Very strong password (score 81-100)
    VeryStrong,
}

impl PasswordStrength {
    pub fn from_score(score: u32) -> Self {
        match score {
            0..=20 => PasswordStrength::VeryWeak,
            21..=40 => PasswordStrength::Weak,
            41..=60 => PasswordStrength::Fair,
            61..=80 => PasswordStrength::Strong,
            _ => PasswordStrength::VeryStrong,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Strong => "Strong",
            PasswordStrength::VeryStrong => "Very Strong",
        }
    }

    pub fn meets_minimum(&self, min: PasswordStrength) -> bool {
        self >= &min
    }
}

/// Password strength evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrengthResult {
    pub strength: PasswordStrength,
    pub score: u32,
    pub feedback: Vec<String>,
    pub is_acceptable: bool,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Maximum password length
    pub max_length: usize,
    /// Require at least one uppercase letter
    pub require_uppercase: bool,
    /// Require at least one lowercase letter
    pub require_lowercase: bool,
    /// Require at least one digit
    pub require_digit: bool,
    /// Require at least one special character
    pub require_special: bool,
    /// Minimum password strength level
    pub min_strength: PasswordStrength,
    /// Disallow common passwords
    pub disallow_common: bool,
    /// Disallow username in password
    pub disallow_username: bool,
    /// Password expiry in days (0 = no expiry)
    pub expiry_days: u32,
    /// Number of previous passwords to remember
    pub history_count: usize,
    /// Minimum time between password changes (hours)
    pub min_age_hours: u32,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            min_strength: PasswordStrength::Fair,
            disallow_common: true,
            disallow_username: true,
            expiry_days: 90,
            history_count: 5,
            min_age_hours: 24,
        }
    }
}

impl PasswordPolicy {
    /// Create a strict password policy for high-security environments
    pub fn strict() -> Self {
        Self {
            min_length: 12,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: true,
            min_strength: PasswordStrength::Strong,
            disallow_common: true,
            disallow_username: true,
            expiry_days: 60,
            history_count: 10,
            min_age_hours: 24,
        }
    }

    /// Create a relaxed password policy for development
    pub fn relaxed() -> Self {
        Self {
            min_length: 6,
            max_length: 128,
            require_uppercase: false,
            require_lowercase: true,
            require_digit: false,
            require_special: false,
            min_strength: PasswordStrength::Weak,
            disallow_common: false,
            disallow_username: false,
            expiry_days: 0,
            history_count: 0,
            min_age_hours: 0,
        }
    }

    /// Validate password against policy
    pub fn validate(&self, password: &str, username: Option<&str>) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Length check
        if password.len() < self.min_length {
            errors.push(format!("Password must be at least {} characters", self.min_length));
        }
        if password.len() > self.max_length {
            errors.push(format!("Password must not exceed {} characters", self.max_length));
        }

        // Character requirements
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        if self.require_digit && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one digit".to_string());
        }
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one special character".to_string());
        }

        // Username check
        if self.disallow_username {
            if let Some(username) = username {
                if password.to_lowercase().contains(&username.to_lowercase()) {
                    errors.push("Password must not contain username".to_string());
                }
            }
        }

        // Common password check
        if self.disallow_common && is_common_password(password) {
            errors.push("Password is too common and easily guessed".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Password strength evaluator
pub struct PasswordStrengthEvaluator {
    policy: PasswordPolicy,
}

impl PasswordStrengthEvaluator {
    pub fn new(policy: PasswordPolicy) -> Self {
        Self { policy }
    }

    /// Evaluate password strength
    pub fn evaluate(&self, password: &str, username: Option<&str>) -> PasswordStrengthResult {
        let mut score = 0u32;
        let mut feedback = Vec::new();

        // Length score (max 30 points)
        let length = password.len();
        score += match length {
            0..=5 => 0,
            6..=7 => 10,
            8..=11 => 20,
            12..=15 => 25,
            _ => 30,
        };

        // Character variety (max 40 points)
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let variety_count = [has_lower, has_upper, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        score += (variety_count as u32) * 10;

        if variety_count < 3 {
            feedback.push("Use a mix of uppercase, lowercase, numbers, and symbols".to_string());
        }

        // Complexity patterns (max 20 points)
        let has_consecutive = has_consecutive_chars(password);
        let has_repeating = has_repeating_chars(password);

        if !has_consecutive {
            score += 10;
        } else {
            feedback.push("Avoid consecutive characters (abc, 123)".to_string());
        }

        if !has_repeating {
            score += 10;
        } else {
            feedback.push("Avoid repeating characters (aaa, 111)".to_string());
        }

        // Common password check (max 10 points)
        if !is_common_password(password) {
            score += 10;
        } else {
            feedback.push("Password is in the list of common passwords".to_string());
        }

        // Username similarity check
        if let Some(username) = username {
            if password.to_lowercase().contains(&username.to_lowercase()) {
                score = score.saturating_sub(20);
                feedback.push("Password should not contain your username".to_string());
            }
        }

        // Determine strength
        let strength = PasswordStrength::from_score(score);

        // Check if acceptable per policy
        let is_acceptable = strength.meets_minimum(self.policy.min_strength)
            && self.policy.validate(password, username).is_ok();

        if !is_acceptable && feedback.is_empty() {
            feedback.push(format!(
                "Password must meet minimum strength: {}",
                self.policy.min_strength.as_str()
            ));
        }

        PasswordStrengthResult {
            strength,
            score,
            feedback,
            is_acceptable,
        }
    }
}

/// Security event types for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecurityEvent {
    /// User login attempt
    LoginAttempt {
        username: String,
        ip: String,
        success: bool,
        method: String,
        reason: Option<String>,
    },
    /// User logout
    Logout {
        username: String,
        ip: String,
        session_duration_seconds: u64,
    },
    /// Password change
    PasswordChange {
        username: String,
        ip: String,
        forced: bool,
    },
    /// Account lockout
    AccountLockout {
        username: String,
        ip: String,
        reason: String,
        duration_seconds: u64,
    },
    /// Permission denied
    PermissionDenied {
        username: String,
        ip: String,
        resource: String,
        action: String,
    },
    /// Security policy violation
    PolicyViolation {
        username: Option<String>,
        ip: String,
        policy: String,
        details: String,
    },
    /// Suspicious activity detected
    SuspiciousActivity {
        username: Option<String>,
        ip: String,
        activity: String,
        risk_score: u32,
    },
    /// Admin action
    AdminAction {
        admin_username: String,
        ip: String,
        action: String,
        target: String,
    },
}

/// Security audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event: SecurityEvent,
    pub severity: SecuritySeverity,
    pub metadata: HashMap<String, String>,
}

/// Security event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Informational event
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl SecurityAuditEntry {
    pub fn new(event: SecurityEvent, severity: SecuritySeverity) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event,
            severity,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Security audit logger
pub struct SecurityAuditLogger {
    entries: Arc<Mutex<VecDeque<SecurityAuditEntry>>>,
    max_entries: usize,
    alert_callback: Option<Arc<dyn Fn(&SecurityAuditEntry) + Send + Sync>>,
}

impl SecurityAuditLogger {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::new())),
            max_entries,
            alert_callback: None,
        }
    }

    /// Set alert callback for high/critical events
    pub fn set_alert_callback<F>(&mut self, callback: F)
    where
        F: Fn(&SecurityAuditEntry) + Send + Sync + 'static,
    {
        self.alert_callback = Some(Arc::new(callback));
    }

    /// Log a security event
    pub fn log(&self, event: SecurityEvent, severity: SecuritySeverity) -> Uuid {
        let entry = SecurityAuditEntry::new(event, severity);
        let id = entry.id;

        // Trigger alert for high/critical events
        if matches!(severity, SecuritySeverity::High | SecuritySeverity::Critical) {
            if let Some(ref callback) = self.alert_callback {
                callback(&entry);
            }
        }

        let mut entries = self.entries.lock().unwrap();
        entries.push_back(entry);

        // Maintain max size
        if entries.len() > self.max_entries {
            entries.pop_front();
        }

        id
    }

    /// Get recent entries
    pub fn get_recent(&self, count: usize) -> Vec<SecurityAuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries.iter().rev().take(count).cloned().collect()
    }

    /// Get entries by severity
    pub fn get_by_severity(&self, severity: SecuritySeverity) -> Vec<SecurityAuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| e.severity == severity)
            .cloned()
            .collect()
    }

    /// Get entries for a specific user
    pub fn get_by_user(&self, username: &str) -> Vec<SecurityAuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| match &e.event {
                SecurityEvent::LoginAttempt { username: u, .. }
                | SecurityEvent::Logout { username: u, .. }
                | SecurityEvent::PasswordChange { username: u, .. }
                | SecurityEvent::AccountLockout { username: u, .. }
                | SecurityEvent::PermissionDenied { username: u, .. } => u == username,
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Get entries by IP address
    pub fn get_by_ip(&self, ip: &str) -> Vec<SecurityAuditEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .filter(|e| match &e.event {
                SecurityEvent::LoginAttempt { ip: i, .. }
                | SecurityEvent::Logout { ip: i, .. }
                | SecurityEvent::PasswordChange { ip: i, .. }
                | SecurityEvent::AccountLockout { ip: i, .. }
                | SecurityEvent::PermissionDenied { ip: i, .. }
                | SecurityEvent::PolicyViolation { ip: i, .. }
                | SecurityEvent::SuspiciousActivity { ip: i, .. }
                | SecurityEvent::AdminAction { ip: i, .. } => i == ip,
            })
            .cloned()
            .collect()
    }

    /// Clear all entries
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }

    /// Get total entry count
    pub fn count(&self) -> usize {
        let entries = self.entries.lock().unwrap();
        entries.len()
    }
}

/// Check if password contains consecutive characters
fn has_consecutive_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
        if window.len() == 3 {
            let c1 = window[0] as u32;
            let c2 = window[1] as u32;
            let c3 = window[2] as u32;
            if c2 == c1 + 1 && c3 == c2 + 1 {
                return true;
            }
        }
    }
    false
}

/// Check if password has repeating characters
fn has_repeating_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    for window in chars.windows(3) {
        if window.len() == 3 && window[0] == window[1] && window[1] == window[2] {
            return true;
        }
    }
    false
}

/// Check if password is in common password list
fn is_common_password(password: &str) -> bool {
    const COMMON_PASSWORDS: &[&str] = &[
        "password",
        "123456",
        "12345678",
        "qwerty",
        "abc123",
        "monkey",
        "1234567",
        "letmein",
        "trustno1",
        "dragon",
        "baseball",
        "111111",
        "iloveyou",
        "master",
        "sunshine",
        "ashley",
        "bailey",
        "passw0rd",
        "shadow",
        "123123",
        "654321",
        "superman",
        "qazwsx",
        "michael",
        "football",
    ];

    COMMON_PASSWORDS.contains(&password.to_lowercase().as_str())
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_levels() {
        assert_eq!(PasswordStrength::from_score(10), PasswordStrength::VeryWeak);
        assert_eq!(PasswordStrength::from_score(30), PasswordStrength::Weak);
        assert_eq!(PasswordStrength::from_score(50), PasswordStrength::Fair);
        assert_eq!(PasswordStrength::from_score(70), PasswordStrength::Strong);
        assert_eq!(PasswordStrength::from_score(90), PasswordStrength::VeryStrong);
    }

    #[test]
    fn test_password_policy_default() {
        let policy = PasswordPolicy::default();
        assert_eq!(policy.min_length, 8);
        assert!(policy.require_uppercase);
        assert!(policy.require_digit);
    }

    #[test]
    fn test_password_policy_validation() {
        let policy = PasswordPolicy::default();

        // Too short
        assert!(policy.validate("Pass1!", None).is_err());

        // Missing uppercase
        assert!(policy.validate("password123!", None).is_err());

        // Missing digit
        assert!(policy.validate("Password!", None).is_err());

        // Valid password
        assert!(policy.validate("Password123!", None).is_ok());
    }

    #[test]
    fn test_password_strength_evaluation() {
        let evaluator = PasswordStrengthEvaluator::new(PasswordPolicy::default());

        let weak = evaluator.evaluate("password", None);
        assert!(weak.strength <= PasswordStrength::Weak);

        let strong = evaluator.evaluate("MyS3cur3P@ssw0rd!", None);
        assert!(strong.strength >= PasswordStrength::Strong);
    }

    #[test]
    fn test_common_password_detection() {
        assert!(is_common_password("password"));
        assert!(is_common_password("123456"));
        assert!(!is_common_password("MyUniqueP@ss123"));
    }

    #[test]
    fn test_consecutive_chars() {
        assert!(has_consecutive_chars("abc123"));
        assert!(has_consecutive_chars("xyz789"));
        assert!(!has_consecutive_chars("aDb1c3"));
    }

    #[test]
    fn test_repeating_chars() {
        assert!(has_repeating_chars("aaa"));
        assert!(has_repeating_chars("password111"));
        assert!(!has_repeating_chars("password12"));
    }

    #[test]
    fn test_security_audit_logger() {
        let logger = SecurityAuditLogger::new(100);

        let event = SecurityEvent::LoginAttempt {
            username: "alice".to_string(),
            ip: "192.168.1.1".to_string(),
            success: true,
            method: "password".to_string(),
            reason: None,
        };

        logger.log(event, SecuritySeverity::Info);

        assert_eq!(logger.count(), 1);

        let recent = logger.get_recent(10);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_audit_logger_filtering() {
        let logger = SecurityAuditLogger::new(100);

        logger.log(
            SecurityEvent::LoginAttempt {
                username: "alice".to_string(),
                ip: "192.168.1.1".to_string(),
                success: true,
                method: "password".to_string(),
                reason: None,
            },
            SecuritySeverity::Info,
        );

        logger.log(
            SecurityEvent::PolicyViolation {
                username: None,
                ip: "192.168.1.2".to_string(),
                policy: "rate_limit".to_string(),
                details: "Too many requests".to_string(),
            },
            SecuritySeverity::Medium,
        );

        let by_user = logger.get_by_user("alice");
        assert_eq!(by_user.len(), 1);

        let by_severity = logger.get_by_severity(SecuritySeverity::Medium);
        assert_eq!(by_severity.len(), 1);
    }
}
