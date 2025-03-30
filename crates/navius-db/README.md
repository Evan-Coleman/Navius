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

Use transactions to ensure data consistency:

```rust
use navius_db::{DatabaseConnectionManager, DatabaseResult};

async fn with_transaction(db: &dyn DatabaseConnectionManager) -> DatabaseResult<()> {
    db.transaction(|mut tx| async move {
        // Perform multiple operations in a transaction
        tx.execute("INSERT INTO logs (message) VALUES ($1)", &["Transaction started"]).await?;
        
        // Do more operations...
        
        tx.execute("INSERT INTO logs (message) VALUES ($1)", &["Transaction completed"]).await?;
        
        // Transaction commits when the closure completes successfully
        Ok(())
    }).await
}
```

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

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details. 