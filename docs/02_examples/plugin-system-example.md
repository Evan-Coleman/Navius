---
title: "Plugin System Example"
description: "Learn how to extend your Navius application using the plugin system and component registry"
category: examples
tags:
  - plugins
  - extensibility
  - component-registry
  - dependency-injection
  - lifecycle
related:
  - 02_examples/dependency-injection-example.md
  - 02_examples/custom-service-example.md
  - 01_getting_started/application-structure.md
last_updated: June 30, 2025
version: 1.0
status: stable
---

# Plugin System Example

This example demonstrates how to use the Navius plugin system to extend your application with custom functionality and leverage the component registry for dependency injection.

## Overview

The Navius plugin system provides a flexible way to extend your application with modular, reusable components. Each plugin can:

- Register components in the component registry
- Manage its own lifecycle (initialization and shutdown)
- Define dependencies on other plugins
- Access shared resources

This example builds a weather service application using plugins to demonstrate how to create, register, and use plugins in a Navius application.

## Quick Navigation

- [Project Structure](#project-structure)
- [Plugin System Concepts](#plugin-system-concepts)
- [Component Registry](#component-registry)
- [Creating Custom Plugins](#creating-custom-plugins)
- [Plugin Lifecycle Management](#plugin-lifecycle-management)
- [Dependency Management](#dependency-management)
- [Using Components](#using-components)
- [Testing Plugins](#testing-plugins)
- [Best Practices](#best-practices)

## Prerequisites

Before working with this example, you should be familiar with:

- Rust programming basics, including traits and trait objects
- Asynchronous programming with Tokio
- Basic understanding of dependency injection
- Navius framework fundamentals

Required dependencies:
- Rust 1.70 or newer
- Navius 0.1.0 or newer, including navius-plugin
- async-trait 0.1.0 or newer
- tokio for asynchronous operations

## Project Structure

```
plugin-system-example/
├── Cargo.toml
├── config/
│   └── default.yaml
└── src/
    ├── main.rs
    ├── plugins/
    │   ├── mod.rs
    │   ├── weather_api_plugin.rs
    │   ├── location_plugin.rs
    │   ├── cache_plugin.rs
    │   └── notification_plugin.rs
    ├── services/
    │   ├── mod.rs
    │   ├── weather_service.rs
    │   ├── location_service.rs
    │   └── notification_service.rs
    ├── models/
    │   ├── mod.rs
    │   ├── weather.rs
    │   └── location.rs
    └── api/
        ├── mod.rs
        └── weather_handler.rs
```

## Plugin System Concepts

The Navius plugin system is built around the `Plugin` trait, which defines the lifecycle and capabilities of a plugin:

```rust
use navius_plugin::Plugin;
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    // The name of the plugin
    fn name(&self) -> &str;
    
    // Dependencies on other plugins, if any
    fn dependencies(&self) -> Vec<&str> { Vec::new() }
    
    // Check if the plugin is initialized
    fn is_initialized(&self) -> bool;
    
    // Initialize the plugin asynchronously
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    // Shutdown the plugin asynchronously
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
```

## Component Registry

The component registry allows plugins to register and retrieve components:

```rust
use navius_plugin::ComponentRegistry;
use std::sync::Arc;

// Register a component
let registry = ComponentRegistry::global();
registry.register::<dyn WeatherService>().with(Arc::new(MyWeatherService::new()) as Arc<dyn WeatherService>);

// Get a component
let weather_service = registry.get::<dyn WeatherService>().unwrap();
```

## Creating Custom Plugins

Let's create a simple weather API plugin:

### `plugins/weather_api_plugin.rs`

```rust
use async_trait::async_trait;
use navius_plugin::{Plugin, ComponentRegistry, SimplePlugin};
use std::sync::{Arc, Mutex};
use crate::services::WeatherService;

pub struct WeatherApiPlugin {
    base: SimplePlugin,
    api_key: String,
    service: Option<Arc<WeatherApiService>>,
}

impl WeatherApiPlugin {
    pub fn new(api_key: String) -> Self {
        Self {
            base: SimplePlugin::new("weather-api"),
            api_key,
            service: None,
        }
    }
}

#[async_trait]
impl Plugin for WeatherApiPlugin {
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn dependencies(&self) -> Vec<&str> {
        vec!["cache", "location"]
    }
    
    fn is_initialized(&self) -> bool {
        self.base.is_initialized()
    }
    
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_initialized() {
            return Ok(());
        }
        
        // Create and register the weather service
        let registry = ComponentRegistry::global();
        
        // Get dependencies
        let cache_service = registry.get::<dyn CacheService>()
            .map_err(|e| format!("Failed to get cache service: {}", e))?;
            
        let location_service = registry.get::<dyn LocationService>()
            .map_err(|e| format!("Failed to get location service: {}", e))?;
        
        // Create our service
        let service = Arc::new(WeatherApiService::new(
            self.api_key.clone(),
            cache_service,
            location_service,
        ));
        
        // Store a reference
        let mut plugin_self = unsafe { &mut *(self as *const _ as *mut Self) };
        plugin_self.service = Some(service.clone());
        
        // Register in the component registry
        registry.register::<dyn WeatherService>().with(service as Arc<dyn WeatherService>)?;
        
        // Mark as initialized
        plugin_self.base.set_initialized(true);
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_initialized() {
            return Ok(());
        }
        
        // Cleanup any resources
        let mut plugin_self = unsafe { &mut *(self as *const _ as *mut Self) };
        plugin_self.service = None;
        plugin_self.base.set_initialized(false);
        
        Ok(())
    }
}
```

## Plugin Lifecycle Management

Managing the lifecycle of plugins:

### `main.rs`

```rust
use navius::{Application, Config};
use navius_plugin::PluginRegistry;
use crate::plugins::{
    WeatherApiPlugin, 
    LocationPlugin, 
    CachePlugin, 
    NotificationPlugin
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create plugin registry
    let plugin_registry = PluginRegistry::new();
    
    // Register plugins
    plugin_registry.register(Box::new(CachePlugin::new()))?;
    plugin_registry.register(Box::new(LocationPlugin::new()))?;
    plugin_registry.register(Box::new(WeatherApiPlugin::new(
        "your-weather-api-key".to_string(),
    )))?;
    plugin_registry.register(Box::new(NotificationPlugin::new()))?;
    
    // Initialize plugins
    plugin_registry.initialize_all().await?;
    
    // Configure and run the application
    let app = Application::new()
        .with_config(Config::from_file("config/default.yaml"))
        .with_plugin_registry(plugin_registry)
        .configure(configure_app)
        .build()?;
    
    // Run the application
    app.run().await?;
    
    // Application is shutting down, clean up plugins
    app.plugin_registry().shutdown_all().await?;
    
    Ok(())
}

fn configure_app(app: &mut navius::App) {
    // Configure routes, middleware, etc.
    app.add_routes(crate::api::routes());
}
```

## Dependency Management

The plugin system automatically handles dependencies:

```rust
fn dependencies(&self) -> Vec<&str> {
    vec!["cache", "location"] // This plugin depends on the cache and location plugins
}
```

The plugin registry ensures that plugins are initialized in the correct order based on their dependencies. If circular dependencies are detected, an error is returned.

## Using Components

Once components are registered by plugins, they can be used anywhere in your application:

### `api/weather_handler.rs`

```rust
use navius::{Json, State};
use navius_plugin::ComponentRegistry;
use axum::{Router, routing::get};
use std::sync::Arc;
use crate::services::WeatherService;
use crate::models::Weather;

async fn get_weather(
    State(registry): State<Arc<ComponentRegistry>>,
    Json(query): Json<WeatherQuery>,
) -> Result<Json<Weather>, navius::Error> {
    let weather_service = registry.get::<dyn WeatherService>()
        .map_err(|e| navius::Error::internal_server_error(e.to_string()))?;
    
    let weather = weather_service.get_current_weather(&query.location).await
        .map_err(|e| navius::Error::internal_server_error(e.to_string()))?;
    
    Ok(Json(weather))
}

pub fn routes() -> Router {
    Router::new()
        .route("/weather", get(get_weather))
}

#[derive(serde::Deserialize)]
struct WeatherQuery {
    location: String,
}
```

## Testing Plugins

Testing plugins in isolation:

```rust
#[tokio::test]
async fn test_weather_api_plugin() {
    // Create a mock component registry
    let registry = ComponentRegistry::new();
    ComponentRegistry::set_global(Arc::new(registry));
    
    // Create mock dependencies
    let mock_cache = Arc::new(MockCacheService::new());
    let mock_location = Arc::new(MockLocationService::new());
    
    // Register mock dependencies
    let registry = ComponentRegistry::global();
    registry.register::<dyn CacheService>().with(mock_cache.clone() as Arc<dyn CacheService>)?;
    registry.register::<dyn LocationService>().with(mock_location.clone() as Arc<dyn LocationService>)?;
    
    // Create and initialize the plugin
    let plugin = WeatherApiPlugin::new("test-api-key".to_string());
    plugin.initialize().await.expect("Failed to initialize plugin");
    
    // Test the plugin
    let weather_service = registry.get::<dyn WeatherService>()
        .expect("Failed to get weather service");
    
    let weather = weather_service.get_current_weather("London").await
        .expect("Failed to get weather");
    
    assert_eq!(weather.location, "London");
    
    // Shutdown the plugin
    plugin.shutdown().await.expect("Failed to shutdown plugin");
}
```

## Best Practices

When working with the Navius plugin system:

1. **Keep Plugins Focused**: Each plugin should have a single responsibility
2. **Handle Dependencies Carefully**: Declare all dependencies explicitly
3. **Manage Resources Properly**: Initialize resources in `initialize()` and clean them up in `shutdown()`
4. **Use Named Components**: When registering multiple implementations of the same trait, use named components
5. **Consider Scopes**: Use appropriate scope types (Singleton or Prototype) for your components
6. **Error Handling**: Provide detailed error messages when plugin operations fail
7. **Testing**: Create mocks for testing plugins in isolation

## Conclusion

The Navius plugin system provides a powerful way to extend your application with modular, reusable components. By following the patterns and practices outlined in this example, you can build flexible, maintainable applications that can grow with your needs.

For more information about the plugin system, see the [Navius Plugin Documentation](/05_reference/plugin-system.md). 