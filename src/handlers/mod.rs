// Use public handler functions
pub use actuator::info;
pub use cache_admin::cache_debug;
pub use examples::pet::fetch_pet_handler;
pub use health::detailed_health_check;
pub use health::health_check;
pub use logging::log_request;
pub use metrics::metrics;

pub mod actuator;
pub mod cache_admin;
pub mod examples;
pub mod health;
pub mod logging;
pub mod metrics;
