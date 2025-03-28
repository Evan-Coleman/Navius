---
title: "Database Service Example"
description: "Examples of using the generic database service interfaces and providers"
category: examples
tags:
  - database
  - service
  - generalization
  - providers
related:
  - examples/repository-pattern-example.md
  - roadmaps/25-generic-service-implementations.md
  - reference/patterns/repository-pattern.md
last_updated: March 27, 2025
version: 1.0
---

# Database Service Example

This example demonstrates how to use the generic database service implementation, including defining providers, configuring the service, and performing database operations.

## Overview

The Database Service implementation follows a provider-based architecture that enables:

- Abstracting database operations from specific implementations
- Supporting multiple database types through providers
- Configuration-based selection of database providers
- Easy testing with in-memory database implementations

## Core Components

The database service consists of several key components:

1. **DatabaseOperations Trait**: Defines core database operations
2. **DatabaseProvider Trait**: Defines interface for creating database instances
3. **DatabaseProviderRegistry**: Manages and creates database instances
4. **DatabaseConfig**: Configures database connection settings
5. **InMemoryDatabase**: Default in-memory implementation for testing

## Basic Usage

### Accessing the Database Service

The database service is accessible through the application's service registry:

```rust
use crate::core::services::ServiceRegistry;
use crate::core::services::database_service::DatabaseService;

// Get the service from service registry
let db_service = service_registry.get::<DatabaseService>();

// Use the database service
let result = db_service.create_database().await?;
```rust

### Performing Basic Operations

Once you have a database instance, you can perform operations:

```rust
// Get a value
let user_json = db.get("users", "user-123").await?;

// Set a value
db.set("users", "user-456", &user_json_string).await?;

// Delete a value
let deleted = db.delete("users", "user-789").await?;

// Query with a filter
let active_users = db.query("users", "status='active'").await?;
```rust

## Implementing a Custom Provider

You can implement your own database provider by implementing the `DatabaseProvider` trait:

```rust
use crate::core::services::database_service::{DatabaseOperations, DatabaseProvider};
use crate::core::services::error::ServiceError;
use async_trait::async_trait;

pub struct MyCustomDatabaseProvider;

#[async_trait]
impl DatabaseProvider for MyCustomDatabaseProvider {
    type Database = MyCustomDatabase;
    
    async fn create_database(&self, config: DatabaseConfig) -> Result<Self::Database, ServiceError> {
        // Create and return your database implementation
        Ok(MyCustomDatabase::new(config))
    }
    
    fn supports(&self, config: &DatabaseConfig) -> bool {
        config.provider_type == "custom"
    }
}

pub struct MyCustomDatabase {
    // Your database implementation details
}

#[async_trait]
impl DatabaseOperations for MyCustomDatabase {
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        // Implement get operation
    }
    
    // Implement other required operations...
}
```rust

## Registering a Provider

Register your custom provider with the database service:

```rust
use crate::core::services::database_service::DatabaseProviderRegistry;

// Create a registry
let mut registry = DatabaseProviderRegistry::new();

// Register your provider
registry.register("custom", MyCustomDatabaseProvider);

// Create the database service with the registry
let db_service = DatabaseService::new(registry);
```rust

## Using the In-Memory Database

The in-memory database provider is useful for testing:

```rust
use crate::core::services::memory_database::{InMemoryDatabaseProvider, InMemoryDatabase};

#[tokio::test]
async fn test_database_operations() {
    // Create a provider and configuration
    let provider = InMemoryDatabaseProvider::new();
    let config = DatabaseConfig::default().with_provider("memory");
    
    // Create a database instance
    let db = provider.create_database(config).await.unwrap();
    
    // Set a test value
    db.set("test", "key1", "value1").await.unwrap();
    
    // Get the value back
    let value = db.get("test", "key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));
}
```rust

## Configuration

Configure the database service in your application configuration:

```yaml
# In config/default.yaml
database:
  provider: memory  # Could be postgres, mongodb, etc.
  connection_string: ""
  max_connections: 10
  connection_timeout_ms: 5000
  retry_attempts: 3
  enable_logging: true
```rust

Loading the configuration:

```rust
use crate::core::config::AppConfig;
use crate::core::services::database_service::DatabaseConfig;

// Load from application config
let app_config = AppConfig::load()?;
let db_config = DatabaseConfig::from_app_config(&app_config);

// Or create it programmatically
let db_config = DatabaseConfig::default()
    .with_provider("postgres")
    .with_connection_string("postgres://user:pass@localhost/dbname")
    .with_max_connections(20);
```rust

## Complete Example

Here's a complete example showing how to set up and use the database service:

```rust
use crate::core::services::database_service::{
    DatabaseService, DatabaseConfig, DatabaseProviderRegistry
};
use crate::core::services::memory_database::register_memory_database_provider;

async fn setup_database_service() -> Result<DatabaseService, ServiceError> {
    // Create a provider registry
    let mut registry = DatabaseProviderRegistry::new();
    
    // Register the built-in memory provider
    register_memory_database_provider(&mut registry);
    
    // Create configuration
    let config = DatabaseConfig::default()
        .with_provider("memory")
        .with_max_connections(5);
    
    // Create service
    let service = DatabaseService::new(registry)
        .with_default_config(config);
    
    // Initialize the service
    service.init().await?;
    
    Ok(service)
}

async fn example_usage(service: &DatabaseService) -> Result<(), ServiceError> {
    // Create a database instance
    let db = service.create_database().await?;
    
    // Store user data
    let user_data = r#"{"id":"user-123","name":"Alice","role":"admin"}"#;
    db.set("users", "user-123", user_data).await?;
    
    // Retrieve user data
    if let Some(data) = db.get("users", "user-123").await? {
        println!("User data: {}", data);
    }
    
    // Query users by role
    let admins = db.query("users", "role='admin'").await?;
    println!("Found {} admin users", admins.len());
    
    Ok(())
}
```rust

## Best Practices

1. **Provider Selection**: Choose the appropriate provider based on your requirements
2. **Error Handling**: Always handle database errors properly
3. **Connection Management**: Reuse database connections where possible
4. **Testing**: Use the in-memory database for testing
5. **Configuration**: Externalize database configuration
6. **Transactions**: Use transactions for multi-step operations
7. **Security**: Always sanitize input to prevent injection attacks

## Related Documentation

- [Repository Pattern Example](repository-pattern-example.md)
- [Generic Service Implementations Roadmap](../roadmaps/25-generic-service-implementations.md)
- [Database API Reference](../reference/api/database-api.md) 