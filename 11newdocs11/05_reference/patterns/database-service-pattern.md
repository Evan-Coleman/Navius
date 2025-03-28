---
title: "Database Service Pattern"
description: "Design and implementation of the database service pattern with pluggable providers"
category: patterns
tags:
  - patterns
  - database
  - architecture
  - providers
related:
  - reference/patterns/repository-pattern.md
  - reference/api/database-api.md
  - examples/database-service-example.md
last_updated: March 27, 2025
version: 1.0
---

# Database Service Pattern

## Overview

The Database Service Pattern provides a generic abstraction for database operations with pluggable provider implementations. This enables applications to work with different database technologies through a consistent interface while allowing for easy switching between implementations.

## Problem Statement

Applications typically need to interact with databases, but direct coupling to specific database technologies creates several challenges:

- Difficult to switch between database providers (e.g., PostgreSQL to MongoDB)
- Testing is complicated by dependencies on actual database instances
- Code becomes tightly coupled to specific database APIs
- Difficult to implement caching or other cross-cutting concerns
- Limited ability to leverage different databases for different use cases

## Solution: Database Service Pattern with Pluggable Providers

The Database Service Pattern in Navius uses a provider-based architecture with these components:

1. **DatabaseOperations Trait**: Defines core database operations
2. **DatabaseProvider Trait**: Creates database instances
3. **DatabaseProviderRegistry**: Manages and selects appropriate providers
4. **DatabaseConfig**: Configures database connections and behavior
5. **DatabaseService**: Orchestrates database operations

### Pattern Structure

```
┌────────────────────┐     creates     ┌─────────────────────┐
│  DatabaseService   │─────────────────│DatabaseProviderRegistry│
└────────┬───────────┘                 └─────────┬───────────┘
         │                                       │ selects
         │                                       ▼
         │                             ┌─────────────────────┐
         │                             │  DatabaseProvider   │
         │                             └─────────┬───────────┘
         │                                       │ creates
         │                                       ▼
         │ uses                        ┌─────────────────────┐
         └─────────────────────────────│ DatabaseOperations  │
                                       └─────────────────────┘
```

### Implementation

#### 1. Database Operations Interface

The `DatabaseOperations` trait defines the contract for all database implementations:

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

#### 2. Database Provider Interface

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

#### 3. Database Service

The `DatabaseService` manages database instances and provides access to them:

```rust
pub struct DatabaseService {
    provider_registry: Arc<RwLock<DatabaseProviderRegistry>>,
    default_config: DatabaseConfig,
}

impl DatabaseService {
    pub fn new(registry: DatabaseProviderRegistry) -> Self {
        Self {
            provider_registry: Arc::new(RwLock::new(registry)),
            default_config: DatabaseConfig::default(),
        }
    }
    
    pub fn with_default_config(mut self, config: DatabaseConfig) -> Self {
        self.default_config = config;
        self
    }
    
    pub async fn create_database(&self) -> Result<Box<dyn DatabaseOperations>, ServiceError> {
        // Use registry to create appropriate database instance
    }
}
```

#### 4. Provider Registry

The `DatabaseProviderRegistry` stores available providers and selects the appropriate one:

```rust
pub struct DatabaseProviderRegistry {
    providers: HashMap<String, Box<dyn AnyDatabaseProvider>>,
}

impl DatabaseProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }
    
    pub fn register<P: DatabaseProvider + 'static>(&mut self, name: &str, provider: P) {
        self.providers.insert(name.to_string(), Box::new(provider));
    }
    
    pub async fn create_database(
        &self, 
        provider_name: &str, 
        config: DatabaseConfig
    ) -> Result<Box<dyn DatabaseOperations>, ServiceError> {
        // Find provider and create database
    }
}
```

## Benefits

1. **Abstraction**: Decouples application from specific database technologies
2. **Testability**: Simplifies testing with in-memory database implementations
3. **Flexibility**: Easy to switch between database providers
4. **Consistency**: Provides uniform interface for different database technologies
5. **Extensibility**: New database providers can be added without changing client code
6. **Cross-Cutting Concerns**: Enables adding logging, metrics, and caching consistently

## Implementation Considerations

### 1. Transaction Support

Different databases have different transaction models:

- Relational databases have ACID transactions
- Some NoSQL databases have limited transaction support
- In-memory implementations may need to simulate transactions

The pattern should provide a consistent abstraction that works across different implementations:

```rust
// Example transaction usage
db.transaction(|tx| {
    tx.set("users", "user-1", r#"{"name":"Alice"}"#)?;
    tx.set("accounts", "account-1", r#"{"owner":"user-1","balance":100}"#)?;
    Ok(())
}).await?;
```

### 2. Query Language

Database technologies use different query languages (SQL, NoSQL query APIs). The pattern should provide:

- A simple string-based query interface for basic filtering
- Support for native query formats where needed
- Helpers for common query patterns

### 3. Connection Pooling

Database connections are often expensive resources:

- Implement connection pooling in database providers
- Configure pool sizes and connection timeouts
- Handle connection errors gracefully

### 4. Error Handling

Database errors should be mapped to application-specific errors:

- Create meaningful error categories (NotFound, Conflict, etc.)
- Include useful context in error messages
- Avoid exposing internal database details in errors

## Example Implementations

### In-Memory Database

```rust
pub struct InMemoryDatabase {
    data: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

#[async_trait]
impl DatabaseOperations for InMemoryDatabase {
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        let data = self.data.read().await;
        if let Some(collection_data) = data.get(collection) {
            return Ok(collection_data.get(key).cloned());
        }
        Ok(None)
    }
    
    async fn set(&self, collection: &str, key: &str, value: &str) -> Result<(), ServiceError> {
        let mut data = self.data.write().await;
        let collection_data = data.entry(collection.to_string()).or_insert_with(HashMap::new);
        collection_data.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    // Other methods implementation...
}
```

### PostgreSQL Database

```rust
pub struct PostgresDatabase {
    pool: PgPool,
}

#[async_trait]
impl DatabaseOperations for PostgresDatabase {
    async fn get(&self, collection: &str, key: &str) -> Result<Option<String>, ServiceError> {
        let query = format!(
            "SELECT data FROM {} WHERE id = $1",
            sanitize_identifier(collection)
        );
        
        let result = sqlx::query_scalar(&query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ServiceError::database_error(e.to_string()))?;
            
        Ok(result)
    }
    
    // Other methods implementation...
}
```

## API Example

```rust
// Get the database service
let db_service = service_registry.get::<DatabaseService>();

// Create a database instance
let db = db_service.create_database().await?;

// Store user data
let user_data = r#"{"id":"user-123","name":"Alice","role":"admin"}"#;
db.set("users", "user-123", user_data).await?;

// Retrieve user data
if let Some(data) = db.get("users", "user-123").await? {
    let user: User = serde_json::from_str(&data)?;
    println!("Found user: {}", user.name);
}

// Query users by role
let admins = db.query("users", "role='admin'").await?;
println!("Found {} admin users", admins.len());

// Execute a transaction
db.transaction(|tx| {
    tx.set("users", "user-1", r#"{"name":"Alice"}"#)?;
    tx.set("accounts", "account-1", r#"{"owner":"user-1","balance":100}"#)?;
    Ok(())
}).await?;
```

## Related Patterns

- **Repository Pattern**: Often used with Database Service Pattern to provide domain-specific data access
- **Factory Pattern**: Used to create database instances
- **Strategy Pattern**: Different database providers implement different strategies
- **Adapter Pattern**: Adapts specific database APIs to the common interface
- **Builder Pattern**: Used for configuration building

## References

- [Generic Repository Pattern](https://www.martinfowler.com/eaaCatalog/repository.html)
- [Data Mapper Pattern](https://www.martinfowler.com/eaaCatalog/dataMapper.html)
- [JDBC in Java](https://docs.oracle.com/javase/tutorial/jdbc/basics/index.html)
- [SQLx in Rust](https://github.com/launchbadge/sqlx) 