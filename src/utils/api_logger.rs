//! # API Logger
//!
//! This module provides user-extensible logging utilities for API operations.
//! The core implementation is in the core/utils/api_logger module.

// Re-export core functionality
pub use crate::core::utils::api_logger::*;

// Add your custom API logging utilities below
//
// Example:
//
// /// Log specialized API metrics
// pub fn log_api_metrics(
//     api_name: &str,
//     endpoint: &str,
//     duration_ms: u64,
//     status_code: u16,
// ) {
//     info!(
//         "📊 API metrics - {}: endpoint={}, duration={}ms, status={}",
//         api_name, endpoint, duration_ms, status_code
//     );
//
//     // Record metrics
//     metrics::histogram!(
//         "api_request_duration_ms",
//         duration_ms as f64,
//         "api" => api_name.to_string(),
//         "endpoint" => endpoint.to_string()
//     );
//
//     metrics::counter!(
//         "api_requests_total",
//         1,
//         "api" => api_name.to_string(),
//         "endpoint" => endpoint.to_string(),
//         "status" => status_code.to_string()
//     );
// }
