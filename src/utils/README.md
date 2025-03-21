# Utility Extensions

This module provides a user-friendly interface to extend the core utility features of the application. The core utility implementation is in `src/core/utils`.

## Structure

- `api_logger.rs` - Extend API logging functionality
- `api_resource` - Extend API resource abstractions
- `openapi.rs` - Extend OpenAPI utilities

## Usage

### API Resource Extensions

The `api_resource` module provides a high-level abstraction for working with API resources. You can extend it with custom functionality:

```rust
// In src/utils/api_resource/mod.rs

/// Create a custom health check for API resources
pub fn create_custom_health_check<T: ApiResource>(
    api_url: String
) -> impl Fn(&Arc<AppState>) -> futures::future::BoxFuture<'static, DependencyStatus> + Send + Sync + 'static {
    move |state: &Arc<AppState>| {
        let api_url = api_url.clone();
        Box::pin(async move {
            // Custom health check implementation
            DependencyStatus {
                name: format!("{} API", T::api_name()),
                status: "healthy".to_string(),
                details: Some(serde_json::json!({ "url": api_url })),
            }
        })
    }
}
```

### API Logger Extensions

The `api_logger` module provides utilities for logging API operations. You can extend it with custom logging functionality:

```rust
// In src/utils/api_logger.rs

/// Log specialized API metrics
pub fn log_api_metrics(
    api_name: &str,
    endpoint: &str,
    duration_ms: u64,
    status_code: u16,
) {
    info!(
        "📊 API metrics - {}: endpoint={}, duration={}ms, status={}",
        api_name, endpoint, duration_ms, status_code
    );
    
    // Record metrics
    metrics::histogram!(
        "api_request_duration_ms", 
        duration_ms as f64,
        "api" => api_name.to_string(),
        "endpoint" => endpoint.to_string()
    );
}
```

### OpenAPI Extensions

The `openapi` module provides utilities for working with OpenAPI. You can extend it with custom functionality:

```rust
// In src/utils/openapi.rs

/// Serve a custom OpenAPI document
pub async fn serve_custom_openapi_doc(
    State(state): State<Arc<AppState>>,
    Path(doc_name): Path<String>
) -> impl IntoResponse {
    let base_path = state.config.openapi_spec_path();
    let parent_dir = PathBuf::from(&base_path).parent().unwrap_or(PathBuf::from("").as_path());
    let custom_path = parent_dir.join(format!("{}.yaml", doc_name));
    
    match std::fs::read_to_string(custom_path) {
        Ok(content) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/yaml")],
                content,
            )
        }
        Err(_) => {
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("Custom OpenAPI doc '{}' not found", doc_name),
            )
        }
    }
} 