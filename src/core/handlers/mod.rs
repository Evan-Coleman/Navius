// This module contains core handlers that should not be modified by users
// Currently we're reusing the main handlers from the top-level handlers module,
// but in the future core-specific handlers could be implemented here.

// Re-exports for key handlers that are considered part of the core API
pub use crate::handlers::{actuator, docs, health, logging};
