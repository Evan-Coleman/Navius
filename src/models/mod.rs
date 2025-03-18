use crate::cache::cache_manager::CacheStats;
use serde::{Deserialize, Serialize};

/// Data structure for the data endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub fact: String,
    pub length: i32,
}

/// Health check response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub cache_enabled: bool,
    pub cache_stats: Option<CacheStats>,
}

/// Metrics response structure (for documentation purposes)
#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub metrics: String,
}

/// API response schema
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub code: Option<i32>,
    pub r#type: Option<String>,
    pub message: Option<String>,
}
