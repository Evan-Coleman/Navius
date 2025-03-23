# Application Repositories

This directory contains user-defined repositories that extend the core data access layer of the Navius application. Use this module to define custom repositories for your application-specific models.

## Usage

To use repositories in your application code:

```rust
use crate::app::repository;
use crate::core::database::get_connection_pool;

async fn example() -> Result<()> {
    let pool = get_connection_pool();
    
    // Use an existing repository
    let users = repository::user::UserRepository::new(pool.clone());
    let user = users.find_by_id("user-1").await?;
    
    // Use your custom repository
    let profiles = repository::profile_repository::ProfileRepository::new(pool);
    let profile = profiles.find_by_user_id(user.id).await?;
    
    Ok(())
}
```

## Creating a Custom Repository

1. Create a new file in this directory for your repository
2. Define your repository struct and methods
3. Use the core repository patterns and error handling

```rust
// src/app/repository/profile_repository.rs
use crate::core::{database::PgPool, error::Result};
use crate::models::Profile;

pub struct ProfileRepository {
    pool: PgPool,
}

impl ProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn find_by_user_id(&self, user_id: &str) -> Result<Profile> {
        let profile = sqlx::query_as!(
            Profile,
            r#"
            SELECT * FROM profiles WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(profile)
    }
    
    // Add more methods as needed
}
```

## Repository Pattern Best Practices

1. Keep repositories focused on a single entity
2. Use meaningful method names (find_by_*, create_*, update_*, delete_*)
3. Return `Result<T>` for all operations
4. Use transactions for operations that modify multiple tables
5. Write unit tests for all repository methods
6. Document complex queries
7. Add proper error handling

## Core Repositories

The core repositories are provided by `crate::core::repository` and include:

- Base repository traits and implementations
- Common query builders and utilities
- Standard CRUD operations

Do not modify the core repositories directly. Instead, use this directory to extend and customize repositories for your specific application needs. 