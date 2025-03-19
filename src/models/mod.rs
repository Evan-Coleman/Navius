use crate::cache::cache_manager::CacheStats;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Data structure for the data endpoint
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Data {
    pub fact: String,
    pub length: i32,
}

/// Basic health check response structure
///
/// For simple Kubernetes/Load Balancer health checks
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub dependencies: Option<Vec<DependencyStatus>>,
}

/// Dependency status for detailed health checks
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyStatus {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, String>>,
}

/// Detailed health check response structure
///
/// For administrators and detailed monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub environment: String,
    pub dependencies: Vec<DependencyStatus>,
    pub cache_enabled: bool,
    pub cache_stats: Option<CacheStats>,
}

/// Actuator entry for key-value responses
#[derive(Debug, Serialize, Deserialize)]
pub struct ActuatorEntry {
    pub name: String,
    pub value: String,
}

/// Info endpoint response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub status: String,
    pub entries: Vec<ActuatorEntry>,
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
