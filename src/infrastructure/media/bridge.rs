//! Media Bridge for Call Forwarding
//!
//! Bridges media between two endpoints (caller and callee)

use super::stream::MediaStream;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Media Bridge
///
/// Connects two media streams and forwards packets between them
pub struct MediaBridge {
    /// Stream for leg A (caller)
    leg_a: Arc<MediaStream>,
    /// Stream for leg B (callee)
    leg_b: Arc<MediaStream>,
    /// Bridge active flag
    active: Arc<RwLock<bool>>,
}

impl MediaBridge {
    /// Create a new media bridge
    pub fn new(leg_a: Arc<MediaStream>, leg_b: Arc<MediaStream>) -> Self {
        info!(
            "Creating media bridge: A(ssrc={:08x}) <-> B(ssrc={:08x})",
            leg_a.ssrc(),
            leg_b.ssrc()
        );

        Self {
            leg_a,
            leg_b,
            active: Arc::new(RwLock::new(false)),
        }
    }

    /// Start bridging
    pub async fn start(&self) -> Result<(), std::io::Error> {
        info!("Starting media bridge");
        *self.active.write().await = true;

        // Start both media streams
        self.leg_a.start().await?;
        self.leg_b.start().await?;

        // In a real implementation, we would:
        // 1. Receive RTP packets from leg A
        // 2. Optionally transcode if codecs differ
        // 3. Forward to leg B (and vice versa)
        //
        // For now, the MediaStream handles its own RTP reception
        // and this bridge just manages the lifecycle

        info!("Media bridge active");
        Ok(())
    }

    /// Stop bridging
    pub async fn stop(&self) {
        info!("Stopping media bridge");
        *self.active.write().await = false;

        self.leg_a.stop().await;
        self.leg_b.stop().await;

        info!("Media bridge stopped");
    }

    /// Check if bridge is active
    pub async fn is_active(&self) -> bool {
        *self.active.read().await
    }
}

/// Media Bridge Manager
///
/// Manages multiple media bridges for active calls
pub struct MediaBridgeManager {
    bridges: Arc<RwLock<std::collections::HashMap<String, Arc<MediaBridge>>>>,
}

impl MediaBridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create and start a new bridge
    pub async fn create_bridge(
        &self,
        call_id: String,
        leg_a: Arc<MediaStream>,
        leg_b: Arc<MediaStream>,
    ) -> Result<Arc<MediaBridge>, std::io::Error> {
        let bridge = Arc::new(MediaBridge::new(leg_a, leg_b));
        bridge.start().await?;

        let mut bridges = self.bridges.write().await;
        bridges.insert(call_id.clone(), bridge.clone());

        info!("Created bridge for call: {}", call_id);
        Ok(bridge)
    }

    /// Remove and stop a bridge
    pub async fn remove_bridge(&self, call_id: &str) {
        let mut bridges = self.bridges.write().await;

        if let Some(bridge) = bridges.remove(call_id) {
            bridge.stop().await;
            info!("Removed bridge for call: {}", call_id);
        } else {
            warn!("No bridge found for call: {}", call_id);
        }
    }

    /// Get a bridge
    pub async fn get_bridge(&self, call_id: &str) -> Option<Arc<MediaBridge>> {
        let bridges = self.bridges.read().await;
        bridges.get(call_id).cloned()
    }

    /// Get active bridge count
    pub async fn active_count(&self) -> usize {
        self.bridges.read().await.len()
    }
}

impl Default for MediaBridgeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        let stream_a = Arc::new(MediaStream::new(10010, 0, 8000).await.unwrap());
        let stream_b = Arc::new(MediaStream::new(10020, 0, 8000).await.unwrap());

        let bridge = MediaBridge::new(stream_a, stream_b);
        assert!(!bridge.is_active().await);
    }

    #[tokio::test]
    async fn test_bridge_manager() {
        let manager = MediaBridgeManager::new();

        let stream_a = Arc::new(MediaStream::new(10030, 0, 8000).await.unwrap());
        let stream_b = Arc::new(MediaStream::new(10040, 0, 8000).await.unwrap());

        manager
            .create_bridge("test-call-1".to_string(), stream_a, stream_b)
            .await
            .unwrap();

        assert_eq!(manager.active_count().await, 1);

        manager.remove_bridge("test-call-1").await;
        assert_eq!(manager.active_count().await, 0);
    }
}
