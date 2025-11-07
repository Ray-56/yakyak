//! Call quality monitoring and analytics
//!
//! Provides comprehensive quality of service (QoS) monitoring for VoIP calls,
//! including MOS scoring, packet loss analysis, jitter tracking, and quality alerts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Quality of Service metrics for a call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosMetrics {
    /// Packet loss percentage (0.0 - 100.0)
    pub packet_loss_percent: f64,
    /// Jitter in milliseconds
    pub jitter_ms: f64,
    /// Round-trip time in milliseconds
    pub rtt_ms: f64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
    /// Packets lost
    pub packets_lost: u64,
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Mean Opinion Score (1.0 - 5.0)
    pub mos: f64,
    /// Codec used
    pub codec: String,
    /// Sample rate (Hz)
    pub sample_rate: u32,
}

impl Default for QosMetrics {
    fn default() -> Self {
        Self {
            packet_loss_percent: 0.0,
            jitter_ms: 0.0,
            rtt_ms: 0.0,
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            bytes_sent: 0,
            bytes_received: 0,
            mos: 4.5, // Excellent by default
            codec: "PCMU".to_string(),
            sample_rate: 8000,
        }
    }
}

impl QosMetrics {
    /// Calculate packet loss percentage
    pub fn calculate_packet_loss(&mut self) {
        let total = self.packets_sent + self.packets_lost;
        if total > 0 {
            self.packet_loss_percent = (self.packets_lost as f64 / total as f64) * 100.0;
        } else {
            self.packet_loss_percent = 0.0;
        }
    }

    /// Calculate MOS (Mean Opinion Score) using E-Model algorithm
    /// MOS ranges from 1.0 (poor) to 5.0 (excellent)
    pub fn calculate_mos(&mut self) {
        // E-Model: R = 93.2 - Id - Ie + A
        // Where:
        // - Id: delay impairment
        // - Ie: equipment impairment
        // - A: advantage factor

        // Delay impairment (based on RTT)
        let delay_ms = self.rtt_ms / 2.0; // One-way delay
        let id = if delay_ms < 177.3 {
            delay_ms / 177.3 * 25.0
        } else {
            25.0 + (delay_ms - 177.3) * 0.1
        };

        // Equipment impairment (based on codec and packet loss)
        let codec_ie = match self.codec.as_str() {
            "PCMU" | "PCMA" => 0.0,  // G.711: no impairment
            "G729" => 11.0,          // G.729: moderate impairment
            "GSM" => 20.0,           // GSM: higher impairment
            "OPUS" => 5.0,           // Opus: low impairment
            _ => 10.0,               // Default moderate impairment
        };

        // Packet loss impairment
        let loss_ie = self.packet_loss_percent * 2.5;

        // Jitter impairment (approximation)
        let jitter_ie = if self.jitter_ms > 20.0 {
            (self.jitter_ms - 20.0) * 0.5
        } else {
            0.0
        };

        let ie = codec_ie + loss_ie + jitter_ie;

        // Advantage factor (0 for most applications)
        let a = 0.0;

        // Calculate R-factor
        let r = 93.2 - id - ie + a;
        let r = r.max(0.0).min(100.0);

        // Convert R-factor to MOS
        // MOS = 1 + 0.035R + R(R-60)(100-R) * 7 * 10^-6
        self.mos = if r < 0.0 {
            1.0
        } else if r > 100.0 {
            4.5
        } else {
            1.0 + 0.035 * r + r * (r - 60.0) * (100.0 - r) * 7.0 * 1e-6
        };

        // Clamp MOS to valid range
        self.mos = self.mos.max(1.0).min(5.0);
    }

    /// Get quality rating from MOS score
    pub fn get_quality_rating(&self) -> QualityRating {
        if self.mos >= 4.3 {
            QualityRating::Excellent
        } else if self.mos >= 4.0 {
            QualityRating::Good
        } else if self.mos >= 3.6 {
            QualityRating::Fair
        } else if self.mos >= 3.1 {
            QualityRating::Poor
        } else {
            QualityRating::Bad
        }
    }

    /// Check if quality is acceptable (MOS >= 3.6)
    pub fn is_acceptable(&self) -> bool {
        self.mos >= 3.6
    }
}

/// Quality rating categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityRating {
    /// MOS >= 4.3 - Excellent quality
    Excellent,
    /// MOS >= 4.0 - Good quality
    Good,
    /// MOS >= 3.6 - Fair quality (acceptable)
    Fair,
    /// MOS >= 3.1 - Poor quality
    Poor,
    /// MOS < 3.1 - Bad quality
    Bad,
}

impl QualityRating {
    pub fn as_str(&self) -> &str {
        match self {
            QualityRating::Excellent => "Excellent",
            QualityRating::Good => "Good",
            QualityRating::Fair => "Fair",
            QualityRating::Poor => "Poor",
            QualityRating::Bad => "Bad",
        }
    }
}

/// Quality alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityAlert {
    /// High packet loss detected
    HighPacketLoss {
        call_id: String,
        loss_percent: f64,
        threshold: f64,
    },
    /// High jitter detected
    HighJitter {
        call_id: String,
        jitter_ms: f64,
        threshold: f64,
    },
    /// High latency detected
    HighLatency {
        call_id: String,
        rtt_ms: f64,
        threshold: f64,
    },
    /// Low MOS score
    LowMos {
        call_id: String,
        mos: f64,
        threshold: f64,
    },
    /// Quality degradation trend
    QualityDegradation {
        call_id: String,
        previous_mos: f64,
        current_mos: f64,
    },
}

/// Quality monitoring session for a single call
pub struct QualityMonitoringSession {
    pub call_id: String,
    pub started_at: DateTime<Utc>,
    pub metrics: QosMetrics,
    metrics_history: VecDeque<QosMetrics>,
    alert_thresholds: QualityThresholds,
    alerts_sent: Vec<QualityAlert>,
    max_history_size: usize,
}

impl QualityMonitoringSession {
    pub fn new(call_id: String, codec: String, sample_rate: u32) -> Self {
        let mut metrics = QosMetrics::default();
        metrics.codec = codec;
        metrics.sample_rate = sample_rate;

        Self {
            call_id,
            started_at: Utc::now(),
            metrics,
            metrics_history: VecDeque::new(),
            alert_thresholds: QualityThresholds::default(),
            alerts_sent: Vec::new(),
            max_history_size: 60, // Keep 60 data points
        }
    }

    /// Update metrics with new RTP statistics
    pub fn update_metrics(
        &mut self,
        packets_sent: u64,
        packets_received: u64,
        packets_lost: u64,
        jitter_ms: f64,
        rtt_ms: f64,
        bytes_sent: u64,
        bytes_received: u64,
    ) {
        self.metrics.packets_sent = packets_sent;
        self.metrics.packets_received = packets_received;
        self.metrics.packets_lost = packets_lost;
        self.metrics.jitter_ms = jitter_ms;
        self.metrics.rtt_ms = rtt_ms;
        self.metrics.bytes_sent = bytes_sent;
        self.metrics.bytes_received = bytes_received;

        // Calculate derived metrics
        self.metrics.calculate_packet_loss();
        self.metrics.calculate_mos();

        // Store in history
        self.metrics_history.push_back(self.metrics.clone());
        if self.metrics_history.len() > self.max_history_size {
            self.metrics_history.pop_front();
        }
    }

    /// Check for quality issues and generate alerts
    pub fn check_quality_alerts(&mut self) -> Vec<QualityAlert> {
        let mut new_alerts = Vec::new();

        // Check packet loss
        if self.metrics.packet_loss_percent > self.alert_thresholds.packet_loss_percent {
            let alert = QualityAlert::HighPacketLoss {
                call_id: self.call_id.clone(),
                loss_percent: self.metrics.packet_loss_percent,
                threshold: self.alert_thresholds.packet_loss_percent,
            };
            new_alerts.push(alert.clone());
            self.alerts_sent.push(alert);
        }

        // Check jitter
        if self.metrics.jitter_ms > self.alert_thresholds.jitter_ms {
            let alert = QualityAlert::HighJitter {
                call_id: self.call_id.clone(),
                jitter_ms: self.metrics.jitter_ms,
                threshold: self.alert_thresholds.jitter_ms,
            };
            new_alerts.push(alert.clone());
            self.alerts_sent.push(alert);
        }

        // Check latency
        if self.metrics.rtt_ms > self.alert_thresholds.rtt_ms {
            let alert = QualityAlert::HighLatency {
                call_id: self.call_id.clone(),
                rtt_ms: self.metrics.rtt_ms,
                threshold: self.alert_thresholds.rtt_ms,
            };
            new_alerts.push(alert.clone());
            self.alerts_sent.push(alert);
        }

        // Check MOS score
        if self.metrics.mos < self.alert_thresholds.min_mos {
            let alert = QualityAlert::LowMos {
                call_id: self.call_id.clone(),
                mos: self.metrics.mos,
                threshold: self.alert_thresholds.min_mos,
            };
            new_alerts.push(alert.clone());
            self.alerts_sent.push(alert);
        }

        // Check for degradation trend
        if let Some(previous) = self.metrics_history.get(self.metrics_history.len().saturating_sub(5)) {
            if previous.mos - self.metrics.mos > 0.5 {
                let alert = QualityAlert::QualityDegradation {
                    call_id: self.call_id.clone(),
                    previous_mos: previous.mos,
                    current_mos: self.metrics.mos,
                };
                new_alerts.push(alert.clone());
                self.alerts_sent.push(alert);
            }
        }

        new_alerts
    }

    /// Get average metrics over the session
    pub fn get_average_metrics(&self) -> QosMetrics {
        if self.metrics_history.is_empty() {
            return self.metrics.clone();
        }

        let count = self.metrics_history.len() as f64;
        let mut avg = QosMetrics::default();

        for m in &self.metrics_history {
            avg.packet_loss_percent += m.packet_loss_percent;
            avg.jitter_ms += m.jitter_ms;
            avg.rtt_ms += m.rtt_ms;
            avg.mos += m.mos;
        }

        avg.packet_loss_percent /= count;
        avg.jitter_ms /= count;
        avg.rtt_ms /= count;
        avg.mos /= count;

        // Use latest values for counters
        avg.packets_sent = self.metrics.packets_sent;
        avg.packets_received = self.metrics.packets_received;
        avg.packets_lost = self.metrics.packets_lost;
        avg.bytes_sent = self.metrics.bytes_sent;
        avg.bytes_received = self.metrics.bytes_received;
        avg.codec = self.metrics.codec.clone();
        avg.sample_rate = self.metrics.sample_rate;

        avg
    }

    pub fn get_duration(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }
}

/// Quality alert thresholds
#[derive(Debug, Clone)]
pub struct QualityThresholds {
    /// Maximum acceptable packet loss percentage
    pub packet_loss_percent: f64,
    /// Maximum acceptable jitter in milliseconds
    pub jitter_ms: f64,
    /// Maximum acceptable RTT in milliseconds
    pub rtt_ms: f64,
    /// Minimum acceptable MOS score
    pub min_mos: f64,
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            packet_loss_percent: 5.0,  // 5% packet loss
            jitter_ms: 30.0,            // 30ms jitter
            rtt_ms: 300.0,              // 300ms RTT
            min_mos: 3.6,               // Fair quality threshold
        }
    }
}

/// Call quality analytics manager
pub struct CallQualityManager {
    active_sessions: Arc<Mutex<HashMap<String, QualityMonitoringSession>>>,
    completed_sessions: Arc<Mutex<Vec<QualityReport>>>,
    alert_callback: Option<Arc<dyn Fn(QualityAlert) + Send + Sync>>,
}

impl CallQualityManager {
    pub fn new() -> Self {
        Self {
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            completed_sessions: Arc::new(Mutex::new(Vec::new())),
            alert_callback: None,
        }
    }

    /// Set alert callback function
    pub fn set_alert_callback<F>(&mut self, callback: F)
    where
        F: Fn(QualityAlert) + Send + Sync + 'static,
    {
        self.alert_callback = Some(Arc::new(callback));
    }

    /// Start monitoring a call
    pub fn start_monitoring(&self, call_id: String, codec: String, sample_rate: u32) {
        let session = QualityMonitoringSession::new(call_id.clone(), codec, sample_rate);
        let mut sessions = self.active_sessions.lock().unwrap();
        sessions.insert(call_id, session);
    }

    /// Stop monitoring a call and generate report
    pub fn stop_monitoring(&self, call_id: &str) -> Option<QualityReport> {
        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some(session) = sessions.remove(call_id) {
            let report = QualityReport::from_session(session);
            let mut completed = self.completed_sessions.lock().unwrap();
            completed.push(report.clone());
            Some(report)
        } else {
            None
        }
    }

    /// Update call quality metrics
    pub fn update_metrics(
        &self,
        call_id: &str,
        packets_sent: u64,
        packets_received: u64,
        packets_lost: u64,
        jitter_ms: f64,
        rtt_ms: f64,
        bytes_sent: u64,
        bytes_received: u64,
    ) {
        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(call_id) {
            session.update_metrics(
                packets_sent,
                packets_received,
                packets_lost,
                jitter_ms,
                rtt_ms,
                bytes_sent,
                bytes_received,
            );

            // Check for alerts
            let alerts = session.check_quality_alerts();
            if let Some(ref callback) = self.alert_callback {
                for alert in alerts {
                    callback(alert);
                }
            }
        }
    }

    /// Get current metrics for a call
    pub fn get_current_metrics(&self, call_id: &str) -> Option<QosMetrics> {
        let sessions = self.active_sessions.lock().unwrap();
        sessions.get(call_id).map(|s| s.metrics.clone())
    }

    /// Get all active monitoring sessions
    pub fn get_active_calls(&self) -> Vec<String> {
        let sessions = self.active_sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    /// Get quality statistics summary
    pub fn get_quality_summary(&self) -> QualitySummary {
        let sessions = self.active_sessions.lock().unwrap();
        let completed = self.completed_sessions.lock().unwrap();

        let mut summary = QualitySummary::default();
        summary.active_calls = sessions.len();

        // Calculate statistics from active calls
        for session in sessions.values() {
            let rating = session.metrics.get_quality_rating();
            match rating {
                QualityRating::Excellent => summary.excellent_count += 1,
                QualityRating::Good => summary.good_count += 1,
                QualityRating::Fair => summary.fair_count += 1,
                QualityRating::Poor => summary.poor_count += 1,
                QualityRating::Bad => summary.bad_count += 1,
            }

            summary.total_packet_loss += session.metrics.packet_loss_percent;
            summary.total_jitter += session.metrics.jitter_ms;
            summary.total_mos += session.metrics.mos;
        }

        // Add completed calls
        for report in completed.iter() {
            let rating = report.average_metrics.get_quality_rating();
            match rating {
                QualityRating::Excellent => summary.excellent_count += 1,
                QualityRating::Good => summary.good_count += 1,
                QualityRating::Fair => summary.fair_count += 1,
                QualityRating::Poor => summary.poor_count += 1,
                QualityRating::Bad => summary.bad_count += 1,
            }
        }

        summary.total_calls = sessions.len() + completed.len();

        // Calculate averages
        if summary.total_calls > 0 {
            let count = sessions.len() as f64;
            if count > 0.0 {
                summary.average_packet_loss = summary.total_packet_loss / count;
                summary.average_jitter = summary.total_jitter / count;
                summary.average_mos = summary.total_mos / count;
            }
        }

        summary
    }

    /// Clear completed session history
    pub fn clear_history(&self) {
        let mut completed = self.completed_sessions.lock().unwrap();
        completed.clear();
    }
}

/// Quality report for a completed call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    pub call_id: String,
    pub started_at: DateTime<Utc>,
    pub duration_seconds: i64,
    pub average_metrics: QosMetrics,
    pub quality_rating: QualityRating,
    pub alerts_count: usize,
}

impl QualityReport {
    fn from_session(session: QualityMonitoringSession) -> Self {
        let average_metrics = session.get_average_metrics();
        let quality_rating = average_metrics.get_quality_rating();
        let duration = session.get_duration();

        Self {
            call_id: session.call_id,
            started_at: session.started_at,
            duration_seconds: duration.num_seconds(),
            average_metrics,
            quality_rating,
            alerts_count: session.alerts_sent.len(),
        }
    }
}

/// Quality statistics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualitySummary {
    pub total_calls: usize,
    pub active_calls: usize,
    pub excellent_count: usize,
    pub good_count: usize,
    pub fair_count: usize,
    pub poor_count: usize,
    pub bad_count: usize,
    pub average_packet_loss: f64,
    pub average_jitter: f64,
    pub average_mos: f64,
    pub total_packet_loss: f64,
    pub total_jitter: f64,
    pub total_mos: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_metrics_default() {
        let metrics = QosMetrics::default();
        assert_eq!(metrics.packet_loss_percent, 0.0);
        assert_eq!(metrics.mos, 4.5);
    }

    #[test]
    fn test_packet_loss_calculation() {
        let mut metrics = QosMetrics::default();
        metrics.packets_sent = 100;
        metrics.packets_lost = 5;
        metrics.calculate_packet_loss();
        assert!((metrics.packet_loss_percent - 4.76).abs() < 0.1);
    }

    #[test]
    fn test_mos_calculation() {
        let mut metrics = QosMetrics::default();
        metrics.rtt_ms = 100.0;
        metrics.jitter_ms = 10.0;
        metrics.packet_loss_percent = 1.0;
        metrics.codec = "PCMU".to_string();
        metrics.calculate_mos();

        assert!(metrics.mos >= 1.0 && metrics.mos <= 5.0);
        assert!(metrics.is_acceptable());
    }

    #[test]
    fn test_quality_rating() {
        let mut metrics = QosMetrics::default();
        metrics.mos = 4.5;
        assert_eq!(metrics.get_quality_rating(), QualityRating::Excellent);

        metrics.mos = 4.1;
        assert_eq!(metrics.get_quality_rating(), QualityRating::Good);

        metrics.mos = 3.7;
        assert_eq!(metrics.get_quality_rating(), QualityRating::Fair);

        metrics.mos = 3.3;
        assert_eq!(metrics.get_quality_rating(), QualityRating::Poor);

        metrics.mos = 2.5;
        assert_eq!(metrics.get_quality_rating(), QualityRating::Bad);
    }

    #[test]
    fn test_quality_monitoring_session() {
        let mut session = QualityMonitoringSession::new(
            "test-call-123".to_string(),
            "PCMU".to_string(),
            8000,
        );

        session.update_metrics(100, 95, 5, 15.0, 120.0, 16000, 15200);

        assert!(session.metrics.packet_loss_percent > 0.0);
        assert!(session.metrics.mos > 0.0);
        assert_eq!(session.metrics_history.len(), 1);
    }

    #[test]
    fn test_quality_alerts() {
        let mut session = QualityMonitoringSession::new(
            "test-call-456".to_string(),
            "PCMU".to_string(),
            8000,
        );

        // Set high packet loss to trigger alert
        session.update_metrics(100, 80, 20, 15.0, 120.0, 16000, 12800);

        let alerts = session.check_quality_alerts();
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_call_quality_manager() {
        let manager = CallQualityManager::new();

        manager.start_monitoring("call-789".to_string(), "PCMU".to_string(), 8000);
        manager.update_metrics("call-789", 100, 95, 5, 15.0, 120.0, 16000, 15200);

        let metrics = manager.get_current_metrics("call-789");
        assert!(metrics.is_some());

        let report = manager.stop_monitoring("call-789");
        assert!(report.is_some());
    }

    #[test]
    fn test_quality_summary() {
        let manager = CallQualityManager::new();

        manager.start_monitoring("call-1".to_string(), "PCMU".to_string(), 8000);
        manager.start_monitoring("call-2".to_string(), "PCMU".to_string(), 8000);

        let summary = manager.get_quality_summary();
        assert_eq!(summary.active_calls, 2);
    }

    #[test]
    fn test_average_metrics() {
        let mut session = QualityMonitoringSession::new(
            "test-call".to_string(),
            "PCMU".to_string(),
            8000,
        );

        // Add multiple metric updates
        for i in 0..5 {
            session.update_metrics(
                100 + i * 10,
                95 + i * 10,
                5,
                15.0 + i as f64,
                120.0,
                16000,
                15200,
            );
        }

        let avg = session.get_average_metrics();
        assert!(avg.jitter_ms > 15.0); // Should be averaged
    }
}
