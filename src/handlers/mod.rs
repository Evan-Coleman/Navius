pub mod examples;
pub mod health;
pub mod logging;
pub mod metrics;

// Re-export handlers for easier imports
pub use examples::catfact::get_catfact;
pub use examples::pet::get_pet_by_id;
pub use health::health_check;
pub use metrics::metrics;
