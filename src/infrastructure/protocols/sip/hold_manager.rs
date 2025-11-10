/// Call hold/resume manager using re-INVITE
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Call hold state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HoldState {
    /// Call is active (sendrecv)
    Active,
    /// Call is on hold by local party (sendonly - sending music on hold)
    LocalHold,
    /// Call is on hold by remote party (recvonly - receiving music on hold)
    RemoteHold,
    /// Call is on hold by both parties (inactive)
    BothHold,
}

/// Hold information for a call
#[derive(Debug, Clone)]
pub struct HoldInfo {
    pub call_id: String,
    pub state: HoldState,
    pub local_sdp: Option<String>,
    pub remote_sdp: Option<String>,
}

/// Manages call hold/resume state
pub struct HoldManager {
    holds: Arc<RwLock<HashMap<String, HoldInfo>>>,
}

impl HoldManager {
    pub fn new() -> Self {
        Self {
            holds: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Put a call on hold (local hold)
    pub async fn hold_call(&self, call_id: &str) -> Result<(), String> {
        let mut holds = self.holds.write().await;

        if let Some(info) = holds.get_mut(call_id) {
            match info.state {
                HoldState::Active => {
                    info.state = HoldState::LocalHold;
                    info!("Call {} placed on hold", call_id);
                    Ok(())
                }
                HoldState::RemoteHold => {
                    info.state = HoldState::BothHold;
                    info!("Call {} placed on hold (both parties)", call_id);
                    Ok(())
                }
                _ => Err("Call is already on hold".to_string()),
            }
        } else {
            // Create new hold info
            holds.insert(
                call_id.to_string(),
                HoldInfo {
                    call_id: call_id.to_string(),
                    state: HoldState::LocalHold,
                    local_sdp: None,
                    remote_sdp: None,
                },
            );
            info!("Call {} placed on hold (new)", call_id);
            Ok(())
        }
    }

    /// Resume a call from hold
    pub async fn resume_call(&self, call_id: &str) -> Result<(), String> {
        let mut holds = self.holds.write().await;

        if let Some(info) = holds.get_mut(call_id) {
            match info.state {
                HoldState::LocalHold => {
                    info.state = HoldState::Active;
                    info!("Call {} resumed from hold", call_id);
                    Ok(())
                }
                HoldState::BothHold => {
                    info.state = HoldState::RemoteHold;
                    info!("Call {} resumed from hold (remote still holding)", call_id);
                    Ok(())
                }
                _ => Err("Call is not on hold".to_string()),
            }
        } else {
            Err("Call not found".to_string())
        }
    }

    /// Mark remote party as holding
    pub async fn remote_hold(&self, call_id: &str) -> Result<(), String> {
        let mut holds = self.holds.write().await;

        if let Some(info) = holds.get_mut(call_id) {
            match info.state {
                HoldState::Active => {
                    info.state = HoldState::RemoteHold;
                    debug!("Call {} - remote party placed on hold", call_id);
                    Ok(())
                }
                HoldState::LocalHold => {
                    info.state = HoldState::BothHold;
                    debug!("Call {} - both parties on hold", call_id);
                    Ok(())
                }
                _ => Ok(()), // Already in appropriate state
            }
        } else {
            // Create new hold info
            holds.insert(
                call_id.to_string(),
                HoldInfo {
                    call_id: call_id.to_string(),
                    state: HoldState::RemoteHold,
                    local_sdp: None,
                    remote_sdp: None,
                },
            );
            debug!("Call {} - remote party placed on hold (new)", call_id);
            Ok(())
        }
    }

    /// Mark remote party as resuming
    pub async fn remote_resume(&self, call_id: &str) -> Result<(), String> {
        let mut holds = self.holds.write().await;

        if let Some(info) = holds.get_mut(call_id) {
            match info.state {
                HoldState::RemoteHold => {
                    info.state = HoldState::Active;
                    debug!("Call {} - remote party resumed", call_id);
                    Ok(())
                }
                HoldState::BothHold => {
                    info.state = HoldState::LocalHold;
                    debug!("Call {} - remote party resumed (local still holding)", call_id);
                    Ok(())
                }
                _ => Ok(()), // Already in appropriate state
            }
        } else {
            Err("Call not found".to_string())
        }
    }

    /// Get hold state for a call
    pub async fn get_state(&self, call_id: &str) -> Option<HoldState> {
        let holds = self.holds.read().await;
        holds.get(call_id).map(|info| info.state.clone())
    }

    /// Update SDP for a call
    pub async fn update_sdp(&self, call_id: &str, local_sdp: Option<String>, remote_sdp: Option<String>) {
        let mut holds = self.holds.write().await;

        if let Some(info) = holds.get_mut(call_id) {
            if let Some(sdp) = local_sdp {
                info.local_sdp = Some(sdp);
            }
            if let Some(sdp) = remote_sdp {
                info.remote_sdp = Some(sdp);
            }
        }
    }

    /// Remove call from hold manager
    pub async fn remove_call(&self, call_id: &str) {
        let mut holds = self.holds.write().await;
        holds.remove(call_id);
        debug!("Call {} removed from hold manager", call_id);
    }

    /// Check if call is on hold (any hold state except Active)
    pub async fn is_on_hold(&self, call_id: &str) -> bool {
        let holds = self.holds.read().await;
        holds.get(call_id).map(|info| info.state != HoldState::Active).unwrap_or(false)
    }

    /// Count total calls being tracked
    pub async fn count(&self) -> usize {
        let holds = self.holds.read().await;
        holds.len()
    }
}

impl Default for HoldManager {
    fn default() -> Self {
        Self::new()
    }
}

/// SDP manipulation for hold/resume
pub struct SdpHoldHelper;

impl SdpHoldHelper {
    /// Modify SDP to indicate call on hold (sendonly or inactive)
    pub fn create_hold_sdp(original_sdp: &str, inactive: bool) -> String {
        let mut lines: Vec<&str> = original_sdp.lines().collect();
        let mut result = Vec::new();
        let mut in_media = false;

        for line in lines.iter() {
            if line.starts_with("m=") {
                in_media = true;
                result.push(line.to_string());
            } else if line.starts_with("a=sendrecv") {
                // Replace sendrecv with sendonly or inactive
                if inactive {
                    result.push("a=inactive".to_string());
                } else {
                    result.push("a=sendonly".to_string());
                }
            } else if line.starts_with("a=sendonly") || line.starts_with("a=recvonly") || line.starts_with("a=inactive") {
                // Replace existing direction attribute
                if inactive {
                    result.push("a=inactive".to_string());
                } else {
                    result.push("a=sendonly".to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }

        result.join("\r\n")
    }

    /// Modify SDP to resume from hold (sendrecv)
    pub fn create_resume_sdp(original_sdp: &str) -> String {
        let mut lines: Vec<&str> = original_sdp.lines().collect();
        let mut result = Vec::new();

        for line in lines.iter() {
            if line.starts_with("a=sendonly") || line.starts_with("a=recvonly") || line.starts_with("a=inactive") {
                result.push("a=sendrecv".to_string());
            } else {
                result.push(line.to_string());
            }
        }

        result.join("\r\n")
    }

    /// Detect if SDP indicates hold state
    pub fn detect_hold_state(sdp: &str) -> HoldState {
        let has_sendonly = sdp.contains("a=sendonly");
        let has_recvonly = sdp.contains("a=recvonly");
        let has_inactive = sdp.contains("a=inactive");
        let has_sendrecv = sdp.contains("a=sendrecv");

        if has_inactive {
            HoldState::BothHold
        } else if has_sendonly {
            HoldState::RemoteHold // Remote is sending only (we are on hold)
        } else if has_recvonly {
            HoldState::LocalHold // Remote is receiving only (we are holding)
        } else if has_sendrecv || (!has_sendonly && !has_recvonly && !has_inactive) {
            HoldState::Active
        } else {
            HoldState::Active // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hold_call() {
        let manager = HoldManager::new();

        // Hold call
        manager.hold_call("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::LocalHold));

        // Try to hold again (should fail)
        assert!(manager.hold_call("test-call").await.is_err());
    }

    #[tokio::test]
    async fn test_resume_call() {
        let manager = HoldManager::new();

        // Hold then resume
        manager.hold_call("test-call").await.unwrap();
        manager.resume_call("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::Active));
    }

    #[tokio::test]
    async fn test_remote_hold() {
        let manager = HoldManager::new();

        // Remote hold
        manager.remote_hold("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::RemoteHold));

        // Local hold (should result in both hold)
        manager.hold_call("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::BothHold));
    }

    #[tokio::test]
    async fn test_both_hold_resume() {
        let manager = HoldManager::new();

        // Both parties hold
        manager.hold_call("test-call").await.unwrap();
        manager.remote_hold("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::BothHold));

        // Local resume
        manager.resume_call("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::RemoteHold));

        // Remote resume
        manager.remote_resume("test-call").await.unwrap();
        assert_eq!(manager.get_state("test-call").await, Some(HoldState::Active));
    }

    #[test]
    fn test_sdp_hold_modification() {
        let original_sdp = "v=0\r\no=- 123 456 IN IP4 192.168.1.1\r\ns=-\r\nc=IN IP4 192.168.1.1\r\nt=0 0\r\nm=audio 5004 RTP/AVP 0\r\na=sendrecv\r\n";

        // Create hold SDP
        let hold_sdp = SdpHoldHelper::create_hold_sdp(original_sdp, false);
        assert!(hold_sdp.contains("a=sendonly"));
        assert!(!hold_sdp.contains("a=sendrecv"));

        // Create inactive SDP
        let inactive_sdp = SdpHoldHelper::create_hold_sdp(original_sdp, true);
        assert!(inactive_sdp.contains("a=inactive"));

        // Create resume SDP
        let resume_sdp = SdpHoldHelper::create_resume_sdp(&hold_sdp);
        assert!(resume_sdp.contains("a=sendrecv"));
        assert!(!resume_sdp.contains("a=sendonly"));
    }

    #[test]
    fn test_detect_hold_state() {
        let active_sdp = "v=0\r\nm=audio 5004 RTP/AVP 0\r\na=sendrecv\r\n";
        assert_eq!(SdpHoldHelper::detect_hold_state(active_sdp), HoldState::Active);

        let sendonly_sdp = "v=0\r\nm=audio 5004 RTP/AVP 0\r\na=sendonly\r\n";
        assert_eq!(SdpHoldHelper::detect_hold_state(sendonly_sdp), HoldState::RemoteHold);

        let recvonly_sdp = "v=0\r\nm=audio 5004 RTP/AVP 0\r\na=recvonly\r\n";
        assert_eq!(SdpHoldHelper::detect_hold_state(recvonly_sdp), HoldState::LocalHold);

        let inactive_sdp = "v=0\r\nm=audio 5004 RTP/AVP 0\r\na=inactive\r\n";
        assert_eq!(SdpHoldHelper::detect_hold_state(inactive_sdp), HoldState::BothHold);
    }

    #[tokio::test]
    async fn test_is_on_hold() {
        let manager = HoldManager::new();

        assert!(!manager.is_on_hold("test-call").await);

        manager.hold_call("test-call").await.unwrap();
        assert!(manager.is_on_hold("test-call").await);

        manager.resume_call("test-call").await.unwrap();
        assert!(!manager.is_on_hold("test-call").await);
    }
}
