//! HTTP middleware components for the Navius server.
//!
//! This module provides various middleware components that can be used with the Navius HTTP server,
//! including CORS, request ID generation, logging, and timeout functionality.

mod cors;
mod logging;
mod request_id;
mod timeout;

// Re-export middleware components
pub use self::cors::{CorsConfig, CorsLayer, cors_layer, permissive_cors_layer};
pub use self::logging::{LoggingConfig, LoggingLayer, detailed_logging_layer, logging_layer};
pub use self::request_id::{RequestIdLayer, request_id_layer};
pub use self::timeout::{
    TimeoutConfig, TimeoutLayer, timeout_layer, timeout_layer_with_duration, with_timeout,
};

/// Convenience function to create a default middleware stack.
pub fn default_middleware() -> (RequestIdLayer, LoggingLayer, CorsLayer, TimeoutLayer) {
    (
        request_id_layer(),
        logging_layer(),
        cors_layer(),
        timeout_layer(),
    )
}
