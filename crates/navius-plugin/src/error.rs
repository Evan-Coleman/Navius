use std::error::Error as StdError;
use std::fmt;
use std::io;
use thiserror::Error;

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

    /// Generic dependency error
    DependencyError(String),

    /// Plugin dependency not found
    DependencyNotFound {
        /// The ID of the plugin that requires the dependency
        plugin_id: String,
        /// The ID of the missing dependency
        dependency_id: String,
        /// The required version of the dependency
        version: String,
    },

    /// Plugin dependency version mismatch
    DependencyVersionMismatch {
        /// The ID of the plugin with the incompatible dependency
        plugin_id: String,
        /// The ID of the dependency with version mismatch
        dependency_id: String,
        /// The required version of the dependency
        required: String,
        /// The actual version found
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
        /// The ID of the plugin missing the capability
        plugin_id: String,
        /// The ID of the missing capability
        capability_id: String,
    },

    /// Plugin in invalid state for operation
    InvalidState {
        /// The ID of the plugin in invalid state
        plugin_id: String,
        /// The current state of the plugin
        current_state: String,
        /// The required state for the operation
        required_state: String,
    },

    /// IO error occurred during plugin operation
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
    IncompatibleApiVersion {
        /// The required API version
        required: String,
        /// The actual API version found
        found: String,
    },

    /// Other error
    Other(String),

    /// Missing plugin dependency
    MissingDependency {
        /// The ID of the plugin that requires the dependency
        plugin_id: String,
        /// The ID of the missing dependency
        dependency_id: String,
        /// The required version of the dependency
        version: String,
    },

    /// Incompatible plugin dependency version
    IncompatibleDependency {
        /// The ID of the plugin with the incompatible dependency
        plugin_id: String,
        /// The ID of the dependency with version mismatch
        dependency_id: String,
        /// The required version of the dependency
        required: String,
        /// The actual version found
        found: String,
    },

    /// Missing plugin capability
    MissingCapability {
        /// The ID of the plugin requiring the capability
        plugin_id: String,
        /// The ID of the missing capability
        capability_id: String,
    },

    /// Invalid plugin lifecycle state transition
    InvalidStateTransition {
        /// The ID of the plugin with invalid state transition
        plugin_id: String,
        /// The current state of the plugin
        current_state: String,
        /// The state that was attempted to transition to
        required_state: String,
    },
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::InitializationError(msg) => {
                write!(f, "Plugin initialization error: {}", msg)
            }
            PluginError::LoadError(msg) => write!(f, "Plugin load error: {}", msg),
            PluginError::UnloadError(msg) => write!(f, "Plugin unload error: {}", msg),
            PluginError::DependencyError(msg) => write!(f, "Plugin dependency error: {}", msg),
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
            PluginError::IoError(err) => write!(f, "Plugin IO error: {}", err),
            PluginError::LibraryError(msg) => write!(f, "Dynamic library error: {}", msg),
            PluginError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            PluginError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            PluginError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            PluginError::IncompatibleApiVersion { required, found } => write!(
                f,
                "API version mismatch: required '{}', found '{}'",
                required, found
            ),
            PluginError::Other(msg) => write!(f, "Plugin error: {}", msg),
            PluginError::MissingDependency {
                plugin_id,
                dependency_id,
                version,
            } => write!(
                f,
                "Plugin '{}' requires dependency '{}' version '{}' which was not found",
                plugin_id, dependency_id, version
            ),
            PluginError::IncompatibleDependency {
                plugin_id,
                dependency_id,
                required,
                found,
            } => write!(
                f,
                "Plugin '{}' dependency '{}' version mismatch: required '{}', found '{}'",
                plugin_id, dependency_id, required, found
            ),
            PluginError::MissingCapability {
                plugin_id,
                capability_id,
            } => write!(
                f,
                "Plugin '{}' requires capability '{}' which was not found",
                plugin_id, capability_id
            ),
            PluginError::InvalidStateTransition {
                plugin_id,
                current_state,
                required_state,
            } => write!(
                f,
                "Plugin '{}' invalid state transition from '{}' to '{}'",
                plugin_id, current_state, required_state
            ),
        }
    }
}

impl StdError for PluginError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
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

/// Error indicating a missing plugin dependency
#[derive(Debug, Error)]
#[error("Plugin {plugin_id} requires dependency {dependency_id} version {version}")]
pub struct MissingDependencyError {
    /// The ID of the plugin that requires the dependency
    pub plugin_id: String,
    /// The ID of the missing dependency
    pub dependency_id: String,
    /// The required version of the dependency
    pub version: String,
}

/// Error indicating an incompatible plugin dependency version
#[derive(Debug, Error)]
#[error(
    "Plugin {plugin_id} dependency {dependency_id} version mismatch: required {required}, found {found}"
)]
pub struct IncompatibleDependencyError {
    /// The ID of the plugin with the incompatible dependency
    pub plugin_id: String,
    /// The ID of the dependency with version mismatch
    pub dependency_id: String,
    /// The required version of the dependency
    pub required: String,
    /// The actual version found
    pub found: String,
}

/// Error indicating a missing plugin capability
#[derive(Debug, Error)]
#[error("Plugin {plugin_id} requires capability {capability_id}")]
pub struct MissingCapabilityError {
    /// The ID of the plugin requiring the capability
    pub plugin_id: String,
    /// The ID of the missing capability
    pub capability_id: String,
}

/// Error indicating an invalid plugin lifecycle state transition
#[derive(Debug, Error)]
#[error("Plugin {plugin_id} invalid state transition from {current_state} to {required_state}")]
pub struct InvalidStateTransitionError {
    /// The ID of the plugin with invalid state transition
    pub plugin_id: String,
    /// The current state of the plugin
    pub current_state: String,
    /// The state that was attempted to transition to
    pub required_state: String,
}

/// Error indicating incompatible API versions
#[derive(Debug, Error)]
#[error("API version mismatch: required {required}, found {found}")]
pub struct IncompatibleApiVersion {
    /// The required API version
    pub required: String,
    /// The actual API version found
    pub found: String,
}
