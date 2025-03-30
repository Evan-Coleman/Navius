use std::error::Error;
use std::fmt;
use std::io;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Represents errors that can occur in the plugin system
#[derive(Debug)]
pub enum PluginError {
    /// Error occurred during plugin initialization
    InitializationError(String),

    /// Error loading a plugin
    LoadError(String),

    /// Error unloading a plugin
    UnloadError(String),

    /// Plugin dependency not found
    DependencyNotFound {
        plugin_id: String,
        dependency_id: String,
        version: String,
    },

    /// Plugin dependency version mismatch
    DependencyVersionMismatch {
        plugin_id: String,
        dependency_id: String,
        required: String,
        found: String,
    },

    /// Circular dependency detected
    CircularDependency(String),

    /// Plugin already registered
    PluginAlreadyRegistered(String),

    /// Plugin not registered
    PluginNotRegistered(String),

    /// Plugin not found
    PluginNotFound(String),

    /// Plugin capability not found
    CapabilityNotFound {
        plugin_id: String,
        capability_id: String,
    },

    /// Plugin in invalid state for operation
    InvalidState {
        plugin_id: String,
        current_state: String,
        required_state: String,
    },

    /// IO error
    IoError(io::Error),

    /// Dynamic library loading error
    LibraryError(String),

    /// Plugin configuration error
    ConfigurationError(String),

    /// Plugin validation error
    ValidationError(String),

    /// Plugin runtime error
    RuntimeError(String),

    /// Incompatible API version
    IncompatibleApiVersion { required: String, found: String },

    /// Other error
    Other(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitializationError(msg) => {
                write!(f, "Plugin initialization error: {}", msg)
            }
            PluginError::LoadError(msg) => write!(f, "Plugin load error: {}", msg),
            PluginError::UnloadError(msg) => write!(f, "Plugin unload error: {}", msg),
            PluginError::DependencyNotFound {
                plugin_id,
                dependency_id,
                version,
            } => write!(
                f,
                "Plugin '{}' requires dependency '{}' version '{}' which was not found",
                plugin_id, dependency_id, version
            ),
            PluginError::DependencyVersionMismatch {
                plugin_id,
                dependency_id,
                required,
                found,
            } => write!(
                f,
                "Plugin '{}' requires dependency '{}' version '{}' but found '{}'",
                plugin_id, dependency_id, required, found
            ),
            PluginError::CircularDependency(msg) => {
                write!(f, "Circular dependency detected: {}", msg)
            }
            PluginError::PluginAlreadyRegistered(id) => {
                write!(f, "Plugin '{}' is already registered", id)
            }
            PluginError::PluginNotRegistered(id) => write!(f, "Plugin '{}' is not registered", id),
            PluginError::PluginNotFound(id) => write!(f, "Plugin '{}' not found", id),
            PluginError::CapabilityNotFound {
                plugin_id,
                capability_id,
            } => write!(
                f,
                "Capability '{}' not found in plugin '{}'",
                capability_id, plugin_id
            ),
            PluginError::InvalidState {
                plugin_id,
                current_state,
                required_state,
            } => write!(
                f,
                "Plugin '{}' is in state '{}' but requires state '{}'",
                plugin_id, current_state, required_state
            ),
            PluginError::IoError(err) => write!(f, "IO error: {}", err),
            PluginError::LibraryError(msg) => write!(f, "Dynamic library error: {}", msg),
            PluginError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PluginError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            PluginError::IncompatibleApiVersion { required, found } => write!(
                f,
                "Incompatible API version: required '{}', found '{}'",
                required, found
            ),
            PluginError::Other(msg) => write!(f, "Plugin error: {}", msg),
        }
    }
}

impl Error for PluginError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for PluginError {
    fn from(err: io::Error) -> Self {
        PluginError::IoError(err)
    }
}

/// Represents the health status of a plugin
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginHealth {
    /// Plugin is healthy and operating normally
    Healthy,

    /// Plugin is degraded but still operational
    Degraded {
        /// Message describing the degraded state
        message: String,
    },

    /// Plugin is unhealthy and not operational
    Unhealthy {
        /// Message describing the unhealthy state
        message: String,
    },
}

impl fmt::Display for PluginHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginHealth::Healthy => write!(f, "Healthy"),
            PluginHealth::Degraded { message } => write!(f, "Degraded: {}", message),
            PluginHealth::Unhealthy { message } => write!(f, "Unhealthy: {}", message),
        }
    }
}
