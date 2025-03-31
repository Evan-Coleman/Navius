use axum::routing::get;
use navius_http::server::HttpServer;
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status of the application
    pub status: String,
    /// Application name
    pub name: String,
    /// Application version
    pub version: String,
    /// Timestamp of the health check
    pub timestamp: String,
    /// Component statuses
    pub components: Vec<ComponentStatus>,
}

/// Status of an individual component
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentStatus {
    /// Component name
    pub name: String,
    /// Component status (UP, DOWN, UNKNOWN)
    pub status: String,
    /// Optional details about the component
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Handler for health check requests
async fn health_check() -> axum::Json<HealthResponse> {
    use chrono::Utc;
    
    let timestamp = Utc::now().to_rfc3339();
    
    // Create a list of component statuses
    let mut components = Vec::new();
    
    // Add application status
    components.push(ComponentStatus {
        name: "application".to_string(),
        status: "UP".to_string(),
        details: None,
    });
    
    // Add database status (if configured)
    #[cfg(feature = "database")]
    {
        components.push(ComponentStatus {
            name: "database".to_string(),
            status: "UP".to_string(),
            details: None,
        });
    }
    
    // Add cache status (if configured)
    #[cfg(feature = "cache")]
    {
        components.push(ComponentStatus {
            name: "cache".to_string(),
            status: "UP".to_string(),
            details: None,
        });
    }
    
    // Add auth status (if configured)
    #[cfg(feature = "entra-auth")]
    {
        components.push(ComponentStatus {
            name: "auth".to_string(),
            status: "UP".to_string(),
            details: None,
        });
    }
    
    let response = HealthResponse {
        status: "UP".to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp,
        components,
    };
    
    axum::Json(response)
}

/// Configure health check routes
pub fn configure_routes(server: HttpServer) -> HttpServer {
    server.route("/actuator/health", get(health_check))
} 