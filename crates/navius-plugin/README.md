# Navius Plugin

A lightweight plugin system and component registry for the Navius framework, inspired by spring-rs patterns.

## Features

- **Plugin System**: Register and manage plugins with lifecycle hooks
- **Component Registry**: Type-safe dependency injection
- **Service Discovery**: Automatically discover and register services
- **Lifecycle Management**: Initialize and shutdown components in the correct order

## Plugin System

The plugin system allows you to create modular applications with clear boundaries:

```rust
use navius_plugin::{Plugin, PluginBuilder, PluginRegistry, SimplePlugin};
use async_trait::async_trait;
use std::any::Any;

// Create a simple plugin
let plugin = SimplePlugin::new(
    PluginBuilder::new("my-plugin")
        .description("My first plugin")
        .depends_on("core-plugin")
);

// Create a plugin registry
let mut registry = PluginRegistry::new();

// Register the plugin
registry.register(plugin).unwrap();

// Initialize all plugins (respecting dependencies)
registry.initialize_all().await.unwrap();

// Shutdown all plugins in reverse order
registry.shutdown_all().await.unwrap();
```

### Custom Plugins

You can create custom plugins by implementing the `Plugin` trait:

```rust
#[derive(Debug)]
struct MyPlugin {
    id: String,
    description: String,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn dependencies(&self) -> Vec<String> {
        vec!["core-plugin".to_string()]
    }

    async fn initialize(&self, registry: &ComponentRegistry) -> PluginResult<()> {
        // Register components
        let service = MyService::new();
        registry.register(
            ComponentKey::default::<MyService>(),
            service,
            Scope::Singleton
        )?;
        
        Ok(())
    }

    async fn shutdown(&self, registry: &ComponentRegistry) -> PluginResult<()> {
        // Cleanup resources
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

## Component Registry

The component registry provides type-safe dependency injection:

```rust
use navius_plugin::{Component, ComponentKey, ComponentRegistry, Scope};

// Create a component registry
let registry = ComponentRegistry::new();

// Register a singleton component
let service = MyService::new();
registry.register(
    ComponentKey::default::<MyService>(),
    service,
    Scope::Singleton
).unwrap();

// Register a named component
let another_service = MyService::new_with_config("special");
registry.register(
    ComponentKey::with_name::<MyService, _>("special"),
    another_service,
    Scope::Singleton
).unwrap();

// Get a component by type
let component: Component<MyService> = registry.get_by_type().unwrap();

// Get a component by name
let named_component: Component<MyService> = registry.get_by_name("special").unwrap();

// Use the component
component.instance().do_something();
```

### Scope Types

- **Singleton**: One instance shared by all consumers
- **Prototype**: New instance created for each consumer

```rust
// Register a prototype component
registry.register(
    ComponentKey::default::<MyFactory>(),
    MyFactory::new(),
    Scope::Prototype
).unwrap();

// Each call to get_by_type() will return a new instance
let factory1 = registry.get_by_type::<MyFactory>().unwrap();
let factory2 = registry.get_by_type::<MyFactory>().unwrap();
// factory1 and factory2 are different instances
```

## Integration with Navius Framework

This crate integrates with other Navius framework components:

- **navius-http**: Register HTTP handlers as components
- **navius-db**: Manage database connections in plugins
- **navius-cache**: Configure cache backends through plugins

## Example: HTTP Plugin

```rust
#[derive(Debug)]
struct HttpPlugin {
    id: String,
    description: String,
}

#[async_trait]
impl Plugin for HttpPlugin {
    // ... plugin implementation ...

    async fn initialize(&self, registry: &ComponentRegistry) -> PluginResult<()> {
        // Register HTTP handlers
        let user_handler = UserHandler::new();
        registry.register(
            ComponentKey::default::<UserHandler>(),
            user_handler,
            Scope::Singleton
        )?;

        // Create and register the router
        let router = create_router(registry)?;
        registry.register(
            ComponentKey::default::<Router>(),
            router,
            Scope::Singleton
        )?;
        
        Ok(())
    }
}

fn create_router(registry: &ComponentRegistry) -> PluginResult<Router> {
    let user_handler = registry.get_by_type::<UserHandler>()?;
    
    let router = Router::new()
        .route("/users", get(user_handler.instance().list_users))
        .route("/users/:id", get(user_handler.instance().get_user));
        
    Ok(router)
}
```

## License

MIT OR Apache-2.0 