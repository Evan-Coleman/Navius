---
title: "Database API Reference"
description: "API documentation for Navius database service and operations"
category: api
tags:
  - api
  - database
  - storage
  - repository
related:
  - reference/patterns/database-service-pattern.md
  - examples/database-service-example.md
  - reference/patterns/repository-pattern.md
last_updated: March 26, 2024
version: 1.0
---

# Database API Reference

## Overview

The Database API provides a generic interface for interacting with databases through the Database Service. While this is primarily a programmatic API rather than a REST API, this reference documents the core interfaces, operations, and usage patterns for working with the Database Service.

## Core Interfaces

### DatabaseOperations

The `DatabaseOperations` trait defines the core operations available for all database implementations:

```rust
#[async_trait]
pub trait DatabaseOperations: Send + Sync {
    /// Get a value from the database
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError>;
    
    /// Set a value in the database
    async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError>;
    
    /// Delete a value from the database
    async fn delete(&self, collection: &str, key: &str) -> Result<bool, ServiceError>;
    
    /// Query the database with a filter
    async fn query(&self, collection: &str, filter: &str) -> Result<Vec<String>, ServiceError>;
    
    /// Execute a database transaction with multiple operations
    async fn transaction<F, T>(&self, operations: F) -> Result<T, ServiceError>
    where
        F: FnOnce(&dyn DatabaseOperations) -> Result<T, ServiceError> + Send + 'static,
        T: Send + 'static;
}
```

### DatabaseProvider

The `DatabaseProvider` trait enables creating database instances:

```rust
#[async_trait]
pub trait DatabaseProvider: Send + Sync {
    /// The type of database this provider creates
    type Database: DatabaseOperations;
    
    /// Create a new database instance
    async fn create_database(&self, config: DatabaseConfig) -> Result<Self::Database, ServiceError>;
    
    /// Check if this provider supports the given configuration
    fn supports(&self, config: &DatabaseConfig) -> bool;
    
    /// Get the name of this provider
    fn name(&self) -> &str;
}
```

### DatabaseService

The `DatabaseService` manages database instances:

```rust
pub struct DatabaseService {
    provider_registry: Arc<RwLock<DatabaseProviderRegistry>>,
    default_config: DatabaseConfig,
}

impl DatabaseService {
    pub fn new(registry: DatabaseProviderRegistry) -> Self {
        // Implementation details...
    }
    
    pub fn with_default_config(mut self, config: DatabaseConfig) -> Self {
        // Implementation details...
    }
    
    pub async fn create_database(&self) -> Result<Box<dyn DatabaseOperations>, ServiceError> {
        // Implementation details...
    }
}
```

## Using the Database API

### Accessing the Database Service

The database service is accessible through the application's service registry:

```rust
use crate::core::services::ServiceRegistry;
use crate::core::services::database_service::DatabaseService;

// Get the service from service registry
let db_service = service_registry.get::<DatabaseService>();

// Create a database instance
let db = db_service.create_database().await?;
```

### Basic CRUD Operations

#### Creating/Updating Records

```rust
// Create a new user record
let user = User {
    id: "user-123",
    name: "Alice",
    email: "alice@example.com",
};

// Serialize to JSON
let user_json = serde_json::to_string(&user)?;

// Store in database
db.set("users", &user.id, &user_json).await?;
```

#### Reading Records

```rust
// Get a user by ID
if let Some(user_json) = db.get("users", "user-123").await? {
    // Deserialize from JSON
    let user: User = serde_json::from_str(&user_json)?;
    println!("Found user: {}", user.name);
}
```

#### Querying Records

```rust
// Query users with role=admin
let admin_users = db.query("users", "role='admin'").await?;

// Process results
for user_json in admin_users {
    let user: User = serde_json::from_str(&user_json)?;
    println!("Admin user: {}", user.name);
}
```

#### Deleting Records

```rust
// Delete a user
let deleted = db.delete("users", "user-123").await?;
if deleted {
    println!("User deleted successfully");
} else {
    println!("User not found");
}
```

### Transactions

Transactions allow multiple operations to be executed atomically:

```rust
// Execute a transaction
db.transaction(|tx| {
    // Create a new user
    tx.set("users", "user-1", r#"{"name":"Alice","balance":0}"#)?;
    
    // Create an account for the user
    tx.set("accounts", "account-1", r#"{"owner":"user-1","balance":100}"#)?;
    
    // Create initial transaction record
    tx.set("transactions", "tx-1", 
           r#"{"account":"account-1","amount":100,"type":"deposit"}"#)?;
    
    Ok(())
}).await?;
```

### Using Repository Pattern

The Database API is typically used via the Repository pattern:

```rust
use crate::core::models::{Entity, Repository};
use crate::core::services::repository_service::GenericRepository;

// Create a repository for User entities
let user_repo = GenericRepository::<User>::with_service(&repository_service).await?;

// Create a new user
let mut user = User::new("Alice", "alice@example.com");

// Save the user
let saved_user = user_repo.save(&user).await?;

// Find a user by ID
if let Some(found_user) = user_repo.find_by_id(&user.id).await? {
    println!("Found user: {}", found_user.name);
}

// Delete a user
let deleted = user_repo.delete(&user.id).await?;
```

## Available Database Providers

### InMemoryDatabaseProvider

The in-memory database provider is useful for development and testing:

```rust
use crate::core::services::memory_database::InMemoryDatabaseProvider;

// Create a provider
let provider = InMemoryDatabaseProvider::new();

// Create a database instance
let config = DatabaseConfig::default();
let db = provider.create_database(config).await?;
```

### PostgresDatabaseProvider

When PostgreSQL integration is enabled, the PostgreSQL provider is available:

```rust
use crate::core::services::postgres_database::PostgresDatabaseProvider;

// Create a provider with connection string
let provider = PostgresDatabaseProvider::new("postgres://user:pass@localhost/dbname");

// Create a database instance
let config = DatabaseConfig::default();
let db = provider.create_database(config).await?;
```

## Configuration

The Database Service can be configured in `config/default.yaml`:

```yaml
# Database configuration
database:
  # Default provider to use
  provider: memory
  
  # Provider-specific configurations
  providers:
    memory:
      enabled: true
      
    postgres:
      enabled: true
      connection_string: "postgres://user:pass@localhost/dbname"
      max_connections: 10
      connection_timeout_ms: 5000
      idle_timeout_ms: 300000
      
  # Common settings
  common:
    query_timeout_ms: 3000
    log_queries: true
```

## Error Handling

The Database API uses `ServiceError` for error handling:

```rust
// Example error handling
match db.get("users", "user-123").await {
    Ok(Some(user_json)) => {
        // Process user
    },
    Ok(None) => {
        // User not found
        println!("User not found");
    },
    Err(e) => {
        match e {
            ServiceError::DatabaseError { message, .. } => {
                // Handle database error
                println!("Database error: {}", message);
            },
            ServiceError::NotFound { message } => {
                // Handle not found error
                println!("Not found: {}", message);
            },
            _ => {
                // Handle other errors
                println!("Error: {}", e);
            }
        }
    }
}
```

## Implementing a Custom Provider

You can implement your own database provider by implementing the `DatabaseProvider` trait:

```rust
use crate::core::services::database_service::{
    DatabaseOperations, DatabaseProvider, DatabaseConfig
};
use crate::core::services::error::ServiceError;
use async_trait::async_trait;

// Custom database implementation
pub struct CustomDatabase {
    // Implementation details...
}

#[async_trait]
impl DatabaseOperations for CustomDatabase {
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        // Implementation...
    }
    
    // Other methods...
}

// Custom provider
pub struct CustomDatabaseProvider {
    // Provider details...
}

#[async_trait]
impl DatabaseProvider for CustomDatabaseProvider {
    type Database = CustomDatabase;
    
    async fn create_database(&self, config: DatabaseConfig) -> Result<Self::Database, ServiceError> {
        // Implementation...
    }
    
    fn supports(&self, config: &DatabaseConfig) -> bool {
        config.provider_type == "custom"
    }
    
    fn name(&self) -> &str {
        "custom"
    }
}
```

Register your custom provider:

```rust
let mut registry = DatabaseProviderRegistry::new();
registry.register("custom", CustomDatabaseProvider::new());

let service = DatabaseService::new(registry);
```

## Best Practices

### Collection Naming

- Use lowercase, plural nouns for collection names (e.g., `users`, `accounts`)
- Use dashes instead of spaces or underscores (e.g., `order-items`)
- Keep collection names consistent across the application

### Key Generation

- Use UUIDs or other globally unique identifiers for keys
- Consider using prefixed keys for better organization (e.g., `user-123`)
- Be consistent with key formats within each collection

### JSON Serialization

- Use serde for JSON serialization/deserialization
- Define clear schema for each collection's documents
- Include version information in documents for schema evolution
- Consider using compression for large documents

### Query Patterns

- Keep queries simple and specific to indexes
- Use appropriate filters to minimize data transfer
- Consider pagination for large result sets
- Use transactions for operations that must be atomic

### Error Handling

- Handle both expected errors (e.g., not found) and unexpected errors
- Provide appropriate context in error messages
- Consider retrying transient errors (e.g., connection issues)
- Don't expose internal database errors to users

## Performance Considerations

- Use connection pooling for database connections
- Cache frequently accessed data
- Use batch operations for multiple records
- Consider data access patterns when designing schemas
- Use appropriate indexes for frequent queries
- Monitor query performance and optimize as needed

## Related Documentation

- [Database Service Pattern](../patterns/database-service-pattern.md)
- [Repository Pattern](../patterns/repository-pattern.md)
- [Database Service Example](../../examples/database-service-example.md)
