use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::cache::CacheStats;

/// Schema for pet model for OpenAPI documentation
#[derive(ToSchema)]
#[schema(as = pet::Pet)]
pub struct PetSchema {
    #[schema(example = 10)]
    pub id: Option<i64>,
    #[schema(example = "doggie")]
    pub name: String,
    #[schema(nullable, example = json!({"id": 1, "name": "Dogs"}))]
    pub category: Option<Box<CategorySchema>>,
    #[schema(example = json!(["url1", "url2"]))]
    pub photo_urls: Vec<String>,
    #[schema(nullable, example = json!([{"id": 1, "name": "tag1"}]))]
    pub tags: Option<Vec<TagSchema>>,
    #[schema(nullable, example = "available")]
    pub status: Option<StatusSchema>,
}

/// Schema for pet category for OpenAPI documentation
#[derive(ToSchema)]
#[schema(as = pet::Category)]
pub struct CategorySchema {
    #[schema(example = 1)]
    pub id: Option<i64>,
    #[schema(example = "Dogs")]
    pub name: Option<String>,
}

/// Schema for pet tag for OpenAPI documentation
#[derive(ToSchema)]
#[schema(as = pet::Tag)]
pub struct TagSchema {
    #[schema(example = 1)]
    pub id: Option<i64>,
    #[schema(example = "tag1")]
    pub name: Option<String>,
}

/// Schema for pet status for OpenAPI documentation
#[derive(ToSchema)]
pub enum StatusSchema {
    Available,
    Pending,
    Sold,
}

/// Data structure for the data endpoint
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Data {
    pub fact: String,
    pub length: i32,
}

/// API response schema
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse {
    pub code: Option<i32>,
    pub r#type: Option<String>,
    pub message: Option<String>,
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

/// Helper to convert Status enum to string
pub fn status_to_string(status: &StatusSchema) -> String {
    match status {
        StatusSchema::Available => "available".to_string(),
        StatusSchema::Pending => "pending".to_string(),
        StatusSchema::Sold => "sold".to_string(),
    }
}
