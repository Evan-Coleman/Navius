use std::fmt;
use thiserror::Error;

/// Dependency Injection Error
#[derive(Error, Debug)]
pub enum Error {
    /// A component was not found in the registry
    #[error("Component not found: {name}")]
    ComponentNotFound { name: String },

    /// A qualifier was not found in the registry
    #[error("Qualifier not found: {qualifier}")]
    QualifierNotFound { qualifier: String },

    /// A type mismatch occurred between expected and found types
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    /// A circular dependency was detected during component initialization
    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: String },

    /// A component lifecycle error occurred
    #[error("Lifecycle error: {message}")]
    LifecycleError { message: String },

    /// An autowiring error occurred
    #[error("Autowiring error: {message}")]
    AutowiringError { message: String },

    /// Multiple matching components were found for an autowired dependency
    #[error("Multiple matching components found for dependency {dependency}")]
    MultipleMatchingComponents { dependency: String },

    /// A component with the same qualifier already exists
    #[error("A component with qualifier '{qualifier}' already exists")]
    DuplicateQualifier { qualifier: String },

    /// An operation timed out
    #[error("Operation timed out: {operation}")]
    Timeout { operation: String },

    /// A scope error occurred
    #[error("Scope error: {message}")]
    ScopeError { message: String },

    /// Failed to initialize component
    #[error("Failed to initialize component: {message}")]
    InitializationFailed { message: String },

    /// Failed to shut down component
    #[error("Failed to shut down component: {message}")]
    ShutdownFailed { message: String },

    /// Failed to downcast component to target type
    #[error("Invalid component type: {message}")]
    InvalidType { message: String },

    /// Configuration not found
    #[error("Configuration not found: {key}")]
    ConfigNotFound { key: String },

    /// Failed to bind configuration
    #[error("Failed to bind configuration: {message}")]
    ConfigBindingFailed { message: String },

    /// Plugin initialization failed
    #[error("Plugin initialization failed: {message}")]
    PluginInitializationFailed { message: String },

    /// Invalid scope for component
    #[error("Invalid component scope: {message}")]
    InvalidScope { message: String },

    /// Missing dependency
    #[error("Missing dependency: {name}")]
    MissingDependency { name: String },

    /// Any other error
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Create a new error with a message
    pub fn new(message: &str) -> Self {
        Self::Other(message.to_string())
    }

    /// Create a component not found error
    pub fn component_not_found<T: std::fmt::Debug>(component: T) -> Self {
        Self::ComponentNotFound {
            name: format!("{:?}", component),
        }
    }

    /// Create a qualifier not found error
    pub fn qualifier_not_found(qualifier: &str) -> Self {
        Self::QualifierNotFound {
            qualifier: qualifier.to_string(),
        }
    }

    /// Create an initialization failed error
    pub fn initialization_failed<E: std::fmt::Display>(error: E) -> Self {
        Self::InitializationFailed {
            message: error.to_string(),
        }
    }

    /// Create a shutdown failed error
    pub fn shutdown_failed<E: std::fmt::Display>(error: E) -> Self {
        Self::ShutdownFailed {
            message: error.to_string(),
        }
    }

    /// Create a configuration not found error
    pub fn config_not_found(key: &str) -> Self {
        Self::ConfigNotFound {
            key: key.to_string(),
        }
    }

    /// Create a configuration binding failed error
    pub fn config_binding_failed<E: std::fmt::Display>(error: E) -> Self {
        Self::ConfigBindingFailed {
            message: error.to_string(),
        }
    }

    /// Create a plugin initialization failed error
    pub fn plugin_initialization_failed<E: std::fmt::Display>(error: E) -> Self {
        Self::PluginInitializationFailed {
            message: error.to_string(),
        }
    }

    /// Create a missing dependency error
    pub fn missing_dependency(name: &str) -> Self {
        Self::MissingDependency {
            name: name.to_string(),
        }
    }
}

/// Result type for dependency injection operations
pub type Result<T> = std::result::Result<T, Error>;

/// Convert a string error to a DI error
pub trait IntoError<T> {
    /// Convert to an Error::InitializationFailed error
    fn into_initialization_error(self) -> Result<T>;

    /// Convert to an Error::ShutdownFailed error
    fn into_shutdown_error(self) -> Result<T>;

    /// Convert to an Error::ConfigBindingFailed error
    fn into_config_error(self) -> Result<T>;

    /// Convert to an Error::PluginInitializationFailed error
    fn into_plugin_error(self) -> Result<T>;
}

impl<T, E: fmt::Display> IntoError<T> for std::result::Result<T, E> {
    fn into_initialization_error(self) -> Result<T> {
        self.map_err(Error::initialization_failed)
    }

    fn into_shutdown_error(self) -> Result<T> {
        self.map_err(Error::shutdown_failed)
    }

    fn into_config_error(self) -> Result<T> {
        self.map_err(Error::config_binding_failed)
    }

    fn into_plugin_error(self) -> Result<T> {
        self.map_err(Error::plugin_initialization_failed)
    }
}
