/*!
Navius Plugin - Plugin system and component registry for the Navius framework

This crate provides a lightweight plugin system and component registry for the Navius framework,
inspired by spring-rs patterns. It enables modular application design with clear boundaries
and dependencies between components.

## Features

- **Plugin System**: Register and manage plugins with lifecycle hooks
- **Component Registry**: Type-safe dependency injection
- **Service Discovery**: Automatically discover and register services
- **Lifecycle Management**: Initialize and shutdown components in the correct order

## Main Components

- `Plugin`: Trait for creating plugins with lifecycle hooks
- `PluginRegistry`: Central registry for managing plugins
- `Component`: Type-safe wrapper for accessing components
- `ComponentRegistry`: Registry for component registration and resolution
*/

pub mod component;
pub mod error;
pub mod plugin;
#[cfg(test)]
mod tests;

// Re-exports
pub use component::{Component, ComponentKey, ComponentRegistry, Scope};
pub use error::PluginError;
pub use plugin::{Plugin, PluginBuilder, PluginRegistry, SimplePlugin};
