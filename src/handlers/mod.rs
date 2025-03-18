pub mod cache_admin;
pub mod examples;
pub mod health;
pub mod logging;
pub mod metrics;

// Re-export handlers for easier imports
pub use cache_admin::cache_debug;
pub use examples::catfact::fetch_catfact_handler;
pub use examples::pet::fetch_pet_handler;
pub use health::health_check;
pub use metrics::metrics;
