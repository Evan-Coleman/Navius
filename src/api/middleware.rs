use navius_http::server::HttpServer;
use std::time::Duration;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// Configure common middleware for the application
pub fn configure_middleware(server: HttpServer) -> HttpServer {
    // Add tracing for all requests
    let server = server.layer(TraceLayer::new_for_http());

    // Add compression
    let server = server.layer(CompressionLayer::new());

    // Add CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let server = server.layer(cors);

    // Add timeout
    let server = server.layer(TimeoutLayer::new(Duration::from_secs(30)));

    server
}
