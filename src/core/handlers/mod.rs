// Core handlers for common routes
// These handlers provide the core functionality and should not be modified by users

// Health check handlers
pub mod core_health;

// Debug and management actuator endpoints
pub mod core_actuator;

// API documentation handlers
pub mod core_docs;

// Logging middleware
pub mod core_logging;

// Re-export key handlers for easier access
pub use self::{core_actuator::*, core_docs::*, core_health::*, core_logging::*};
