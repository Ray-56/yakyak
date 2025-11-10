/// SIP Trunk configuration and management
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

/// SIP trunk type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrunkType {
    /// Register-based trunk (client registers to provider)
    Register,
    /// IP-based trunk (authenticated by source IP)
    IpBased,
    /// Peer trunk (bidirectional, no registration)
    Peer,
}

/// Trunk direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrunkDirection {
    /// Inbound calls only
    Inbound,
    /// Outbound calls only
    Outbound,
    /// Both inbound and outbound
    Bidirectional,
}

/// Codec preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodecPreference {
    pub codec: String,
    pub priority: u32,
}

/// SIP Trunk configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SipTrunk {
    pub id: Uuid,
    pub name: String,
    pub trunk_type: TrunkType,
    pub direction: TrunkDirection,

    // Provider settings
    pub provider_name: String,
    pub sip_server: String,
    pub sip_port: u16,
    pub backup_server: Option<String>,

    // Authentication
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth_username: Option<String>,
    pub realm: Option<String>,

    // IP-based authentication
    pub allowed_ips: Vec<String>,

    // Registration settings (for Register type)
    pub register_enabled: bool,
    pub register_interval: u32, // seconds
    pub register_expiry: u32,   // seconds

    // Call routing
    pub prefix: Option<String>,
    pub strip_prefix: bool,
    pub add_prefix: Option<String>,
    pub caller_id_number: Option<String>,
    pub caller_id_name: Option<String>,

    // Codec settings
    pub codecs: Vec<CodecPreference>,
    pub dtmf_mode: DtmfMode,

    // Capacity limits
    pub max_concurrent_calls: u32,
    pub max_calls_per_second: u32,

    // Quality settings
    pub enable_rtcp: bool,
    pub enable_t38: bool,
    pub enable_srtp: bool,

    // Status
    pub enabled: bool,
    pub registered: bool,
    pub last_registration: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DTMF transmission mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DtmfMode {
    /// RFC 2833 (RTP events)
    Rfc2833,
    /// SIP INFO
    SipInfo,
    /// In-band audio
    Inband,
}

impl SipTrunk {
    /// Create a new SIP trunk
    pub fn new(name: String, provider_name: String, trunk_type: TrunkType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            trunk_type,
            direction: TrunkDirection::Bidirectional,
            provider_name,
            sip_server: String::new(),
            sip_port: 5060,
            backup_server: None,
            username: None,
            password: None,
            auth_username: None,
            realm: None,
            allowed_ips: Vec::new(),
            register_enabled: trunk_type == TrunkType::Register,
            register_interval: 60,
            register_expiry: 3600,
            prefix: None,
            strip_prefix: false,
            add_prefix: None,
            caller_id_number: None,
            caller_id_name: None,
            codecs: Self::default_codecs(),
            dtmf_mode: DtmfMode::Rfc2833,
            max_concurrent_calls: 100,
            max_calls_per_second: 10,
            enable_rtcp: true,
            enable_t38: false,
            enable_srtp: false,
            enabled: true,
            registered: false,
            last_registration: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set SIP server details
    pub fn with_server(mut self, server: String, port: u16) -> Self {
        self.sip_server = server;
        self.sip_port = port;
        self
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, username: String, password: String) -> Self {
        self.username = Some(username.clone());
        self.password = Some(password);
        self.auth_username = Some(username);
        self
    }

    /// Set caller ID
    pub fn with_caller_id(mut self, number: String, name: String) -> Self {
        self.caller_id_number = Some(number);
        self.caller_id_name = Some(name);
        self
    }

    /// Set direction
    pub fn with_direction(mut self, direction: TrunkDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Add allowed IP for IP-based trunk
    pub fn add_allowed_ip(&mut self, ip: String) {
        if !self.allowed_ips.contains(&ip) {
            self.allowed_ips.push(ip);
        }
    }

    /// Check if trunk can handle outbound calls
    pub fn can_handle_outbound(&self) -> bool {
        self.enabled
            && (self.direction == TrunkDirection::Outbound
                || self.direction == TrunkDirection::Bidirectional)
    }

    /// Check if trunk can handle inbound calls
    pub fn can_handle_inbound(&self) -> bool {
        self.enabled
            && (self.direction == TrunkDirection::Inbound
                || self.direction == TrunkDirection::Bidirectional)
    }

    /// Check if IP is allowed for IP-based trunk
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if self.trunk_type != TrunkType::IpBased {
            return true;
        }
        self.allowed_ips.iter().any(|allowed| allowed == ip)
    }

    /// Format number for outbound call
    pub fn format_outbound_number(&self, number: &str) -> String {
        let mut formatted = number.to_string();

        // Strip prefix if configured
        if self.strip_prefix {
            if let Some(ref prefix) = self.prefix {
                if formatted.starts_with(prefix) {
                    formatted = formatted[prefix.len()..].to_string();
                }
            }
        }

        // Add prefix if configured
        if let Some(ref add_prefix) = self.add_prefix {
            formatted = format!("{}{}", add_prefix, formatted);
        }

        formatted
    }

    /// Default codec preferences
    fn default_codecs() -> Vec<CodecPreference> {
        vec![
            CodecPreference {
                codec: "PCMU".to_string(),
                priority: 100,
            },
            CodecPreference {
                codec: "PCMA".to_string(),
                priority: 99,
            },
            CodecPreference {
                codec: "G729".to_string(),
                priority: 98,
            },
        ]
    }

    /// Mark as registered
    pub fn mark_registered(&mut self) {
        self.registered = true;
        self.last_registration = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Mark as unregistered
    pub fn mark_unregistered(&mut self) {
        self.registered = false;
        self.updated_at = Utc::now();
    }

    /// Check if registration is needed
    pub fn needs_registration(&self) -> bool {
        if !self.register_enabled || self.trunk_type != TrunkType::Register {
            return false;
        }

        if !self.registered {
            return true;
        }

        // Check if registration expired
        if let Some(last_reg) = self.last_registration {
            let elapsed = (Utc::now() - last_reg).num_seconds() as u32;
            elapsed >= (self.register_expiry - 60) // Re-register 60 seconds before expiry
        } else {
            true
        }
    }
}

/// Trunk statistics
#[derive(Debug, Clone, Serialize)]
pub struct TrunkStatistics {
    pub trunk_id: Uuid,
    pub current_calls: u32,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub average_call_duration: f64,
    pub total_minutes: f64,
    pub last_call_time: Option<DateTime<Utc>>,
}

impl TrunkStatistics {
    pub fn new(trunk_id: Uuid) -> Self {
        Self {
            trunk_id,
            current_calls: 0,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            average_call_duration: 0.0,
            total_minutes: 0.0,
            last_call_time: None,
        }
    }

    /// Record a completed call
    pub fn record_call(&mut self, duration_seconds: u64, successful: bool) {
        self.total_calls += 1;

        if successful {
            self.successful_calls += 1;
            let minutes = duration_seconds as f64 / 60.0;
            self.total_minutes += minutes;

            // Update average
            self.average_call_duration = self.total_minutes / self.successful_calls as f64 * 60.0;
        } else {
            self.failed_calls += 1;
        }

        self.last_call_time = Some(Utc::now());
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.successful_calls as f64 / self.total_calls as f64) * 100.0
    }
}

/// Repository trait for SIP trunk persistence
#[async_trait::async_trait]
pub trait SipTrunkRepository: Send + Sync {
    /// Create a new SIP trunk
    async fn create_trunk(&self, trunk: SipTrunk) -> Result<SipTrunk, String>;

    /// Get a trunk by ID
    async fn get_trunk(&self, trunk_id: Uuid) -> Result<Option<SipTrunk>, String>;

    /// Get a trunk by name
    async fn get_trunk_by_name(&self, name: &str) -> Result<Option<SipTrunk>, String>;

    /// Update a trunk
    async fn update_trunk(&self, trunk: &SipTrunk) -> Result<(), String>;

    /// Delete a trunk
    async fn delete_trunk(&self, trunk_id: Uuid) -> Result<(), String>;

    /// List all trunks
    async fn list_trunks(&self, enabled_only: bool) -> Result<Vec<SipTrunk>, String>;

    /// Get or create statistics for a trunk
    async fn get_statistics(&self, trunk_id: Uuid) -> Result<Option<TrunkStatistics>, String>;

    /// Update statistics for a trunk
    async fn update_statistics(&self, stats: &TrunkStatistics) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sip_trunk_creation() {
        let trunk = SipTrunk::new(
            "Provider1".to_string(),
            "SIP Provider Inc.".to_string(),
            TrunkType::Register,
        );

        assert_eq!(trunk.name, "Provider1");
        assert_eq!(trunk.trunk_type, TrunkType::Register);
        assert!(trunk.register_enabled);
        assert!(!trunk.registered);
    }

    #[test]
    fn test_trunk_with_server() {
        let trunk = SipTrunk::new(
            "Provider1".to_string(),
            "SIP Provider Inc.".to_string(),
            TrunkType::Register,
        )
        .with_server("sip.provider.com".to_string(), 5060);

        assert_eq!(trunk.sip_server, "sip.provider.com");
        assert_eq!(trunk.sip_port, 5060);
    }

    #[test]
    fn test_trunk_direction() {
        let mut trunk = SipTrunk::new(
            "Provider1".to_string(),
            "Provider".to_string(),
            TrunkType::Register,
        )
        .with_direction(TrunkDirection::Outbound);

        assert!(trunk.can_handle_outbound());
        assert!(!trunk.can_handle_inbound());

        trunk.direction = TrunkDirection::Bidirectional;
        assert!(trunk.can_handle_outbound());
        assert!(trunk.can_handle_inbound());
    }

    #[test]
    fn test_ip_based_trunk() {
        let mut trunk = SipTrunk::new(
            "Provider1".to_string(),
            "Provider".to_string(),
            TrunkType::IpBased,
        );

        trunk.add_allowed_ip("192.168.1.100".to_string());
        trunk.add_allowed_ip("192.168.1.101".to_string());

        assert!(trunk.is_ip_allowed("192.168.1.100"));
        assert!(trunk.is_ip_allowed("192.168.1.101"));
        assert!(!trunk.is_ip_allowed("192.168.1.102"));
    }

    #[test]
    fn test_number_formatting() {
        let mut trunk = SipTrunk::new(
            "Provider1".to_string(),
            "Provider".to_string(),
            TrunkType::Register,
        );

        trunk.prefix = Some("9".to_string());
        trunk.strip_prefix = true;
        trunk.add_prefix = Some("1".to_string());

        let formatted = trunk.format_outbound_number("95551234");
        assert_eq!(formatted, "15551234");
    }

    #[test]
    fn test_registration_needed() {
        let mut trunk = SipTrunk::new(
            "Provider1".to_string(),
            "Provider".to_string(),
            TrunkType::Register,
        );

        assert!(trunk.needs_registration());

        trunk.mark_registered();
        assert!(!trunk.needs_registration());
        assert!(trunk.registered);
        assert!(trunk.last_registration.is_some());
    }

    #[test]
    fn test_trunk_statistics() {
        let trunk_id = Uuid::new_v4();
        let mut stats = TrunkStatistics::new(trunk_id);

        assert_eq!(stats.total_calls, 0);
        assert_eq!(stats.success_rate(), 0.0);

        stats.record_call(120, true);
        stats.record_call(180, true);
        stats.record_call(0, false);

        assert_eq!(stats.total_calls, 3);
        assert_eq!(stats.successful_calls, 2);
        assert_eq!(stats.failed_calls, 1);
        assert_eq!(stats.success_rate(), 66.66666666666666);
    }

    #[test]
    fn test_caller_id() {
        let trunk = SipTrunk::new(
            "Provider1".to_string(),
            "Provider".to_string(),
            TrunkType::Register,
        )
        .with_caller_id("5551234".to_string(), "Company Name".to_string());

        assert_eq!(trunk.caller_id_number, Some("5551234".to_string()));
        assert_eq!(trunk.caller_id_name, Some("Company Name".to_string()));
    }
}
