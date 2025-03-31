// Export base types for mocking
pub mod config;
pub mod events;
pub mod expect;

// Define the Expectation type and other key types
use std::any::Any;
use std::sync::{Arc, Mutex};

// Core traits and structures for mocking
pub trait MockProvider {
    fn register(&self, registry: &MockRegistry) -> Arc<Self>;
}

#[derive(Debug)]
pub struct MockRegistry;

impl MockRegistry {
    pub fn new() -> Self {
        MockRegistry
    }
}

// Re-export from the config, events, and expect modules
pub use self::config::*;
pub use self::events::*;
pub use self::expect::*;
