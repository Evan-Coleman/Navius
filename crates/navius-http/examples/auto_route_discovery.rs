use axum::{Json, extract::State};
use navius_http::{
    WebPlugin,
    server::route_discovery::{RouteDiscoveryConfig, RouteRegistry},
};
use navius_macros::{navius_router, nest, route};
use serde_json::json;
use tracing::info;

// Define the API module with routes
#[nest(prefix = "/api/v1")]
mod api {
    use super::*;

    // Define route handlers with route attributes
    #[route(path = "/hello", method = "GET")]
    async fn hello_world() -> Json<serde_json::Value> {
        Json(json!({ "message": "Hello, World!" }))
    }

    #[route(path = "/echo/:message", method = "GET")]
    async fn echo(axum::extract::Path(message): axum::extract::Path<String>) -> String {
        format!("Echo: {}", message)
    }
}

// Define the admin module with routes
#[nest(prefix = "/admin")]
mod admin {
    use super::*;

    // Define route handlers with route attributes
    #[route(path = "/status", method = "GET")]
    async fn status() -> Json<serde_json::Value> {
        Json(json!({
            "status": "OK",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": "10m"
        }))
    }
}

// Define a route registration function that uses the navius_router macro
#[navius_router(modules = [self::api, self::admin])]
fn register_routes(registry: &mut RouteRegistry) {
    // The macro will generate code to register all discovered routes
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting server with automatic route discovery");

    // Create a route registry
    let mut registry = RouteRegistry::new();

    // Register all routes
    register_routes(&mut registry);

    // Build the router
    let router = registry.build_router();

    // Using port 3003 to avoid conflicts
    let port = 3003;

    // Create a web plugin with the discovered routes
    let web_plugin = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(port)
        .with_router(router);

    // Alternatively, you can use the automatic route discovery feature directly:
    /*
    let web_plugin = WebPlugin::new()
        .with_host("127.0.0.1")
        .with_port(port)
        .with_route_discovery_config(
            RouteDiscoveryConfig::new()
                .with_module_paths(vec!["api", "admin"])
        );
    */

    info!("Server started at http://127.0.0.1:{}", port);
    info!("Try accessing:");
    info!("  - http://127.0.0.1:{}/api/v1/hello", port);
    info!("  - http://127.0.0.1:{}/api/v1/echo/hello", port);
    info!("  - http://127.0.0.1:{}/admin/status", port);
    info!("Press Ctrl+C to stop the server");

    // In a real application, this would be provided by the App builder
    let app = navius_core::di::Application::builder().build();
    let app_state = navius_http::AppState {
        app: std::sync::Arc::new(app),
    };

    // Start the server directly
    web_plugin.start_server(app_state).await?;

    Ok(())
}
