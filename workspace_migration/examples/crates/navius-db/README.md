# Navius DB

Database functionality for the Navius framework. This crate provides database connectivity, query execution, and ORM functionality with pluggable database backends.

## Features

- Database-agnostic interfaces and traits
- Connection management
- Transaction handling
- Repository pattern
- Query building
- Entity management
- Pluggable database providers

## Architecture

Navius DB uses a provider-based architecture that separates the database interfaces from their implementations:

- **navius-db**: Core interfaces and traits
- **navius-db-postgres**: PostgreSQL implementation using SQLx
- Future providers can be added for other database systems

## Usage

Add navius-db and a specific implementation (like navius-db-postgres) to your dependencies:

```toml
[dependencies]
navius-db = "0.1.0"
navius-db-postgres = "0.1.0"  # If using PostgreSQL
```

### Working with Entities

Define your entities using the `Entity` trait:

```rust
use navius_db::Entity;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    email: String,
}

impl Entity for User {
    fn id(&self) -> Uuid {
        self.id
    }
    
    fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }
    
    fn table_name() -> &'static str {
        "users"
    }
}
```

### Database Configuration

Create a database configuration:

```rust
use navius_db::DatabaseConfig;

fn create_config() -> DatabaseConfig {
    let mut config = DatabaseConfig::new("postgres://user:password@localhost:5432/mydb".to_string());
    config.max_connections = 20;
    config.min_connections = 5;
    config.run_migrations = true;
    config.migrations_path = "migrations".to_string();
    config
}
```

### Repository Pattern

Use the Repository pattern to interact with entities:

```rust
use navius_db::{Repository, DatabaseResult};
use uuid::Uuid;

async fn repository_example<R: Repository<User>>(repo: &R) -> DatabaseResult<()> {
    // Create a new user
    let mut user = User {
        id: Uuid::nil(), // Will be auto-generated
        username: "johndoe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    // Save user to database
    user = repo.save(&user).await?;
    println!("Created user with ID: {}", user.id());
    
    // Find user by ID
    let found_user = repo.find_by_id(user.id()).await?;
    println!("Found user: {:?}", found_user);
    
    // Find all users
    let all_users = repo.find_all().await?;
    println!("Found {} users", all_users.len());
    
    // Delete user
    repo.delete(user.id()).await?;
    println!("User deleted");
    
    Ok(())
}
```

### Transaction Management

The `navius-db` crate provides robust transaction management capabilities, including:

- Standard transactions with commit/rollback
- Savepoints for partial rollback within transactions
- Nested transactions using savepoints
- Automatic retry logic for transient errors

### Basic Transaction Usage

```rust
// Execute a transaction
let result = db.transaction(|mut tx| async move {
    // Execute queries within the transaction
    tx.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
    tx.execute("INSERT INTO logs (message) VALUES ('User created')").await?;
    
    // Commit happens automatically when the closure completes successfully
    // Rollback happens automatically if an error is returned or the transaction is dropped
    Ok(())
}).await;
```

### Savepoints

Savepoints allow you to create checkpoints within a transaction that you can later roll back to if needed.

```rust
db.transaction(|mut tx| async move {
    // First part of transaction
    tx.execute("INSERT INTO logs (message) VALUES ('Starting operation')").await?;
    
    // Create a savepoint
    tx.savepoint("before_risky_part").await?;
    
    // Try operation that might fail
    let result = tx.execute("UPDATE accounts SET status = 'PROCESSING' WHERE id = 123").await;
    
    if result.is_err() {
        // Roll back to savepoint if the operation failed
        tx.rollback_to_savepoint("before_risky_part").await?;
        
        // Try an alternative approach
        tx.execute("INSERT INTO logs (message) VALUES ('Trying alternative approach')").await?;
        // ...
    } else {
        // Operation succeeded, release the savepoint
        tx.release_savepoint("before_risky_part").await?;
    }
    
    Ok(())
}).await
```

### Nested Transactions

You can use nested transactions to create transaction-like semantics within a transaction:

```rust
db.transaction(|mut tx| async move {
    // Main transaction operations
    tx.execute("INSERT INTO logs (message) VALUES ('Starting parent operation')").await?;
    
    // Begin a nested transaction
    tx.nested(|| async {
        // Operations in nested transaction
        tx.execute("INSERT INTO logs (message) VALUES ('Starting nested operation')").await?;
        
        // This will only be rolled back if the nested transaction fails
        let result = tx.execute("UPDATE accounts SET status = 'PROCESSING' WHERE id = 123").await;
        
        if let Err(e) = result {
            // Return the error, which will rollback the nested transaction
            return Err(e);
        }
        
        Ok(())
    }).await?;
    
    // Continue with parent transaction, even if nested transaction failed
    tx.execute("INSERT INTO logs (message) VALUES ('Parent operation continuing')").await?;
    
    Ok(())
}).await
```

### Retry Logic

You can use automatic retry logic for operations that might fail due to transient errors:

```rust
db.transaction(|mut tx| async move {
    // Use retry logic for operations that might fail transiently
    tx.with_retry(3, || async {
        // This will be retried up to 3 times if it fails
        tx.execute("UPDATE accounts SET balance = balance - 100 WHERE id = 123 AND balance >= 100").await
    }).await?;
    
    Ok(())
}).await
```

## Error Handling

Errors are propagated through the transaction API and will cause automatic rollback if not handled. The `DatabaseError` enum provides detailed error information, including:

- Connection errors
- Transaction errors
- Query errors
- Migration errors
- Validation errors

The library also distinguishes between transient errors (which can be retried) and permanent errors.

## Implementing a New Provider

To implement a new database provider:

1. Create a new crate (e.g., `navius-db-mysql`)
2. Implement the interfaces defined in `navius-db`
3. Implement the `DatabaseProvider` trait

```rust
use navius_db::DatabaseProvider;

pub struct MyDatabaseProvider;

impl DatabaseProvider for MyDatabaseProvider {
    fn name(&self) -> &'static str {
        "my-database"
    }
    
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
```

For detailed guidelines on implementing a database provider, see the [Database Provider Implementation Guide](./DATABASE_PROVIDER_GUIDE.md).

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details. 