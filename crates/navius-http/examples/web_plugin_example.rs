use axum::{Json, Router, routing::get};
use navius_http::WebPlugin;
use serde_json::json;
use tracing::info;

/// A simple handler that returns a hello world message
async fn hello_world() -> Json<serde_json::Value> {
    Json(json!({ "message": "Hello, World!" }))
}

/// A simple handler that returns some configuration information
async fn server_info() -> Json<serde_json::Value> {
    Json(json!({
        "server": "Navius WebPlugin",
        "version": env!("CARGO_PKG_VERSION"),
        "environment": "development"
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create API router with our routes
    let api_router = Router::new()
        .route("/hello", get(hello_world))
        .route("/info", get(server_info));

    // Using port 3002 to avoid conflicts
    let port = 3002;

    // Create a web plugin with custom settings
    let web_plugin = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(port)
        .with_router(Router::new().nest("/api/v1", api_router));

    info!("Starting server on http://127.0.0.1:{}", port);
    info!("Try accessing: http://127.0.0.1:{}/api/v1/hello", port);
    info!("             : http://127.0.0.1:{}/api/v1/info", port);
    info!("Press Ctrl+C to stop the server");

    // In a real application, this would be provided by the App builder
    let app = navius_core::di::Application::builder().build();
    let app_state = navius_http::AppState {
        app: std::sync::Arc::new(app),
    };

    // Start the server directly (in a real app, we'd use App::new().with_plugin(web_plugin).run())
    web_plugin.start_server(app_state).await?;

    Ok(())
}
