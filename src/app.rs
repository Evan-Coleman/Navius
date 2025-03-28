/// Application router and state management
pub use crate::app::router::*;
pub mod router;

// Router has no init function - removing the incorrect import
//pub use router::init;

// Re-export AppState from core
pub use crate::core::router::AppState;

/// User-facing API endpoints
pub mod api;

/// User-facing services
pub mod services;

/// Example models for entities
pub mod models;

/// Example repositories
pub mod repositories;

/// User-facing metrics functionality
pub mod metrics;

/// User-facing authentication functionality
pub mod auth;

/// User-facing reliability features
pub mod reliability;

/// User-facing utility functions
pub mod utils;

/// User-facing configuration functionality
pub mod config;

/// User-facing caching functionality
pub mod cache;
