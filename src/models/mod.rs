use crate::cache::CacheStats;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Data structure for the data endpoint
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Data {
    pub fact: String,
    pub length: i32,
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
