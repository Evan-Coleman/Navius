mod metrics_service;
pub use metrics_service::*;

// Re-export core metrics functionality
pub use crate::core::metrics::*;

// This is a user-friendly metrics module where application-specific
// metrics can be defined. The implementation of the metrics system
// is in the core/metrics module.

// Example of defining application-specific metrics:
//
// pub fn record_user_login() {
//     metrics::counter!("user_logins_total", 1);
// }
//
// pub fn record_api_request_duration(endpoint: &str, duration_ms: f64) {
//     metrics::histogram!("api_request_duration_ms", duration_ms, "endpoint" => endpoint.to_string());
// }
