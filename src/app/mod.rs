pub mod router;

// Re-export key components from router module
pub use router::create_router;
pub use router::init;

// Re-export AppState from core
pub use crate::core::router::AppState;
