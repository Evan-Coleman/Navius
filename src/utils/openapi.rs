use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    sync::Arc,
};
use tracing::{error, info};

use crate::app::AppState;
use crate::error::{AppError, Result};

/// Handler to serve the user-provided OpenAPI spec file
pub async fn serve_user_openapi_spec(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Use the application-specific spec path
    let spec_path = state.config.openapi_spec_path();
    info!("Serving OpenAPI spec from: {}", spec_path);

    match read_file_to_string(&spec_path) {
        Ok(content) => {
            // Determine the content type based on file extension
            let content_type = if spec_path.ends_with(".yaml") || spec_path.ends_with(".yml") {
                "text/yaml"
            } else {
                "application/json" // Default to JSON
            };

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content,
            )
        }
        Err(e) => {
            error!("Failed to read OpenAPI spec file: {}", e);
            (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("OpenAPI spec file not found or not readable: {}", e),
            )
        }
    }
}

/// Helper function to ensure a directory exists
fn ensure_directory_exists(dir_path: &str) -> std::io::Result<()> {
    let path = Path::new(dir_path);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Helper function to read a file to string
fn read_file_to_string(file_path: &str) -> std::io::Result<String> {
    let path = Path::new(file_path);
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Helper function to validate an OpenAPI spec file
/// This is a simple validation that just checks if it's valid YAML or JSON with basic OpenAPI structure
fn is_valid_openapi(data: &[u8]) -> bool {
    // Try to parse as YAML first
    if let Ok(yaml_value) = serde_yaml::from_slice::<serde_yaml::Value>(data) {
        return has_openapi_structure(&yaml_value);
    }

    // If YAML parsing fails, try JSON
    if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(data) {
        return has_openapi_structure_json(&json_value);
    }

    false
}

/// Check if a YAML value has the basic OpenAPI structure
fn has_openapi_structure(value: &serde_yaml::Value) -> bool {
    if let serde_yaml::Value::Mapping(map) = value {
        // Check for required OpenAPI fields
        let has_openapi = map.contains_key(&serde_yaml::Value::String("openapi".to_string()));
        let has_info = map.contains_key(&serde_yaml::Value::String("info".to_string()));
        let has_paths = map.contains_key(&serde_yaml::Value::String("paths".to_string()));

        return has_openapi && has_info && has_paths;
    }

    false
}

/// Check if a JSON value has the basic OpenAPI structure
fn has_openapi_structure_json(value: &serde_json::Value) -> bool {
    if let serde_json::Value::Object(map) = value {
        // Check for required OpenAPI fields
        let has_openapi = map.contains_key("openapi");
        let has_info = map.contains_key("info");
        let has_paths = map.contains_key("paths");

        return has_openapi && has_info && has_paths;
    }

    false
}
