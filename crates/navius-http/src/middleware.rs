//! HTTP middleware for the Navius framework.
//!
//! This module provides middleware for use with the HTTP server.

#[cfg(feature = "server")]
pub mod request_id;

// Modules to be implemented later
// #[cfg(feature = "server")]
// pub mod cors;
// #[cfg(feature = "server")]
// pub mod logging;
// #[cfg(feature = "server")]
// pub mod timeout;

/// Common middleware re-exports
#[cfg(feature = "server")]
pub mod prelude {
    pub use super::request_id::RequestIdLayer;
    // pub use super::cors::cors_layer;
    // pub use super::logging::LoggingLayer;
    // pub use super::timeout::TimeoutLayer;
}

/// Re-export common middleware creation functions
#[cfg(feature = "server")]
pub use self::request_id::request_id_layer;
// #[cfg(feature = "server")]
// pub use self::cors::cors_layer;
// #[cfg(feature = "server")]
// pub use self::logging::logging_layer;
// #[cfg(feature = "server")]
// pub use self::timeout::timeout_layer;
