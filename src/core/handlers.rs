// Core handlers for common routes
// These handlers provide the core functionality and should not be modified by users

// Health check handlers
pub mod core_health;

// Health dashboard handler
pub mod health_dashboard_handler;

// Debug and management actuator endpoints
pub mod core_actuator;

// API documentation handlers
pub mod core_docs;

// Logging middleware
pub mod core_logging;

// Health handler
pub mod health_handler;

// Redirect handler
pub mod redirect_handler;

// Re-export key handlers for easier access
pub use self::{
    core_actuator::*, core_docs::*, core_health::*, core_logging::*, health_dashboard_handler::*,
    health_handler::*, redirect_handler::*,
};
