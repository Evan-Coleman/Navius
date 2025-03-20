use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse},
};
use std::sync::Arc;
use tracing::info;

use crate::app::AppState;

/// Serves the Swagger UI HTML for the API documentation
pub async fn swagger_ui_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Serving Swagger UI documentation");

    // Get the OpenAPI spec URL from configuration
    let spec_url = state.config.openapi_spec_url();
    info!("Using OpenAPI spec URL from config: {}", spec_url);

    // Create a simple HTML page with Swagger UI
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust Backend API Documentation</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        window.onload = function() {{
            SwaggerUIBundle({{
                url: "{spec_url}",
                dom_id: '#swagger-ui',
                deepLinking: true,
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIBundle.SwaggerUIStandalonePreset
                ],
                layout: "BaseLayout",
                syntaxHighlight: {{
                    activated: true,
                    theme: "agate"
                }}
            }});
        }};
    </script>
</body>
</html>"#
    );

    Html(html)
}

/// Serves the OpenAPI specification file
pub async fn openapi_spec_handler(
    State(state): State<Arc<AppState>>,
    Path(file): Path<String>,
) -> impl IntoResponse {
    // Get the configured spec file from config
    let configured_spec_file = state.config.openapi.spec_file.clone();

    // Security check: Only allow access to the configured spec file
    if file != configured_spec_file {
        info!("Attempted access to unauthorized file: {}", file);
        return (
            StatusCode::FORBIDDEN,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Access denied. Only the OpenAPI specification file is accessible."),
        );
    }

    info!("Serving OpenAPI specification file: {}", file);

    // Get the path to the OpenAPI spec file from config
    let spec_path = state.config.openapi_spec_path();

    // Read the file
    match std::fs::read_to_string(&spec_path) {
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
        Err(e) => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("OpenAPI spec file not found: {}", e),
        ),
    }
}
