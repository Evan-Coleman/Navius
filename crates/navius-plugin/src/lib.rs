//! Navius Plugin System
//!
//! This crate provides the core plugin system for the Navius framework. It enables
//! dynamic loading and management of plugins that extend the functionality of Navius
//! applications. The plugin system supports:
//!
//! - Dynamic plugin loading and unloading
//! - Plugin dependency management
//! - Plugin lifecycle management
//! - Capability-based plugin interfaces
//! - Plugin configuration and state management
//!
//! # Architecture
//!
//! The plugin system is built around several key concepts:
//!
//! - **Plugins**: Modular components that implement the `Plugin` trait
//! - **Capabilities**: Interfaces that plugins can implement to provide functionality
//! - **Registry**: Central management of plugin lifecycle and dependencies
//! - **Configuration**: Plugin-specific configuration management
//!
//! # Example
//!
//! ```rust,no_run
//! use navius_plugin::{Plugin, PluginResult};
//!
//! #[derive(Default)]
//! struct MyPlugin;
//!
//! impl Plugin for MyPlugin {
//!     fn id(&self) -> &str {
//!         "my_plugin"
//!     }
//!
//!     fn version(&self) -> &str {
//!         "1.0.0"
//!     }
//! }
//! ```

#![deny(missing_docs)]
#![warn(clippy::all)]

/// Plugin error types
pub mod error;

/// Core plugin traits and types
pub mod plugin;

/// Plugin registry for managing plugins
pub mod registry;

/// Plugin capabilities
pub mod capability;

/// Plugin loading utilities
pub mod loader;

/// Base plugin implementation
pub mod base;

// Re-export primary types
pub use base::{BasePlugin, PluginBuilder};
pub use capability::{
    Capability, ConfigurationCapability, EventCapability, HealthCheckCapability, HttpCapability,
    HttpRequest, HttpResponse, LoggingCapability, RoutingCapability, StorageCapability,
    downcast_capability, downcast_capability_mut,
};
pub use error::{PluginError, PluginHealth, PluginResult};
pub use loader::{InMemoryPluginProvider, PluginLoader};
pub use plugin::{Plugin, PluginConfig, PluginLifecycleStage, PluginMetadata};
pub use registry::{PluginRegistry, PluginRegistryManager};

/// Macros for easier plugin creation and registration
pub mod macros {
    /// Create a new plugin with the given name, version, description, and author
    #[macro_export]
    macro_rules! plugin {
        ($name:expr, $version:expr, $description:expr, $author:expr) => {
            $crate::base::PluginBuilder::new($name, $version, $description, $author)
        };
    }

    /// Create a plugin capability implementation
    #[macro_export]
    macro_rules! impl_capability {
        ($struct:ty, $cap_name:expr) => {
            impl $crate::capability::Capability for $struct {
                fn name(&self) -> &'static str {
                    $cap_name
                }

                fn as_any(&self) -> &dyn std::any::Any {
                    self
                }

                fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                    self
                }
            }
        };
    }
}
