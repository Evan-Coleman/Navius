pub mod auth;
pub mod handlers;
pub mod router;

// Re-export key components for easier access
pub use auth::{EntraAuthLayer, EntraTokenClient};
pub use router::CoreRouter;
