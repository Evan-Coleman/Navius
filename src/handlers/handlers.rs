// Handlers module for Navius application

// Declare submodules
pub mod admin_handlers;
pub mod api_handlers;
pub mod common;

// Re-export commonly used handlers from the common module
pub use common::{health_check, hello_world};
