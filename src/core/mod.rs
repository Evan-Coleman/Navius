pub mod auth;
pub mod cache;
pub mod config;
pub mod handlers;
pub mod router;

// Re-export key components for easier access
pub use auth::{EntraAuthLayer, EntraTokenClient};
pub use cache::{CacheRegistry, ResourceCache, get_resource_cache, init_cache_registry};
pub use config::app_config::{AppConfig, load_config};
pub use router::CoreRouter;
