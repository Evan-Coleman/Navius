---
title: "Database Optimization Guide"
description: "Comprehensive guide for optimizing PostgreSQL database performance in Navius applications, including indexing, query optimization, and schema design"
category: "Guides"
tags: ["database", "postgresql", "optimization", "performance", "indexing", "queries", "schema"]
last_updated: "April 5, 2025"
version: "1.0"
---

# Database Optimization Guide

## Overview

This guide provides comprehensive strategies for optimizing PostgreSQL database performance in Navius applications. Database performance is critical for application responsiveness and scalability, as database operations often represent the most significant bottleneck in web applications.

## Database Design Principles

### Schema Design

- **Normalize with purpose** - Follow normalization principles but prioritize query performance
- **Choose appropriate data types** - Use the most efficient data types for each column
- **Limit column width** - Use varchar with appropriate length limits instead of unlimited text fields
- **Consider table partitioning** - For very large tables (millions of rows)

### Index Design

- **Primary keys** - Always define explicit primary keys
- **Foreign keys** - Index all foreign key columns
- **Compound indexes** - Create for commonly queried column combinations
- **Cover indexes** - Include additional columns to create covering indexes
- **Partial indexes** - Use for filtered queries on large tables

```sql
-- Example: Compound index for a commonly used query pattern
CREATE INDEX idx_users_email_status ON users (email, status);

-- Example: Covering index to avoid table lookups
CREATE INDEX idx_posts_author_created_title ON posts (author_id, created_at) INCLUDE (title);

-- Example: Partial index for active users
CREATE INDEX idx_active_users ON users (email, last_login) WHERE status = 'active';
```

## Query Optimization

### Query Analysis

Use `EXPLAIN ANALYZE` to understand query execution plans:

```sql
EXPLAIN ANALYZE SELECT * FROM users 
WHERE email LIKE 'user%' AND status = 'active'
ORDER BY created_at DESC LIMIT 10;
```

Look for these issues in query plans:
- Sequential scans on large tables
- High cost operations
- Unused indexes
- Poor join performance

### Common Optimizations

#### 1. Avoid SELECT *

```sql
-- Instead of this
SELECT * FROM users WHERE id = 1;

-- Do this
SELECT id, email, name, created_at FROM users WHERE id = 1;
```

#### 2. Use Parameterized Queries

```rust
// Instead of string interpolation
let users = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE email = $1 AND status = $2")
    .bind(email)
    .bind(status)
    .fetch_all(&pool)
    .await?;
```

#### 3. Batch Operations

```rust
// Instead of multiple single inserts
let mut transaction = pool.begin().await?;

for user in users {
    sqlx::query("INSERT INTO user_logs (user_id, action, timestamp) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind("login")
        .bind(Utc::now())
        .execute(&mut transaction)
        .await?;
}

transaction.commit().await?;
```

#### 4. Use Appropriate WHERE Clauses

```sql
-- Instead of functions on indexed columns
SELECT * FROM users WHERE LOWER(email) = 'user@example.com';

-- Do this
SELECT * FROM users WHERE email = 'user@example.com';
```

## Connection Management

### Connection Pooling

Configure your connection pool appropriately:

```rust
// Optimal connection pool configuration for Navius applications
let pool = PgPoolOptions::new()
    .max_connections(num_cpus::get() * 4) // 4x CPU cores as a starting point
    .min_connections(num_cpus::get()) // Maintain at least one connection per CPU
    .max_lifetime(std::time::Duration::from_secs(30 * 60)) // 30 minutes
    .idle_timeout(std::time::Duration::from_secs(5 * 60)) // 5 minutes
    .connect(&database_url)
    .await?;
```

Guidelines for sizing:
- Measure maximum concurrent database operations during peak load
- Consider PostgreSQL's `max_connections` setting (usually 100-300)
- Monitor connection usage over time

### Transaction Management

- Keep transactions as short as possible
- Don't perform I/O or network operations within transactions
- Use appropriate isolation levels
- Consider using read-only transactions for queries

```rust
// Example: Read-only transaction
let mut tx = pool.begin_read_only().await?;
let users = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE status = $1")
    .bind("active")
    .fetch_all(&mut tx)
    .await?;
tx.commit().await?;
```

## Advanced Optimization Techniques

### PostgreSQL Configuration

Key PostgreSQL settings to tune:

```
# Memory settings
shared_buffers = 25% of system RAM (up to 8GB)
work_mem = 32-64MB
maintenance_work_mem = 256MB

# Checkpoint settings
checkpoint_timeout = 15min
checkpoint_completion_target = 0.9

# Planner settings
random_page_cost = 1.1 (for SSD storage)
effective_cache_size = 75% of system RAM
```

### Materialized Views

For expensive reports or analytics queries:

```sql
CREATE MATERIALIZED VIEW user_stats AS
SELECT 
    date_trunc('day', created_at) as day,
    count(*) as new_users,
    avg(extract(epoch from now() - last_login)) as avg_days_since_login
FROM users
GROUP BY date_trunc('day', created_at);

-- Refresh the view:
REFRESH MATERIALIZED VIEW user_stats;
```

### Database Monitoring

Monitor these metrics:
- Query execution times
- Index usage statistics
- Cache hit ratios
- Lock contention
- Deadlocks

Tools to use:
- [pg_stat_statements](https://www.postgresql.org/docs/current/pgstatstatements.html)
- [pgmetrics](https://pgmetrics.io/)
- [pgHero](https://github.com/ankane/pghero)

## Implementing Repository Pattern in Navius

The Repository pattern helps maintain clean database access and makes queries easier to optimize:

```rust
// User repository implementation
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT id, name, email, status FROM users WHERE email = $1 LIMIT 1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
    }
    
    pub async fn find_active_users(&self, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT id, name, email, status FROM users 
             WHERE status = 'active' 
             ORDER BY last_login DESC 
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
    }
    
    // Additional methods...
}
```

## Performance Testing Database Queries

### Benchmarking Strategies

1. **Isolated Query Testing** - Test queries independently from application
2. **Mock Production Data** - Use production-sized datasets
3. **Concurrent Load Testing** - Test under simultaneous connections
4. **EXPLAIN ANALYZE** - Measure execution plan costs
5. **Cache Warmup/Cooldown** - Test with both hot and cold cache scenarios

### Testing with Criterion

```rust
fn benchmark_user_query(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(establish_connection());
    let repo = UserRepository::new(pool.clone());
    
    c.bench_function("find_active_users", |b| {
        b.iter(|| {
            rt.block_on(repo.find_active_users(100, 0))
        })
    });
}
```

## Common Database Performance Issues

### N+1 Query Problem

```rust
// BAD: N+1 query problem
let users = repo.find_active_users(100, 0).await?;
for user in &users {
    let posts = repo.find_posts_by_user_id(user.id).await?;
    // Process posts...
}

// GOOD: Single query with join
let user_with_posts = repo.find_active_users_with_posts(100, 0).await?;
```

### Missing Indexes

Signs of missing indexes:
- Sequential scans on large tables
- Slow filtering operations
- Slow ORDER BY or GROUP BY clauses

### Oversized Queries

- Fetching unnecessary columns
- Not using LIMIT with large result sets
- Not using pagination
- Using subqueries when joins would be more efficient

## Database Migration Strategies

When migrating or updating schemas:

1. **Create indexes concurrently**
   ```sql
   CREATE INDEX CONCURRENTLY idx_users_status ON users (status);
   ```

2. **Perform updates in batches**
   ```rust
   // Update in batches of 1000
   for batch in user_ids.chunks(1000) {
       sqlx::query("UPDATE users SET status = $1 WHERE id = ANY($2)")
           .bind("inactive")
           .bind(batch)
           .execute(&pool)
           .await?;
   }
   ```

3. **Use temporary tables for complex migrations**
   ```sql
   CREATE TEMPORARY TABLE temp_users AS SELECT * FROM users WHERE false;
   INSERT INTO temp_users SELECT * FROM users WHERE <condition>;
   -- Perform operations on temp_users
   -- Finally update or insert back to users
   ```

## Case Study: Optimizing a High-Traffic User Service

### Initial State
- Average query time: 150ms
- Database CPU: 85% utilization
- Cache hit ratio: 45%
- Frequent timeouts during peak traffic

### Optimization Steps
1. Added compound indexes on commonly queried fields
2. Implemented result caching for frequent queries
3. Optimized schema removing unused columns
4. Implemented connection pooling with optimal settings
5. Added database replicas for read operations

### Results
- Average query time: 15ms (10x improvement)
- Database CPU: 40% utilization
- Cache hit ratio: 80%
- Zero timeouts during peak traffic

## Related Resources

- [PostgreSQL Integration Guide](./postgresql-integration.md)
- [Performance Tuning Guide](./performance-tuning.md)
- [Caching Strategies Guide](./caching-strategies.md)
- [Database Migration Guide](./database-migration.md)
- [Two-Tier Cache Example](../02_examples/two-tier-cache-example.md) 