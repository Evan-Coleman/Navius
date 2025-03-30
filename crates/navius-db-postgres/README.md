# Navius DB PostgreSQL

PostgreSQL implementation for the Navius database framework. This crate provides PostgreSQL connectivity using SQLx and implements the interfaces defined in the core navius-db crate.

## Features

- Complete PostgreSQL implementation for navius-db interfaces
- Connection pooling with configurable parameters
- Transaction management
- Repository pattern implementation
- Query building
- Typed result handling
- Migration support

## Usage

Add both navius-db and navius-db-postgres to your dependencies:

```toml
[dependencies]
navius-db = "0.1.0"
navius-db-postgres = "0.1.0"
```

### Basic Setup

```rust
use navius_db::{DatabaseConfig, DatabaseResult};
use navius_db_postgres::{PgPool, PgConnectionManager, PgPoolOptions};

async fn setup_database() -> DatabaseResult<PgConnectionManager> {
    // Create database configuration
    let config = DatabaseConfig::new("postgres://user:password@localhost:5432/mydb".to_string());
    
    // Create PostgreSQL pool
    let pool = PgPool::new(&config).await?;
    
    // Create connection manager
    let manager = PgConnectionManager::new(pool);
    
    Ok(manager)
}
```

### Using Transactions

```rust
use navius_db::DatabaseResult;
use navius_db_postgres::PgConnectionManager;

async fn transfer_funds(
    db: &PgConnectionManager,
    from_account: &str,
    to_account: &str,
    amount: f64,
) -> DatabaseResult<()> {
    // Use a transaction to ensure both operations succeed or fail together
    db.transaction(|mut tx| async move {
        // Deduct from source account
        let from_query = "UPDATE accounts SET balance = balance - $1 WHERE account_id = $2 AND balance >= $1";
        let rows = tx.execute_with(from_query, &[&amount, &from_account]).await?;
        
        if rows == 0 {
            // No rows updated, likely insufficient funds
            return Err(navius_db::DatabaseError::ValidationError("Insufficient funds".to_string()));
        }
        
        // Add to destination account
        let to_query = "UPDATE accounts SET balance = balance + $1 WHERE account_id = $2";
        tx.execute_with(to_query, &[&amount, &to_account]).await?;
        
        // Transaction automatically commits on success
        Ok(())
    }).await
}
```

### Using Repository Pattern

```rust
use navius_db::{Entity, Repository, DatabaseResult};
use navius_db_postgres::{PgConnectionManager, PgRepository};
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

async fn user_example(db: &PgConnectionManager) -> DatabaseResult<()> {
    // Create repository for User entity
    let repo = PgRepository::<User>::new(db.clone());
    
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
    
    // Delete user
    repo.delete(user.id()).await?;
    println!("User deleted");
    
    Ok(())
}
```

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details. 