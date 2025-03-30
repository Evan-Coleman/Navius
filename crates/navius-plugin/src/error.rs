//! Error types for the plugin system

use std::fmt;
use thiserror::Error;

/// Errors that can occur during plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Error initializing a plugin.
    #[error("Failed to initialize plugin: {message}")]
    InitializationError {
        /// Error message.
        message: String,
    },

    /// Attempted to register a plugin with an ID that already exists.
    #[error("Duplicate plugin ID: {plugin_id}")]
    DuplicatePlugin {
        /// The ID of the duplicate plugin.
        plugin_id: String,
    },

    /// A required dependency was not found.
    #[error("Dependency not found: plugin {plugin_id} requires {dependency_id}")]
    DependencyNotFound {
        /// The ID of the plugin that has the dependency.
        plugin_id: String,
        /// The ID of the missing dependency.
        dependency_id: String,
    },

    /// A circular dependency was detected.
    #[error("Circular dependency detected: {plugin_id}")]
    CircularDependency {
        /// The ID of the plugin involved in the circular dependency.
        plugin_id: String,
    },

    /// A requested component was not found in the registry.
    #[error("Component not found: {component_type}{}", .name.as_ref().map(|n| format!(" with name '{n}'")).unwrap_or_default())]
    ComponentNotFound {
        /// The type of the component.
        component_type: String,
        /// The name of the component, if any.
        name: Option<String>,
    },

    /// Failed to register a component.
    #[error("Failed to register component: {message}")]
    ComponentRegistrationError {
        /// Error message.
        message: String,
    },

    /// The requested component type doesn't match the stored type.
    #[error("Component type mismatch: requested {requested}, found {actual}")]
    ComponentTypeMismatch {
        /// The requested component type.
        requested: String,
        /// The actual component type.
        actual: String,
    },

    /// Error during plugin shutdown.
    #[error("Error shutting down plugins: {}", .errors.join(", "))]
    ShutdownError {
        /// List of errors encountered during shutdown.
        errors: Vec<String>,
    },

    /// Generic error with a custom message.
    #[error("{message}")]
    GenericError {
        /// Error message.
        message: String,
    },
}

/// Result type for plugin operations.
pub type PluginResult<T> = Result<T, PluginError>;

/// Trait for converting generic errors into plugin errors.
pub trait IntoPluginError {
    /// Converts the error into a plugin error.
    fn into_plugin_error(self, context: &str) -> PluginError;
}

impl<E: fmt::Display> IntoPluginError for E {
    fn into_plugin_error(self, context: &str) -> PluginError {
        PluginError::GenericError {
            message: format!("{}: {}", context, self),
        }
    }
}
