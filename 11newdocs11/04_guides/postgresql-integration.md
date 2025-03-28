---
title: "Navius PostgreSQL Integration Guide"
description: "A comprehensive guide to integrating PostgreSQL databases with Navius applications, including connection management, migrations, query optimization, and AWS RDS deployment"
category: guides
tags:
  - database
  - postgresql
  - aws-rds
  - migrations
  - orm
  - performance
  - connection-pooling
  - transactions
related:
  - ../reference/api/database-api.md
  - ../guides/deployment/aws-deployment.md
  - ../guides/features/caching.md
  - ../reference/configuration/environment-variables.md
last_updated: March 23, 2025
version: 1.0
---
# Navius PostgreSQL Integration Guide

This guide explains how to implement PostgreSQL database connections in your Navius application, leveraging the framework's powerful database abstraction layer.

## 🐘 Local Development Setup

For local development, Navius provides a Docker Compose configuration in `test/resources/docker/docker-compose.dev.yml`.

To start a PostgreSQL instance:

```bash
# From the project root:
cd test/resources/docker
docker-compose -f docker-compose.dev.yml up -d
```

This creates a PostgreSQL database with the following connection details:
- Host: localhost
- Port: 5432
- User: postgres
- Password: postgres
- Database: app

## ⚙️ Database Configuration

Ensure your `config/development.yaml` has the database section enabled:

```yaml
database:
  enabled: true
  url: "postgres://postgres:postgres@localhost:5432/app"
  max_connections: 10
  connect_timeout_seconds: 30
  idle_timeout_seconds: 300
```

## 🚀 Implementation Steps

Navius makes it easy to integrate with PostgreSQL for data persistence. Follow these steps:

### 1. Add SQLx Dependency

Navius uses SQLx as its preferred database library. Add it to your `Cargo.toml` if it's not already included:

```toml
[dependencies]
# ... existing dependencies
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "macros", "chrono", "uuid", "json"] }
```

### 2. Implement Database Connection

Update the `src/core/database/connection.rs` to use a real PostgreSQL connection:

```rust
use sqlx::{postgres::PgPoolOptions, PgPool as SqlxPgPool};

// Replace the mock implementation with a real one
#[derive(Debug, Clone)]
pub struct DatabaseConnection {
    config: DatabaseConfig,
    pool: SqlxPgPool,
}

impl DatabaseConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .idle_timeout(config.idle_timeout_seconds.map(Duration::from_secs))
            .connect(&config.url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
            
        Ok(Self {
            config: config.clone(),
            pool,
        })
    }
}

#[async_trait]
impl PgPool for DatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Database ping failed: {}", e)))?;
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx = self.pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;
            
        Ok(Box::new(SqlxTransaction { tx }))
    }
}

// Transaction implementation
struct SqlxTransaction {
    tx: sqlx::Transaction<'static, sqlx::Postgres>,
}

#[async_trait]
impl PgTransaction for SqlxTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        self.tx
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        self.tx
            .rollback()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
        Ok(())
    }
}
```

### 3. Update Database Initialization

Modify the `src/core/database/mod.rs` to use the new implementation:

```rust
pub async fn init_database(config: &DatabaseConfig) -> Result<Arc<Box<dyn PgPool>>, AppError> {
    if !config.enabled {
        tracing::info!("Database is disabled in configuration");
        return Err(AppError::DatabaseError("Database is disabled".into()));
    }

    tracing::info!("Initializing database connection to {}", config.url);

    let conn = DatabaseConnection::new(config)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to initialize database: {}", e)))?;

    // Return an Arc boxed as PgPool trait object
    Ok(Arc::new(Box::new(conn) as Box<dyn PgPool>))
}
```

### 4. Implement Repository With SQL

Update the repository implementations to use SQL queries instead of in-memory storage:

```rust
// Example for UserRepository
impl UserRepository {
    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let db_pool = self.db_pool();
        let tx = db_pool.begin().await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, full_name, is_active, 
                   role as "role: UserRole", created_at, updated_at
            FROM users 
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&*tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user: {}", e)))?;
        
        tx.commit().await?;
        Ok(user)
    }
}

#[async_trait]
impl Repository<User, Uuid> for UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let db_pool = self.db_pool();
        let tx = db_pool.begin().await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, username, email, full_name, is_active, 
                   role as "role: UserRole", created_at, updated_at
            FROM users 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to fetch user: {}", e)))?;
        
        tx.commit().await?;
        Ok(user)
    }

    async fn save(&self, entity: User) -> Result<User, AppError> {
        let db_pool = self.db_pool();
        let tx = db_pool.begin().await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, username, email, full_name, is_active, role, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE
            SET email = $3, full_name = $4, is_active = $5, role = $6, updated_at = $8
            RETURNING id, username, email, full_name, is_active, 
                     role as "role: UserRole", created_at, updated_at
            "#,
            entity.id, entity.username, entity.email, entity.full_name, entity.is_active, 
            entity.role as UserRole, entity.created_at, entity.updated_at
        )
        .fetch_one(&*tx)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to save user: {}", e)))?;
        
        tx.commit().await?;
        Ok(user)
    }

    // Implement other repository methods similarly
}
```

### 5. Create Migrations

Create a migrations folder with SQL migrations:

```sql
-- migrations/001_create_users_table.sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    role VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
```

### 6. Implement Migration Runner

Add a function to run migrations during server startup:

```rust
pub async fn run_migrations(pool: &SqlxPgPool) -> Result<(), AppError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;
        
    tracing::info!("Database migrations completed successfully");
    Ok(())
}
```

## Testing PostgreSQL Implementation

Add integration tests that use a real PostgreSQL database:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;
    
    async fn setup_test_db() -> SqlxPgPool {
        // Use a unique database name for each test run
        let db_name = format!("test_db_{}", Uuid::new_v4().simple());
        
        // Connect to PostgreSQL server
        let admin_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:postgres@localhost:5432/postgres")
            .await
            .expect("Failed to connect to PostgreSQL");
            
        // Create test database
        sqlx::query(&format!("CREATE DATABASE {}", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");
            
        // Connect to test database
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!("postgres://postgres:postgres@localhost:5432/{}", db_name))
            .await
            .expect("Failed to connect to test database");
            
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
            
        pool
    }
    
    #[tokio::test]
    async fn test_user_repository() {
        let pool = setup_test_db().await;
        
        // Create repository
        let repo = UserRepository::new(pool);
        
        // Create user
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
        );
        
        // Save user
        let saved_user = repo.save(user).await.expect("Failed to save user");
        
        // Retrieve user by ID
        let retrieved_user = repo.find_by_id(saved_user.id).await.expect("Failed to find user");
        assert!(retrieved_user.is_some());
        let retrieved_user = retrieved_user.unwrap();
        assert_eq!(retrieved_user.username, "testuser");
        
        // Retrieve user by username
        let by_username = repo.find_by_username("testuser").await.expect("Failed to find user");
        assert!(by_username.is_some());
        assert_eq!(by_username.unwrap().id, saved_user.id);
        
        // Update user
        let mut updated_user = retrieved_user;
        updated_user.email = "updated@example.com".to_string();
        let saved_updated = repo.save(updated_user).await.expect("Failed to update user");
        assert_eq!(saved_updated.email, "updated@example.com");
        
        // Delete user
        let deleted = repo.delete(saved_user.id).await.expect("Failed to delete user");
        assert!(deleted);
        
        // Verify deletion
        let should_be_none = repo.find_by_id(saved_user.id).await.expect("Query failed");
        assert!(should_be_none.is_none());
    }
}
```

## Production Considerations

For production environments:

1. Use AWS RDS for PostgreSQL
2. Configure connection pooling appropriately
3. Use IAM authentication for database access
4. Enable encryption in transit and at rest
5. Set up automated backups
6. Configure monitoring and alerts
7. Use read replicas for read-heavy workloads

## Best Practices

- Use prepared statements for all database queries
- Implement proper error handling and retries
- Keep transactions as short as possible
- Use connection pooling to manage database connections
- Implement database migrations for schema changes
- Use database indexes for performance
- Write integration tests against a real database
- Monitor query performance and optimize slow queries 

## Related Documents
- [Installation Guide](../01_getting_started/installation.md) - How to install the application
- [Development Workflow](development/development-workflow.md) - Development best practices

