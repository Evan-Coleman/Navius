# Navius Dependency Injection

A lightweight dependency injection system for Navius applications, based on the research of spring-rs.

## Features

- **Component Registry**: Type-safe dependency resolution with comprehensive lifecycle management
- **Scoped Components**: Support for singleton, prototype, request, and session scopes
- **Lifecycle Hooks**: Synchronous and asynchronous lifecycle hooks for components
- **Qualifier Support**: Component disambiguation using qualifiers
- **Factory-Based Components**: Support for factory functions to create components
- **Application Framework**: Comprehensive application bootstrapping with configuration and plugin support
- **Configuration Binding**: Prefix-based configuration binding with automatic type conversion
- **Plugin System**: Extensible plugin system for application components

## Usage

### Basic Component Registration

```rust
use navius_di::{ComponentRegistry, Result};

// Define a component
struct DatabaseService {
    connection_string: String,
}

impl DatabaseService {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

// Register and use the component
fn main() -> Result<()> {
    let registry = ComponentRegistry::new();
    
    // Register the component
    registry.register(DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()))?;
    
    // Get the component
    let db_service = registry.get::<DatabaseService>()?;
    println!("Database connection: {}", db_service.connection_string);
    
    Ok(())
}
```

### Component Lifecycle

```rust
use navius_di::{ComponentRegistry, Lifecycle, Result};

struct DatabaseService {
    connection_string: String,
}

// Implement lifecycle hooks
impl Lifecycle for DatabaseService {
    fn on_initialize(&self) -> Result<()> {
        println!("Initializing database connection to {}", self.connection_string);
        // Connect to the database
        Ok(())
    }
    
    fn on_destroy(&self) -> Result<()> {
        println!("Closing database connection to {}", self.connection_string);
        // Close the connection
        Ok(())
    }
}
```

### Factory Registration

```rust
use navius_di::{ComponentRegistry, ComponentScope, Result};

// Register with a factory function
fn main() -> Result<()> {
    let registry = ComponentRegistry::new();
    
    // Register a singleton component with a factory
    registry.register_with_factory(
        || DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()),
        ComponentScope::Singleton,
    );
    
    // Get the component
    let db_service = registry.get::<DatabaseService>()?;
    
    Ok(())
}
```

### Application Builder

```rust
use navius_di::{Application, Result};
use serde::Deserialize;

// Define a configuration structure
#[derive(Debug, Clone, Deserialize)]
struct AppConfig {
    name: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create and configure an application
    let app = Application::builder()
        // Set configuration values
        .with_config("app.name", "My Application".to_string())
        .with_config("app.version", "1.0.0".to_string())
        
        // Register components
        .with_factory(
            || DatabaseService::new("jdbc:postgresql://localhost:5432/mydb".to_string()),
            ComponentScope::Singleton,
        )
        
        // Build the application
        .build()
        .await?;
    
    // Get configuration
    let config = app.config::<AppConfig>("app")?;
    println!("Application: {} v{}", config.name, config.version);
    
    // Get components
    let db_service = app.get::<DatabaseService>()?;
    
    // Shutdown the application
    app.shutdown().unwrap();
    
    Ok(())
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
navius-di = { path = "../navius-di" }
```

## License

Licensed under the Apache License, Version 2.0. 