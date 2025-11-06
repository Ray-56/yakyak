//! Prometheus metrics handler

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

/// Initialize the Prometheus metrics exporter
pub fn init_metrics() -> PrometheusHandle {
    let builder = PrometheusBuilder::new();

    let handle = builder
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .unwrap()
        .install_recorder()
        .unwrap();

    // Describe metrics
    describe_counter!(
        "http_requests_total",
        "Total number of HTTP requests received"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    );
    describe_gauge!(
        "sip_active_calls",
        "Number of currently active SIP calls"
    );
    describe_gauge!(
        "sip_registered_users",
        "Number of currently registered SIP users"
    );
    describe_counter!(
        "sip_registrations_total",
        "Total number of SIP registrations"
    );
    describe_counter!(
        "sip_calls_total",
        "Total number of SIP calls initiated"
    );
    describe_counter!(
        "sip_calls_completed",
        "Total number of SIP calls completed successfully"
    );
    describe_counter!(
        "sip_calls_failed",
        "Total number of SIP calls that failed"
    );

    handle
}

/// HTTP metrics handler
pub async fn metrics_handler(
    axum::extract::State(prometheus_handle): axum::extract::State<PrometheusHandle>,
) -> Response {
    let metrics = prometheus_handle.render();
    (StatusCode::OK, metrics).into_response()
}

/// Record HTTP request
pub fn record_http_request(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    counter!("http_requests_total", "method" => method.to_string(), "path" => path.to_string(), "status" => status.to_string())
        .increment(1);
    histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string()
    )
    .record(duration.as_secs_f64());
}

/// Update active calls gauge
pub fn update_active_calls(count: usize) {
    gauge!("sip_active_calls").set(count as f64);
}

/// Update registered users gauge
pub fn update_registered_users(count: usize) {
    gauge!("sip_registered_users").set(count as f64);
}

/// Record SIP registration
pub fn record_sip_registration(success: bool) {
    counter!("sip_registrations_total", "success" => success.to_string()).increment(1);
}

/// Record SIP call initiation
pub fn record_sip_call_initiated() {
    counter!("sip_calls_total").increment(1);
}

/// Record SIP call completion
pub fn record_sip_call_completed() {
    counter!("sip_calls_completed").increment(1);
}

/// Record SIP call failure
pub fn record_sip_call_failed(reason: &str) {
    counter!("sip_calls_failed", "reason" => reason.to_string()).increment(1);
}

/// Timer for measuring durations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
