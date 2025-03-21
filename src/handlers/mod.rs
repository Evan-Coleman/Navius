// Example handlers for routes
// These are organized by functional area and serve as examples for users

// Examples of basic endpoint handlers
pub mod examples;

// Re-export core handlers for easier access by handlers users
pub use crate::core::handlers::{actuator, docs, health, logging};
