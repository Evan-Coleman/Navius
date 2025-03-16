pub mod data;
pub mod health;
pub mod metrics;
pub mod pet;
pub mod pokemon;
// Re-export handlers for easier imports
pub use data::get_data;
pub use health::health_check;
pub use metrics::metrics;
pub use pet::get_pet_by_id;
