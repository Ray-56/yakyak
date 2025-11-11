/// Enhanced monitoring and metrics collection
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::info;

use super::user_handler::AppState;

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub active_calls: usize,
    pub registered_users: usize,
    pub total_calls_today: u64,
    pub total_calls_all_time: u64,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub disk_usage_percent: f64,
}

/// Call metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallMetrics {
    pub total_calls: u64,
    pub active_calls: usize,
    pub completed_calls: u64,
    pub failed_calls: u64,
    pub average_call_duration_seconds: f64,
    pub max_concurrent_calls: usize,
}

/// Registration metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationMetrics {
    pub total_registrations: u64,
    pub active_registrations: usize,
    pub registration_failures: u64,
    pub average_registration_duration_ms: f64,
}

/// Authentication metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMetrics {
    pub total_auth_attempts: u64,
    pub successful_auths: u64,
    pub failed_auths: u64,
    pub locked_out_ips: usize,
    pub rate_limited_requests: u64,
}

/// Media metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaMetrics {
    pub active_rtp_sessions: usize,
    pub total_packets_sent: u64,
    pub total_packets_received: u64,
    pub packet_loss_percent: f64,
    pub average_jitter_ms: f64,
}

/// Complete system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub status: String, // "healthy", "degraded", "unhealthy"
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub metrics: SystemMetrics,
    pub call_metrics: CallMetrics,
    pub registration_metrics: RegistrationMetrics,
    pub auth_metrics: AuthMetrics,
    pub media_metrics: MediaMetrics,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl SystemHealth {
    pub fn new() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            status: "healthy".to_string(),
            timestamp: now,
            uptime_seconds: 0,
            metrics: SystemMetrics {
                timestamp: now,
                uptime_seconds: 0,
                active_calls: 0,
                registered_users: 0,
                total_calls_today: 0,
                total_calls_all_time: 0,
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0,
                disk_usage_percent: 0.0,
            },
            call_metrics: CallMetrics {
                total_calls: 0,
                active_calls: 0,
                completed_calls: 0,
                failed_calls: 0,
                average_call_duration_seconds: 0.0,
                max_concurrent_calls: 0,
            },
            registration_metrics: RegistrationMetrics {
                total_registrations: 0,
                active_registrations: 0,
                registration_failures: 0,
                average_registration_duration_ms: 0.0,
            },
            auth_metrics: AuthMetrics {
                total_auth_attempts: 0,
                successful_auths: 0,
                failed_auths: 0,
                locked_out_ips: 0,
                rate_limited_requests: 0,
            },
            media_metrics: MediaMetrics {
                active_rtp_sessions: 0,
                total_packets_sent: 0,
                total_packets_received: 0,
                packet_loss_percent: 0.0,
                average_jitter_ms: 0.0,
            },
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Check overall health status
    pub fn check_health(&mut self) {
        self.warnings.clear();
        self.errors.clear();

        // Check for high CPU usage
        if self.metrics.cpu_usage_percent > 80.0 {
            self.warnings.push(format!("High CPU usage: {:.1}%", self.metrics.cpu_usage_percent));
        }
        if self.metrics.cpu_usage_percent > 95.0 {
            self.errors.push(format!("Critical CPU usage: {:.1}%", self.metrics.cpu_usage_percent));
        }

        // Check for high memory usage
        if self.metrics.memory_usage_mb > 1000 {
            self.warnings.push(format!("High memory usage: {} MB", self.metrics.memory_usage_mb));
        }

        // Check for high disk usage
        if self.metrics.disk_usage_percent > 80.0 {
            self.warnings.push(format!("High disk usage: {:.1}%", self.metrics.disk_usage_percent));
        }
        if self.metrics.disk_usage_percent > 95.0 {
            self.errors.push(format!("Critical disk usage: {:.1}%", self.metrics.disk_usage_percent));
        }

        // Check call failure rate
        if self.call_metrics.total_calls > 0 {
            let failure_rate = (self.call_metrics.failed_calls as f64 / self.call_metrics.total_calls as f64) * 100.0;
            if failure_rate > 10.0 {
                self.warnings.push(format!("High call failure rate: {:.1}%", failure_rate));
            }
            if failure_rate > 25.0 {
                self.errors.push(format!("Critical call failure rate: {:.1}%", failure_rate));
            }
        }

        // Check authentication failure rate
        if self.auth_metrics.total_auth_attempts > 0 {
            let auth_failure_rate = (self.auth_metrics.failed_auths as f64 / self.auth_metrics.total_auth_attempts as f64) * 100.0;
            if auth_failure_rate > 20.0 {
                self.warnings.push(format!("High auth failure rate: {:.1}%", auth_failure_rate));
            }
        }

        // Check for locked out IPs
        if self.auth_metrics.locked_out_ips > 10 {
            self.warnings.push(format!("Many locked out IPs: {}", self.auth_metrics.locked_out_ips));
        }

        // Check packet loss
        if self.media_metrics.packet_loss_percent > 5.0 {
            self.warnings.push(format!("High packet loss: {:.1}%", self.media_metrics.packet_loss_percent));
        }
        if self.media_metrics.packet_loss_percent > 10.0 {
            self.errors.push(format!("Critical packet loss: {:.1}%", self.media_metrics.packet_loss_percent));
        }

        // Update overall status
        self.status = if !self.errors.is_empty() {
            "unhealthy".to_string()
        } else if !self.warnings.is_empty() {
            "degraded".to_string()
        } else {
            "healthy".to_string()
        };
    }
}

impl Default for SystemHealth {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics collector
pub struct MetricsCollector {
    health: Arc<RwLock<SystemHealth>>,
    start_time: SystemTime,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            health: Arc::new(RwLock::new(SystemHealth::new())),
            start_time: SystemTime::now(),
        }
    }

    /// Get current system health
    pub async fn get_health(&self) -> SystemHealth {
        let mut health = self.health.write().await;

        // Update uptime
        let uptime = self.start_time.elapsed().unwrap_or(Duration::from_secs(0));
        health.uptime_seconds = uptime.as_secs();
        health.metrics.uptime_seconds = uptime.as_secs();

        // Update timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        health.timestamp = now;
        health.metrics.timestamp = now;

        // Check health status
        health.check_health();

        health.clone()
    }

    /// Increment call counter
    pub async fn increment_calls(&self, success: bool) {
        let mut health = self.health.write().await;
        health.call_metrics.total_calls += 1;
        health.metrics.total_calls_all_time += 1;

        if success {
            health.call_metrics.completed_calls += 1;
        } else {
            health.call_metrics.failed_calls += 1;
        }
    }

    /// Update active calls
    pub async fn update_active_calls(&self, count: usize) {
        let mut health = self.health.write().await;
        health.call_metrics.active_calls = count;
        health.metrics.active_calls = count;

        if count > health.call_metrics.max_concurrent_calls {
            health.call_metrics.max_concurrent_calls = count;
        }
    }

    /// Update registered users
    pub async fn update_registered_users(&self, count: usize) {
        let mut health = self.health.write().await;
        health.metrics.registered_users = count;
        health.registration_metrics.active_registrations = count;
    }

    /// Record authentication attempt
    pub async fn record_auth_attempt(&self, success: bool) {
        let mut health = self.health.write().await;
        health.auth_metrics.total_auth_attempts += 1;

        if success {
            health.auth_metrics.successful_auths += 1;
        } else {
            health.auth_metrics.failed_auths += 1;
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Get detailed system health
pub async fn get_system_health(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("Fetching system health");

    let mut health = SystemHealth::new();

    // Get active calls
    if let Some(ref call_router) = state.call_router {
        let active_calls = call_router.active_call_count().await;
        health.metrics.active_calls = active_calls;
        health.call_metrics.active_calls = active_calls;
    }

    // Get registered users
    if let Some(ref registrar) = state.registrar {
        let registered_count = registrar.get_registration_count().await;
        health.metrics.registered_users = registered_count;
        health.registration_metrics.active_registrations = registered_count;
    }

    // Get CDR statistics
    if let Some(ref cdr_repo) = state.cdr_repository {
        // Count total calls
        if let Ok(total) = cdr_repo.count(Default::default()).await {
            health.call_metrics.total_calls = total as u64;
            health.metrics.total_calls_all_time = total as u64;
        }

        // Get today's calls
        let today_start = chrono::Utc::now()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let mut today_filter = crate::domain::cdr::CdrFilters::default();
        today_filter.start_time_from = Some(today_start);

        if let Ok(today_count) = cdr_repo.count(today_filter).await {
            health.metrics.total_calls_today = today_count as u64;
        }

        // Count completed/failed calls
        let mut completed_filter = crate::domain::cdr::CdrFilters::default();
        completed_filter.status = Some(crate::domain::cdr::CallStatus::Completed);
        if let Ok(completed) = cdr_repo.count(completed_filter).await {
            health.call_metrics.completed_calls = completed as u64;
        }

        let mut failed_filter = crate::domain::cdr::CdrFilters::default();
        failed_filter.status = Some(crate::domain::cdr::CallStatus::Failed);
        if let Ok(failed) = cdr_repo.count(failed_filter).await {
            health.call_metrics.failed_calls = failed as u64;
        }
    }

    // Check health status
    health.check_health();

    (StatusCode::OK, Json(health)).into_response()
}

/// Get Prometheus metrics in text format
pub async fn get_prometheus_metrics(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("Fetching Prometheus metrics");

    let mut active_calls = 0;
    let mut registered_users = 0;
    let mut total_calls = 0;
    let mut completed_calls = 0;
    let mut failed_calls = 0;

    // Get active calls
    if let Some(ref call_router) = state.call_router {
        active_calls = call_router.active_call_count().await;
    }

    // Get registered users
    if let Some(ref registrar) = state.registrar {
        registered_users = registrar.get_registration_count().await;
    }

    // Get call statistics
    if let Some(ref cdr_repo) = state.cdr_repository {
        if let Ok(total) = cdr_repo.count(Default::default()).await {
            total_calls = total;
        }

        let mut completed_filter = crate::domain::cdr::CdrFilters::default();
        completed_filter.status = Some(crate::domain::cdr::CallStatus::Completed);
        if let Ok(completed) = cdr_repo.count(completed_filter).await {
            completed_calls = completed;
        }

        let mut failed_filter = crate::domain::cdr::CdrFilters::default();
        failed_filter.status = Some(crate::domain::cdr::CallStatus::Failed);
        if let Ok(failed) = cdr_repo.count(failed_filter).await {
            failed_calls = failed;
        }
    }

    // Format Prometheus metrics
    let metrics = format!(
        "# HELP yakyak_active_calls Number of active calls\n\
         # TYPE yakyak_active_calls gauge\n\
         yakyak_active_calls {}\n\
         \n\
         # HELP yakyak_registered_users Number of registered SIP endpoints\n\
         # TYPE yakyak_registered_users gauge\n\
         yakyak_registered_users {}\n\
         \n\
         # HELP yakyak_total_calls Total calls processed\n\
         # TYPE yakyak_total_calls counter\n\
         yakyak_total_calls {}\n\
         \n\
         # HELP yakyak_completed_calls Total completed calls\n\
         # TYPE yakyak_completed_calls counter\n\
         yakyak_completed_calls {}\n\
         \n\
         # HELP yakyak_failed_calls Total failed calls\n\
         # TYPE yakyak_failed_calls counter\n\
         yakyak_failed_calls {}\n\
         \n\
         # HELP yakyak_call_duration_seconds Call duration histogram\n\
         # TYPE yakyak_call_duration_seconds histogram\n\
         yakyak_call_duration_seconds_bucket{{le=\"30\"}} 0\n\
         yakyak_call_duration_seconds_bucket{{le=\"60\"}} 0\n\
         yakyak_call_duration_seconds_bucket{{le=\"120\"}} 0\n\
         yakyak_call_duration_seconds_bucket{{le=\"300\"}} 0\n\
         yakyak_call_duration_seconds_bucket{{le=\"+Inf\"}} 0\n\
         yakyak_call_duration_seconds_sum 0\n\
         yakyak_call_duration_seconds_count 0\n",
        active_calls,
        registered_users,
        total_calls,
        completed_calls,
        failed_calls
    );

    (StatusCode::OK, metrics).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_health_creation() {
        let health = SystemHealth::new();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.warnings.len(), 0);
        assert_eq!(health.errors.len(), 0);
    }

    #[test]
    fn test_health_check_warnings() {
        let mut health = SystemHealth::new();
        health.metrics.cpu_usage_percent = 85.0;
        health.metrics.memory_usage_mb = 1500;
        health.check_health();

        assert_eq!(health.status, "degraded");
        assert!(health.warnings.len() > 0);
    }

    #[test]
    fn test_health_check_errors() {
        let mut health = SystemHealth::new();
        health.metrics.cpu_usage_percent = 96.0;
        health.check_health();

        assert_eq!(health.status, "unhealthy");
        assert!(health.errors.len() > 0);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        collector.increment_calls(true).await;
        collector.increment_calls(false).await;
        collector.update_active_calls(5).await;
        collector.update_registered_users(10).await;

        let health = collector.get_health().await;
        assert_eq!(health.call_metrics.total_calls, 2);
        assert_eq!(health.call_metrics.completed_calls, 1);
        assert_eq!(health.call_metrics.failed_calls, 1);
        assert_eq!(health.metrics.active_calls, 5);
        assert_eq!(health.metrics.registered_users, 10);
    }
}
