//! Dependency Injection module for Navius applications.
//!
//! This module provides a lightweight dependency injection system for Navius applications,
//! based on research of the spring-rs framework. It includes a component registry for
//! managing dependencies and lifecycle hooks for components.

// Export the component module
pub mod application;
pub mod component;

// Re-export common types for convenience
pub use application::{Application, ApplicationBuilder};
pub use component::{ComponentRef, ComponentRegistry, ComponentScope};

/// Initialize the DI system with a new registry
pub fn init() -> component::ComponentRegistry {
    component::ComponentRegistry::new()
}

/// Initialize the DI system with a new application builder
pub fn init_application() -> application::ApplicationBuilder {
    application::ApplicationBuilder::new()
}

/// Initialize the DI system with a custom configuration
pub fn init_application_with_config(
    config: crate::config::Config,
) -> application::ApplicationBuilder {
    application::ApplicationBuilder::with_config(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_di_init() {
        let registry = init();
        assert_eq!(registry.component_types().len(), 0);
    }

    #[test]
    fn test_application_init() {
        let app = init_application().build();
        assert!(app.registry().lock().unwrap().component_types().is_empty());
    }
}
