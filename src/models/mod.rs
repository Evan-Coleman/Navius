use crate::cache::cache_manager::CacheStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// Export the loggable module
pub mod loggable;
pub use loggable::LoggableResponse;

/// Data structure for the data endpoint
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Data {
    pub fact: String,
    pub length: i32,
}

impl LoggableResponse for Data {
    fn log_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        // Add truncated fact if it's longer than 50 chars
        let fact_preview = if self.fact.len() > 50 {
            format!("{}...", &self.fact[..47])
        } else {
            self.fact.clone()
        };

        summary.insert("fact".to_string(), fact_preview);
        summary.insert("length".to_string(), self.length.to_string());

        summary
    }
}

/// Health check response structure
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub cache_enabled: bool,
    pub cache_stats: Option<CacheStats>,
}

impl LoggableResponse for HealthCheckResponse {
    fn log_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();
        summary.insert("status".to_string(), self.status.clone());
        summary.insert("uptime".to_string(), format!("{}s", self.uptime_seconds));
        summary
    }
}

/// Metrics response structure (for documentation purposes)
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MetricsResponse {
    #[schema(
        example = "# HELP http_requests_total Total HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total{path=\"/health\"} 1"
    )]
    pub metrics: String,
}

/// API response schema
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse {
    pub code: Option<i32>,
    pub r#type: Option<String>,
    pub message: Option<String>,
}

impl LoggableResponse for ApiResponse {
    fn log_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        if let Some(code) = self.code {
            summary.insert("code".to_string(), code.to_string());
        }

        if let Some(message) = &self.message {
            // Truncate long messages
            let msg_preview = if message.len() > 50 {
                format!("{}...", &message[..47])
            } else {
                message.clone()
            };
            summary.insert("message".to_string(), msg_preview);
        }

        summary
    }
}
