/// Application router and state management
pub use crate::app::router::*;
pub mod router;

// Re-export key components from router module
pub use router::create_router;
pub use router::init;

// Re-export AppState from core
pub use crate::core::router::AppState;

/// User-facing API endpoints
pub mod api;

/// User-facing services
pub mod services;
