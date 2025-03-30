# Navius Plugin System

A comprehensive plugin system for the Navius framework that provides extensibility and modularity.

## Features

- Dynamic plugin loading and management
- Dependency management between plugins
- Comprehensive lifecycle management
- Capability-based plugin architecture
- Health monitoring
- Simple plugin creation API

## Architecture

The Navius Plugin System is designed with the following components:

1. **Plugin Registry**: Central manager for plugins and their lifecycle
2. **Plugin Capabilities**: Extensible interfaces for plugin functionality
3. **Plugin Loader**: Mechanism for dynamically loading plugins
4. **Plugin Lifecycle**: Clear stages from initialization to shutdown

## Plugin Lifecycle

Plugins go through the following lifecycle stages:

1. **Created**: Plugin instance is created but not initialized
2. **Initialized**: Plugin is initialized with configuration
3. **Started**: Plugin is running and providing functionality
4. **Stopped**: Plugin has been stopped but not unloaded
5. **Unloaded**: Plugin has been unloaded from memory

## Basic Usage

```rust
use navius_plugin::{plugin, PluginRegistry, PluginRegistryManager};

// Create a plugin registry
let registry = PluginRegistry::new();

// Create a simple plugin using the builder
let simple_plugin = plugin!(
    "SimplePlugin",
    "1.0.0",
    "A simple example plugin",
    "Navius Team"
)
.with_tag("example")
.build();

// Register the plugin
let plugin_id = registry.register_plugin(simple_plugin).await?;

// Start all plugins
registry.start_all_plugins().await?;

// Do work with plugins...

// Stop all plugins when done
registry.stop_all_plugins().await?;
```

## Plugin Capabilities

The capability system allows plugins to expose specific functionality:

```rust
use navius_plugin::{impl_capability, LoggingCapability};

// Define a custom logging capability
#[derive(Debug)]
struct MyLogger {
    prefix: String,
}

// Implement the Capability trait
impl_capability!(MyLogger, "my_logger");

// Implement the LoggingCapability trait
impl LoggingCapability for MyLogger {
    fn info(&self, message: &str) {
        println!("[INFO] {}: {}", self.prefix, message);
    }
    
    // Implement other methods...
}

// Create a plugin with this capability
let plugin = plugin!("LoggingPlugin", "1.0.0", "Logging plugin", "Navius Team")
    .with_capability(MyLogger::new("LogPlugin"))
    .build();
```

Available capability traits include:

- `LoggingCapability`: For log management
- `ConfigurationCapability`: For plugin configuration
- `HealthCheckCapability`: For health monitoring
- `StorageCapability`: For data storage
- `EventCapability`: For event publishing/subscribing
- `HttpCapability`: For HTTP functionality
- `RoutingCapability`: For HTTP routing

## Dependency Management

Plugins can declare dependencies on other plugins:

```rust
let plugin = plugin!("MyPlugin", "1.0.0", "My plugin", "Navius Team")
    .with_dependency("OtherPlugin", "1.0.0")
    .build();
```

The plugin registry handles:

- Resolving dependencies during plugin registration
- Detecting circular dependencies
- Starting plugins in dependency order
- Stopping plugins in reverse dependency order

## Dynamic Loading

For dynamically loading plugins from shared libraries:

```rust
use navius_plugin::PluginLoader;

let mut loader = PluginLoader::new();
loader.add_search_path("./plugins");

// Find available plugins
let plugin_files = loader.find_plugins()?;

// Load a specific plugin
let plugin = loader.load_plugin("./plugins/my_plugin.so")?;

// Or scan and load all plugins
let plugins = loader.scan_and_load()?;
```

## Example

A complete example of using the plugin system:

```rust
use navius_plugin::{
    plugin, impl_capability, PluginRegistry, 
    LoggingCapability, HealthCheckCapability
};

// Create capabilities...
// Create plugins...

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and set up the registry
    let registry = PluginRegistry::new();
    
    // Register plugins
    registry.register_plugin(plugin1).await?;
    registry.register_plugin(plugin2).await?;
    
    // Resolve dependencies
    registry.resolve_dependencies()?;
    
    // Start all plugins
    registry.start_all_plugins().await?;
    
    // Use plugins
    // ...
    
    // Check health
    let health = registry.health_check_all().await;
    
    // Stop all plugins
    registry.stop_all_plugins().await?;
    
    Ok(())
}
```

See the `examples/` directory for more examples.

## License

MIT OR Apache-2.0 