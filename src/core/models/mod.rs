use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// User-extensible model definitions
pub mod extensions;

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

/// API response schema
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub code: Option<i32>,
    pub r#type: Option<String>,
    pub message: Option<String>,
}
