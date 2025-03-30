# Navius DB PostgreSQL

PostgreSQL implementation of the Navius DB interfaces. This crate provides a complete implementation of the database abstraction layer for PostgreSQL databases using SQLx.

## Features

- Complete implementation of the `navius-db` interface traits
- PostgreSQL-specific optimizations and features
- SQLx integration with connection pooling
- Transaction management
- Migration support
- Repository pattern implementation

## Installation

Add navius-db-postgres to your dependencies:

```toml
[dependencies]
navius-db = "0.1.0"
navius-db-postgres = "0.1.0"
```

## Configuration

Configure a PostgreSQL database connection:

```rust
use navius_db_postgres::PgDatabaseConfig;

fn create_config() -> PgDatabaseConfig {
    let mut config = PgDatabaseConfig::new("postgres://user:password@localhost:5432/mydb".to_string());
    
    // Configure PostgreSQL-specific options
    config.ssl_mode = Some("prefer".to_string());
    config.application_name = Some("my-application".to_string());
    
    // Configure common options
    config.core.max_connections = 20;
    config.core.min_connections = 5;
    config.core.run_migrations = true;
    config.core.migrations_path = "migrations".to_string();
    
    config
}
```

## Usage

### Creating a Database Provider

```rust
use navius_db::{DatabaseProvider, DatabaseConnectionManager};
use navius_db_postgres::{PgProvider, PgDatabaseConfig};

async fn create_provider() -> PgProvider {
    let config = PgDatabaseConfig::new("postgres://user:password@localhost:5432/mydb".to_string());
    PgProvider::new(config).await.expect("Failed to create PostgreSQL provider")
}
```

### Working with Repositories

```rust
use navius_db::{Entity, Repository};
use navius_db_postgres::{PgProvider, PgRepository};
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

async fn repository_example(provider: &PgProvider) -> Result<(), Box<dyn std::error::Error>> {
    // Create a repository for users
    let repo = PgRepository::<User>::new(provider);
    
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

### Using Transactions

```rust
use navius_db::DatabaseConnectionManager;
use navius_db_postgres::PgProvider;

async fn transaction_example(provider: &PgProvider) -> Result<(), Box<dyn std::error::Error>> {
    provider.transaction(|mut tx| async move {
        // Execute multiple operations in a transaction
        tx.execute_raw("INSERT INTO logs (message) VALUES ($1)", &["Transaction started"]).await?;
        
        // Do more operations...
        
        tx.execute_raw("INSERT INTO logs (message) VALUES ($1)", &["Transaction completed"]).await?;
        
        // Transaction commits when the closure completes successfully
        Ok(())
    }).await?;
    
    Ok(())
}
```

### Running Migrations

```rust
use navius_db_postgres::{PgProvider, PgDatabaseConfig};

async fn run_migrations(provider: &PgProvider) -> Result<(), Box<dyn std::error::Error>> {
    // Run migrations from the specified path
    provider.run_migrations("./migrations").await?;
    println!("Migrations completed successfully");
    
    Ok(())
}
```

## Implementing Custom Repositories

You can extend the base `PgRepository` to add custom query methods:

```rust
use navius_db::{Repository, DatabaseResult};
use navius_db_postgres::{PgProvider, PgRepository};

struct UserRepository<'a> {
    base: PgRepository<'a, User>,
}

impl<'a> UserRepository<'a> {
    pub fn new(provider: &'a PgProvider) -> Self {
        Self {
            base: PgRepository::new(provider),
        }
    }
    
    // Implement all base Repository methods by delegating to base
    
    // Add custom methods
    pub async fn find_by_username(&self, username: &str) -> DatabaseResult<Option<User>> {
        // Implementation using the provider's query capabilities
        let query = self.base.provider().query_builder()
            .select_from(User::table_name())
            .where_equal("username", username)
            .build();
            
        self.base.provider().query_one::<User>(query).await
    }
}
```

## Platform Support

This crate supports PostgreSQL 11+ and requires SQLx with the postgres feature.

## Current Status

The navius-db-postgres crate is currently in development (25% complete). The following features are in progress:

- Core interface implementations (50% complete)
- SQLx integration (50% complete)
- Repository implementation (0% complete)
- Migration support (0% complete)
- Tests (5% complete)

See the [implementation progress](../../workspace_migration/roadmap/sub-process/implementation-progress.md) for more details.

## License

This project is licensed under the Apache License 2.0 - see the LICENSE file for details. 