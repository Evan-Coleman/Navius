---
title: "Plugin System Reference"
description: "Comprehensive reference documentation for the Navius Plugin System"
category: reference
tags:
  - plugins
  - component-registry
  - dependency-injection
  - lifecycle
  - api-reference
related:
  - 02_examples/plugin-system-example.md
  - 02_examples/dependency-injection-example.md
  - 05_reference/core-concepts.md
last_updated: June 30, 2025
version: 1.0
status: stable
---

# Plugin System Reference

The Navius Plugin System provides a structured way to extend your application with modular, reusable components. This reference document provides a comprehensive overview of the plugin system's architecture, APIs, and usage patterns.

## Core Concepts

### Plugin

A plugin is a discrete unit of functionality that can be added to a Navius application. Plugins can:

1. Register components in the component registry
2. Define dependencies on other plugins
3. Manage their own lifecycle (initialization and shutdown)
4. Access shared resources

### Component Registry

The component registry provides a centralized store for application components. It enables:

1. Type-safe registration and retrieval of components
2. Support for different component scopes (Singleton and Prototype)
3. Named component registration

### Lifecycle Management

The plugin system manages the lifecycle of plugins, ensuring that:

1. Plugins are initialized in the correct order based on dependencies
2. Resources are properly allocated during initialization
3. Resources are cleaned up during shutdown

## API Reference

### Plugin Trait

The core interface for defining plugins:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Plugin: Send + Sync + 'static {
    /// Returns the name of the plugin
    fn name(&self) -> &str;
    
    /// Returns a list of plugin names that this plugin depends on
    fn dependencies(&self) -> Vec<&str> { 
        Vec::new() 
    }
    
    /// Returns true if the plugin has been initialized
    fn is_initialized(&self) -> bool;
    
    /// Initializes the plugin and registers its components
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Shuts down the plugin and cleans up resources
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
```

### SimplePlugin

A helper implementation for common plugin functionality:

```rust
pub struct SimplePlugin {
    name: String,
    initialized: AtomicBool,
}

impl SimplePlugin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            initialized: AtomicBool::new(false),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }
    
    pub fn set_initialized(&self, value: bool) {
        self.initialized.store(value, Ordering::Relaxed);
    }
}
```

### PluginRegistry

Manages the collection of plugins and their dependencies:

```rust
pub struct PluginRegistry {
    plugins: DashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    /// Creates a new plugin registry
    pub fn new() -> Self { /* ... */ }
    
    /// Registers a plugin with the registry
    pub fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { /* ... */ }
    
    /// Finds a plugin by name
    pub fn find_plugin(&self, name: &str) -> Option<&dyn Plugin> { /* ... */ }
    
    /// Initializes all registered plugins in dependency order
    pub async fn initialize_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { /* ... */ }
    
    /// Shuts down all plugins in reverse initialization order
    pub async fn shutdown_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> { /* ... */ }
    
    /// Returns an iterator over all registered plugins
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Box<dyn Plugin>)> { /* ... */ }
}
```

### ComponentRegistry

Manages the registration and retrieval of components:

```rust
pub struct ComponentRegistry {
    components: DashMap<TypeId, DashMap<Option<String>, Box<dyn Any + Send + Sync>>>,
}

impl ComponentRegistry {
    /// Creates a new component registry
    pub fn new() -> Self { /* ... */ }
    
    /// Sets the global component registry
    pub fn set_global(registry: Arc<ComponentRegistry>) { /* ... */ }
    
    /// Gets the global component registry
    pub fn global() -> Arc<ComponentRegistry> { /* ... */ }
    
    /// Starts the registration of a component type
    pub fn register<T: ?Sized + 'static>(&self) -> ComponentRegistration<T> { /* ... */ }
    
    /// Gets a registered component by type
    pub fn get<T: ?Sized + 'static>(&self) -> Result<Arc<T>, ComponentError> { /* ... */ }
    
    /// Gets a named component by type and name
    pub fn get_named<T: ?Sized + 'static>(&self, name: &str) -> Result<Arc<T>, ComponentError> { /* ... */ }
    
    /// Checks if a component of the given type is registered
    pub fn contains<T: ?Sized + 'static>(&self) -> bool { /* ... */ }
    
    /// Removes a component by type
    pub fn remove<T: ?Sized + 'static>(&self) -> bool { /* ... */ }
    
    /// Removes a named component by type and name
    pub fn remove_named<T: ?Sized + 'static>(&self, name: &str) -> bool { /* ... */ }
    
    /// Clears all registered components
    pub fn clear(&self) { /* ... */ }
}
```

### ComponentRegistration

Fluent API for registering components:

```rust
pub struct ComponentRegistration<'a, T: ?Sized + 'static> {
    registry: &'a ComponentRegistry,
    component_type: PhantomData<T>,
    name: Option<String>,
    scope: Scope,
}

impl<'a, T: ?Sized + 'static> ComponentRegistration<'a, T> {
    /// Names the component being registered
    pub fn named(mut self, name: impl Into<String>) -> Self { /* ... */ }
    
    /// Sets the scope of the component
    pub fn with_scope(mut self, scope: Scope) -> Self { /* ... */ }
    
    /// Registers the component implementation
    pub fn with<U>(self, component: Arc<U>) -> Result<(), ComponentError> 
    where
        U: 'static + Send + Sync,
        U: AsRef<T>,
    { /* ... */ }
}
```

### Scope

Defines the lifecycle and instantiation behavior of components:

```rust
pub enum Scope {
    /// A single instance shared by all consumers
    Singleton,
    
    /// A new instance created for each consumer
    Prototype,
}
```

## Usage Examples

### Basic Plugin Definition

```rust
use navius_plugin::{Plugin, SimplePlugin};
use async_trait::async_trait;

pub struct LoggingPlugin {
    base: SimplePlugin,
}

impl LoggingPlugin {
    pub fn new() -> Self {
        Self {
            base: SimplePlugin::new("logging"),
        }
    }
}

#[async_trait]
impl Plugin for LoggingPlugin {
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn is_initialized(&self) -> bool {
        self.base.is_initialized()
    }
    
    async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_initialized() {
            return Ok(());
        }
        
        // Registration code here
        
        // Mark as initialized
        let mut plugin_self = unsafe { &mut *(self as *const _ as *mut Self) };
        plugin_self.base.set_initialized(true);
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.is_initialized() {
            return Ok(());
        }
        
        // Cleanup code here
        
        // Mark as not initialized
        let mut plugin_self = unsafe { &mut *(self as *const _ as *mut Self) };
        plugin_self.base.set_initialized(false);
        
        Ok(())
    }
}
```

### Registering Components

```rust
use navius_plugin::ComponentRegistry;
use std::sync::Arc;

// Define a service trait
trait EmailService: Send + Sync {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Implement the service
struct SmtpEmailService {
    smtp_server: String,
}

impl EmailService for SmtpEmailService {
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation details...
        Ok(())
    }
}

// Register the service
let registry = ComponentRegistry::global();
let service = Arc::new(SmtpEmailService { smtp_server: "smtp.example.com".to_string() });

registry.register::<dyn EmailService>()
    .with(service as Arc<dyn EmailService>)
    .expect("Failed to register email service");

// Use the service elsewhere
let email_service = registry.get::<dyn EmailService>()
    .expect("Failed to get email service");

email_service.send_email("user@example.com", "Hello", "This is a test email")
    .expect("Failed to send email");
```

### Using Named Components

```rust
use navius_plugin::ComponentRegistry;
use std::sync::Arc;

// Register multiple implementations of the same trait
let registry = ComponentRegistry::global();

// Register a development implementation
registry.register::<dyn EmailService>()
    .named("development")
    .with(Arc::new(MockEmailService::new()) as Arc<dyn EmailService>)
    .expect("Failed to register development email service");

// Register a production implementation
registry.register::<dyn EmailService>()
    .named("production")
    .with(Arc::new(SmtpEmailService::new("smtp.example.com")) as Arc<dyn EmailService>)
    .expect("Failed to register production email service");

// Get the appropriate implementation based on configuration
let config_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
let email_service = registry.get_named::<dyn EmailService>(&config_env)
    .expect("Failed to get email service");
```

### Component Scopes

```rust
use navius_plugin::{ComponentRegistry, Scope};
use std::sync::Arc;

// Register a singleton component (default)
registry.register::<dyn DatabaseConnection>()
    .with(Arc::new(PostgresConnection::new()) as Arc<dyn DatabaseConnection>)
    .expect("Failed to register database connection");

// Register a prototype component (new instance per consumer)
registry.register::<dyn RequestContext>()
    .with_scope(Scope::Prototype)
    .with(Arc::new(DefaultRequestContext::new()) as Arc<dyn RequestContext>)
    .expect("Failed to register request context");
```

## Best Practices

### Plugin Design

1. **Single Responsibility**: Each plugin should focus on a specific functionality domain
2. **Clear Dependencies**: Explicitly declare all dependencies through the `dependencies()` method
3. **Resource Management**: Initialize resources in `initialize()` and clean them up in `shutdown()`
4. **Error Handling**: Provide detailed error messages when operations fail

### Component Registration

1. **Use Interfaces**: Register components as trait objects for better decoupling
2. **Consider Scope**: Use singleton scope for stateless or shared services, and prototype scope for per-request services
3. **Naming Conventions**: When using named components, adopt a consistent naming scheme
4. **Avoid Circular Dependencies**: Design your component hierarchy to avoid circular dependencies

### Testing

1. **Mock Dependencies**: Use mock implementations of dependencies when testing plugins
2. **Test Isolation**: Test plugins in isolation using a dedicated component registry
3. **Lifecycle Testing**: Test both initialization and shutdown paths
4. **Error Handling**: Test error conditions and recovery mechanisms

## Error Handling

The plugin system provides several error types for different failure scenarios:

```rust
pub enum PluginError {
    /// A plugin with the given name already exists
    DuplicatePlugin(String),
    
    /// A plugin with the given name was not found
    PluginNotFound(String),
    
    /// A circular dependency was detected
    CircularDependency(Vec<String>),
    
    /// An error occurred during plugin initialization
    InitializationError(String, Box<dyn std::error::Error + Send + Sync>),
    
    /// An error occurred during plugin shutdown
    ShutdownError(String, Box<dyn std::error::Error + Send + Sync>),
}

pub enum ComponentError {
    /// A component of the given type was not found
    ComponentNotFound(String),
    
    /// A component with the given name was not found
    NamedComponentNotFound(String, String),
    
    /// The component could not be cast to the requested type
    TypeMismatch(String),
    
    /// An error occurred during component registration
    RegistrationError(String),
    
    /// An error occurred during component creation
    CreationError(String),
}
```

## Conclusion

The Navius Plugin System provides a powerful and flexible way to extend your application with modular, reusable components. By following the patterns and practices outlined in this reference, you can build applications that are easier to maintain, test, and extend over time.

For practical examples of the plugin system in action, see the [Plugin System Example](/02_examples/plugin-system-example.md). 