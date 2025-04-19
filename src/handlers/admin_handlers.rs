use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

/// Return the status of the application
pub async fn status() -> Json<serde_json::Value> {
    Json(json!({
        "status": "OK",
        "uptime": "10m",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get metrics by type
pub async fn get_metrics(Path(metric_type): Path<String>) -> impl IntoResponse {
    match metric_type.as_str() {
        "cpu" => (
            StatusCode::OK,
            Json(json!({
                "cpu_usage": "12%",
                "cores": 8,
                "load": [0.5, 0.7, 0.4]
            })),
        ),
        "memory" => (
            StatusCode::OK,
            Json(json!({
                "total": "16GB",
                "used": "4.2GB",
                "free": "11.8GB"
            })),
        ),
        "requests" => (
            StatusCode::OK,
            Json(json!({
                "total": 12500,
                "success_rate": "99.7%",
                "avg_response_time": "42ms"
            })),
        ),
        _ => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": format!("Metric type '{}' not found", metric_type)
            })),
        ),
    }
}
