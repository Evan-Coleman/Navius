// Core handlers for common routes
// These handlers provide the core functionality and should not be modified by users

// Health check handlers
pub mod health;

// Debug and management actuator endpoints
pub mod actuator;

// API documentation handlers
pub mod docs;

// Logging middleware
pub mod logging;

// Re-export key handlers for easier access
pub use self::{actuator::*, docs::*, health::*, logging::*};
