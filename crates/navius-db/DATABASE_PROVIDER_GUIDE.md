# Database Provider Implementation Guide

This document outlines the guidelines and requirements for implementing a database provider for the Navius framework. Database providers implement the interfaces defined in `navius-db` for specific database systems.

## Architecture Overview

Navius uses a provider-based architecture for database access:

1. **navius-db**: Core interfaces and database-agnostic functionality
2. **navius-db-[provider]**: Implementation for a specific database system (e.g., `navius-db-postgres`)

This separation allows for:
- Clean dependency management
- Support for multiple database backends
- Reduced compile times for applications not using specific databases
- Better testability through mock implementations

## Creating a New Provider

### 1. Crate Structure

Create a new crate with the naming convention `navius-db-[provider]` where `[provider]` is the name of the database system (e.g., `mysql`, `sqlite`).

```
crates/navius-db-[provider]/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # Main entry point and exports
    ├── config.rs        # Provider-specific configuration
    ├── connection.rs    # Connection management
    ├── error.rs         # Error handling
    ├── pool.rs          # Connection pooling
    ├── query.rs         # Query building and execution
    ├── repository.rs    # Repository pattern implementation
    └── transaction.rs   # Transaction handling
```

### 2. Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
# Core dependencies
navius-db = { path = "../navius-db", version = "0.1.0" }
navius-core = { path = "../navius-core", version = "0.1.0" }
async-trait = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
uuid = { workspace = true }
thiserror = { workspace = true }

# Provider-specific dependencies
# For example, for MySQL:
# mysql = "X.Y.Z"
```

### 3. Required Implementations

Each provider must implement the following:

#### Provider Struct

```rust
/// Provider for navius-db
pub struct [Provider]Provider;

impl navius_db::DatabaseProvider for [Provider]Provider {
    fn name(&self) -> &'static str {
        "[provider-name]" // e.g., "mysql"
    }
    
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
```

#### Core Interfaces

Implement all of these interfaces:

1. **DatabasePool**: Connection pooling
2. **DatabaseConnection**: Database connection management
3. **DatabaseTransaction**: Transaction management
4. **DatabaseRowSet**: Result set handling
5. **Repository<T>**: Entity repository implementation

### 4. Naming Conventions

Use a consistent prefix for all types (e.g., `Pg` for PostgreSQL, `My` for MySQL):

```rust
pub struct [Prefix]Pool { /* ... */ }
pub struct [Prefix]Connection { /* ... */ }
pub struct [Prefix]Transaction { /* ... */ }
pub struct [Prefix]Row { /* ... */ }
pub struct [Prefix]Repository<T> { /* ... */ }
```

### 5. Error Handling

Follow these guidelines for error handling:

1. Create a provider-specific error type that wraps provider-specific errors
2. Implement conversion to and from `navius_db::DatabaseError`
3. Use structured error types, not string-based errors
4. Follow the error handling guidelines in `.cursor/rules/022-error-handling.mdc`

Example:

```rust
#[derive(Debug, Error)]
pub enum [Prefix]DatabaseError {
    #[error("Connection error: {0}")]
    ConnectionError(#[from] [Provider]Error),
    
    // Other provider-specific errors
}

impl From<[Prefix]DatabaseError> for navius_db::DatabaseError {
    fn from(err: [Prefix]DatabaseError) -> Self {
        match err {
            [Prefix]DatabaseError::ConnectionError(e) => 
                navius_db::DatabaseError::ConnectionError(e.to_string()),
            // Map other errors appropriately
        }
    }
}
```

### 6. Configuration

Provide a provider-specific configuration that extends or wraps the core `DatabaseConfig`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct [Prefix]DatabaseConfig {
    // Provider-specific configuration options
    pub provider_specific_setting: String,
    
    // Common settings delegated to core config
    #[serde(flatten)]
    pub core: navius_db::DatabaseConfig,
}

impl [Prefix]DatabaseConfig {
    pub fn new(url: String) -> Self {
        Self {
            provider_specific_setting: String::new(),
            core: navius_db::DatabaseConfig::new(url),
        }
    }
    
    // Provider-specific configuration methods
}
```

### 7. Public Exports

In your `lib.rs`, export all the public interfaces:

```rust
// Re-export provider library for convenience (if applicable)
pub use [provider_lib];

// Internal modules
mod config;
mod connection;
mod error;
mod pool;
mod query;
mod repository;
mod transaction;

// Public exports
pub use config::[Prefix]DatabaseConfig;
pub use connection::[Prefix]ConnectionManager;
pub use error::[Prefix]DatabaseError;
pub use pool::{[Prefix]Pool, [Prefix]Connection, [Prefix]Row};
pub use query::{[Prefix]Query, [Prefix]QueryExecutor};
pub use repository::[Prefix]Repository;
pub use transaction::[Prefix]Transaction;
pub use [provider]_provider::[Provider]Provider;
```

### 8. Documentation

Provide comprehensive documentation:

1. Update the `README.md` with:
   - Features and capabilities
   - Usage examples
   - Configuration options
   - Limitations or differences from other providers

2. Add extensive doc comments to all public types and functions

## Testing Requirements

Each provider must include thorough testing:

1. **Unit Tests**: For individual components
2. **Integration Tests**: For database operations
3. **Mock Tests**: For testing without a database
4. **Error Handling Tests**: For proper error propagation

Example test structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit_tests {
        use super::*;
        // Test individual components
    }
    
    mod integration_tests {
        use super::*;
        // Test database operations (requires database)
    }
    
    mod mock_tests {
        use super::*;
        // Test with mock database
    }
    
    mod error_tests {
        use super::*;
        // Test error handling
    }
}
```

## Example Provider

The `navius-db-postgres` crate serves as a reference implementation for new providers. Refer to its code structure and approach when implementing a new provider.

## Adding to the Workspace

Once your provider is implemented:

1. Add it to the workspace members in the root `Cargo.toml`
2. Add it to the root dependencies in the root `Cargo.toml`
3. Update the documentation to include the new provider

## Guidance for Specific Database Systems

### MySQL

- Use the `mysql` or `sqlx` (with MySQL features) crate
- Handle MySQL-specific types and error codes
- Implement connection pooling appropriate for MySQL

### SQLite

- Use the `rusqlite` or `sqlx` (with SQLite features) crate
- Handle SQLite's simpler connection model
- Consider file-based configuration instead of URL-based

### MongoDB

- Use the `mongodb` crate
- Adapt the repository pattern for document databases
- Handle MongoDB's non-SQL query language and schema-less documents

### Others

- Follow database-specific best practices
- Maintain consistent interfaces with other providers
- Document any provider-specific behaviors or limitations 