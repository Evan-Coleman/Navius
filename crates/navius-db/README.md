# navius-db

Database connectivity and operations for the Navius framework.

## Features

- Connection pooling with SQLx
- Transaction management
- Repository pattern for database operations
- Query building with type safety
- Error handling with clear error types
- Entity mapping to and from database records
- Migration support

## Usage

Add to your Cargo.toml:

```toml
[dependencies]
navius-db = { version = "0.1.0", features = ["postgres"] }
```

### Basic Example

```rust
use navius_db::{
    config::DatabaseConfig,
    connection::DatabaseConnectionManager,
    pool::PgPool,
    repository::{Entity, Repository, SqlRepository},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    name: String,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database configuration
    let config = DatabaseConfig::new("postgres://user:pass@localhost:5432/mydb".to_string());
    
    // Create database connection pool
    let pool = PgPool::new(&config).await?;
    
    // Create connection manager
    let db = DatabaseConnectionManager::new(pool);
    
    // Create repository
    let repository = SqlRepository::<User>::new(db);
    
    // Create a user
    let mut user = User {
        id: Uuid::nil(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };
    
    // Save the user
    user = repository.save(&user).await?;
    
    // Find the user
    let found_user = repository.find_by_id(user.id).await?;
    
    // Delete the user
    repository.delete(user.id).await?;
    
    Ok(())
}
```

## Features

- `postgres` - Enable PostgreSQL support (enabled by default)

## License

Apache-2.0 