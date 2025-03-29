---
title: "Database Migrations Guide"
description: "Comprehensive guide for managing database schema migrations in Navius applications, ensuring reliable and safe schema changes"
category: "Guides"
tags: ["database", "migrations", "schema", "postgresql", "versioning", "deployment"]
last_updated: "April 5, 2025"
version: "1.0"
---

# Database Migrations Guide

## Overview

This guide provides comprehensive strategies for managing database schema migrations in Navius applications. Database migrations are essential for evolving your application's data model while ensuring data integrity and minimizing downtime.

## Migration Fundamentals

### What Are Migrations?

Database migrations are versioned, incremental changes to your database schema that:

- Create, modify, or delete tables and columns
- Add or remove constraints, indexes, and foreign keys
- Update data to conform to new schema requirements
- Can be applied and rolled back reliably

### Key Principles

1. **Versioned** - Each migration has a unique version number or identifier
2. **Directional** - Migrations can be applied (up) or rolled back (down)
3. **Incremental** - Small, focused changes rather than large schema overhauls
4. **Repeatable** - Same migration produces the same result when applied multiple times
5. **Transactional** - A migration should fully succeed or fully fail

## Migration Tools for Navius

Navius supports multiple migration tools:

### 1. SQLx Migrations

```rust
// Example: Creating a new migration with SQLx
sqlx migrate add -r create_users_table

// Example: Running all pending migrations
sqlx migrate run
```

### 2. Refinery

```rust
// Example: Defining migrations in code with Refinery
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    embedded::migrations::runner()
        .run_async(pool)
        .await?;
    Ok(())
}
```

### 3. Custom Migration Framework

Navius also provides a custom migration framework with additional features:

```rust
use navius::database::migrations::{MigrationManager, Migration};

async fn run_migrations(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let manager = MigrationManager::new(pool.clone());
    manager.run_migrations().await?;
    Ok(())
}
```

## Creating Migrations

### Migration Structure

A well-structured migration includes:

1. **Version** - Usually a timestamp (e.g., `20250405120000`)
2. **Name** - Descriptive name (e.g., `create_users_table`)
3. **Up migration** - SQL to apply the change
4. **Down migration** - SQL to roll back the change (when applicable)

Example SQLx migration:

```sql
-- 20250405120000_create_users_table.up.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users (email);
CREATE INDEX idx_users_status ON users (status);

-- 20250405120000_create_users_table.down.sql
DROP TABLE users;
```

### Best Practices for Migration Content

1. **Add New Tables/Columns First**
   - Create new structures before updating existing ones
   - Allows the application to gracefully handle both old and new schemas

2. **Avoid Direct RENAME Operations**
   - Create new column/table, copy data, remove old column/table
   - Helps avoid issues with running application code during deployment

3. **Non-Destructive Migrations**
   - Use `ALTER TABLE ADD COLUMN` instead of dropping and recreating tables
   - Add with nullable columns or reasonable defaults

4. **Handle Large Tables Carefully**
   - Use batched operations for large data migrations
   - Consider maintenance windows for significant schema changes

## Running Migrations

### Development Environment

In development, run migrations manually:

```bash
# Using SQLx CLI
sqlx migrate run

# Using Navius CLI
navius db migrate
```

### Production Deployment

For production, migrations are typically run as part of the deployment process:

```rust
// Initialize database during application startup
async fn initialize_database(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running database migrations");
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    log::info!("Migrations completed successfully");
    Ok(())
}
```

### Deployment Strategies

#### 1. Zero Downtime Migrations

For minimal disruption, follow this process:

1. Deploy application that handles both old and new schema
2. Run migrations
3. Deploy application that uses only new schema

#### 2. Maintenance Window Migrations

For complex changes:

1. Enable maintenance mode
2. Run migrations
3. Deploy new application version
4. Disable maintenance mode

## Testing Migrations

Always test migrations thoroughly before deploying to production:

### 1. Automated Tests

```rust
#[sqlx::test]
async fn test_migrations() -> Result<(), Box<dyn std::error::Error>> {
    // Set up a clean test database
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:password@localhost/test_db")
        .await?;
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    // Verify schema
    let result = sqlx::query!("SELECT COUNT(*) as count FROM pg_tables WHERE tablename = 'users'")
        .fetch_one(&pool)
        .await?;
    
    assert_eq!(result.count, Some(1));
    Ok(())
}
```

### 2. Staging Environment

1. Clone production database to staging (with sanitized data)
2. Apply migrations in staging
3. Run application and perform manual testing
4. Verify performance with realistic data volumes

## Advanced Migration Techniques

### Data Migrations

For moving or transforming data:

```sql
-- Example: Data migration to populate a new column
-- 20250405123000_populate_username.up.sql
UPDATE users
SET username = LOWER(REGEXP_REPLACE(email, '@.*$', ''))
WHERE username IS NULL;

-- 20250405123000_populate_username.down.sql
-- No rollback needed, or could set username to NULL
```

### Schema Migrations with Foreign Keys

Handling foreign key constraints:

```sql
-- Adding a new table with foreign keys
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_user_id
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE
);
```

### Multiple Environment Support

Configure migrations for different environments:

```rust
async fn run_migrations(pool: &PgPool, env: &str) -> Result<(), Box<dyn std::error::Error>> {
    let migration_path = match env {
        "test" => "./migrations/test",
        "development" => "./migrations/development",
        _ => "./migrations",
    };
    
    sqlx::migrate!(migration_path)
        .run(pool)
        .await?;
    
    Ok(())
}
```

## Common Challenges and Solutions

### 1. Long-Running Migrations

For tables with millions of rows:

```sql
-- Instead of a single ALTER TABLE
-- Do this in batches:
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT id FROM users WHERE processed = FALSE ORDER BY id LIMIT 10000
    LOOP
        -- Process each batch
        UPDATE users SET data_json = jsonb_set(data_json, '{status}', '"active"')
        WHERE id = r.id;
        
        -- Commit progress periodically
        COMMIT;
    END LOOP;
END $$;
```

### 2. Schema Drift Detection

Detecting unexpected schema changes:

```rust
async fn check_schema_consistency(pool: &PgPool) -> Result<bool, Box<dyn std::error::Error>> {
    let result = sqlx::query!("
        SELECT COUNT(*) as count 
        FROM information_schema.columns 
        WHERE table_name = 'users' 
        AND column_name NOT IN ('id', 'email', 'name', 'password_hash', 'status', 'created_at', 'updated_at')
    ")
    .fetch_one(pool)
    .await?;
    
    Ok(result.count == Some(0))
}
```

### 3. Version Control Conflicts

When multiple developers create migrations:

1. Use timestamped migration names to avoid numbering conflicts
2. Rebase migrations in development before merging to main
3. Never change a migration that has been applied to any environment

## Migration Monitoring and Management

### Migration Status

Check migration status:

```bash
# Using SQLx CLI
sqlx migrate info

# Using Navius CLI
navius db status
```

### Version Tracking

Navius tracks all applied migrations in a dedicated table:

```sql
SELECT * FROM _sqlx_migrations ORDER BY version;
```

## Case Study: Complex Schema Evolution

### Background
Our user service needed to split the monolithic `users` table into specialized tables for better performance and organization.

### Approach
We implemented the migration across multiple releases:

#### Release 1: Preparation
1. Created new tables (`user_profiles`, `user_settings`)
2. Added triggers to sync data between old and new tables
3. Deployed application that could read from both schemas

#### Release 2: Data Migration
1. Migrated existing data from `users` to new tables
2. Verified data consistency
3. Updated application to use both schemas

#### Release 3: Schema Finalization
1. Removed duplicate columns from `users` table
2. Updated triggers for remaining data sync
3. Deployed application using only new schema

#### Release 4: Cleanup
1. Removed triggers and temporary structures
2. Optimized new tables with appropriate indexes

### Results
- Zero downtime during migration
- Improved query performance by 60%
- Better data organization and maintainability

## Related Resources

- [Database Optimization Guide](./database-optimization.md)
- [PostgreSQL Integration Guide](./postgresql-integration.md)
- [Performance Tuning Guide](./performance-tuning.md)
- [Deployment Guide](./deployment.md)
- [CI/CD Integration Guide](./deployment/continuous-integration.md) 