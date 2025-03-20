# Database Integration Roadmap

## Overview
A straightforward approach to database integration using PostgreSQL with a focus on core functionality and type safety.

## Current State
Basic database functionality is needed with proper connection management and type-safe queries.

## Target State
A simple, reliable database layer with:
- PostgreSQL connection with proper pooling
- Type-safe SQL queries with SQLx
- Simple migrations
- Minimal but effective error handling

## Implementation Progress Tracking

### Phase 1: Core Database Setup
1. **Basic Connection**
   - [ ] Set up PostgreSQL connection configuration
   - [ ] Implement simple service account access
   - [ ] Create connection pooling with deadpool-postgres
   
   *Updated at: Not started*

2. **Type-Safe Queries**
   - [ ] Integrate SQLx for compile-time SQL validation
   - [ ] Create basic repository pattern for database access
   - [ ] Implement simple error handling
   
   *Updated at: Not started*

3. **Transaction Support**
   - [ ] Create basic transaction wrapper
   - [ ] Implement automatic rollback on error
   - [ ] Add simple logging for database operations
   
   *Updated at: Not started*

### Phase 2: Migration Management
1. **Migration System**
   - [ ] Set up SQLx migrations
   - [ ] Create basic migration command tooling
   - [ ] Implement version tracking
   
   *Updated at: Not started*

2. **Schema Management**
   - [ ] Create initial schema definition
   - [ ] Implement type-safe model mapping
   - [ ] Add database documentation generation
   
   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 0% complete
- **Last Updated**: March 20, 2024
- **Next Milestone**: Basic PostgreSQL Connection

## Success Criteria
- Database connections work reliably
- Queries are type-safe with SQLx
- Simple migrations can be run manually and on startup
- Errors are handled gracefully

## Implementation Notes
This approach focuses on simplicity and reliability. Rather than implementing complex features upfront, we'll start with a minimal viable implementation and add functionality as needed. 

AWS-specific features (like IAM authentication and RDS configuration) and security features (like Entra identity integration) are handled in the AWS Integration roadmap.

### Example Implementation

```rust
// Very simple database service
trait DbService: Send + Sync {
    async fn query_one<T>(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<T, DbError>
    where
        T: for<'a> FromRow<'a> + Send + Unpin;
        
    async fn execute(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, DbError>;
    
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> BoxFuture<'_, Result<R, DbError>> + Send,
        R: Send;
}

// Simple database implementation
struct PostgresDb {
    pool: Pool<PostgresConnectionManager<MakeTlsConnector>>,
}

impl PostgresDb {
    async fn new(config: &str) -> Result<Self, DbError> {
        let manager = PostgresConnectionManager::new_from_stringlike(
            config,
            MakeTlsConnector::new(TlsConnector::new()?),
        )?;
        
        let pool = Pool::builder()
            .max_size(15) // Reasonable default
            .build(manager)
            .await?;
        
        Ok(Self { pool })
    }
}

impl DbService for PostgresDb {
    async fn query_one<T>(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<T, DbError>
    where
        T: for<'a> FromRow<'a> + Send + Unpin,
    {
        let client = self.pool.get().await?;
        let stmt = client.prepare(query).await?;
        let row = client.query_one(&stmt, params).await?;
        Ok(T::from_row(&row)?)
    }
    
    async fn execute(&self, query: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, DbError> {
        let client = self.pool.get().await?;
        let stmt = client.prepare(query).await?;
        let result = client.execute(&stmt, params).await?;
        Ok(result)
    }
    
    async fn transaction<F, R>(&self, f: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut Transaction<'_, Postgres>) -> BoxFuture<'_, Result<R, DbError>> + Send,
        R: Send,
    {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;
        
        let result = f(&mut tx).await;
        
        match result {
            Ok(value) => {
                tx.commit().await?;
                Ok(value)
            }
            Err(e) => {
                // Auto-rollback happens when tx is dropped
                Err(e)
            }
        }
    }
}

// Simple API handler example
async fn create_user(
    State(db): State<Arc<dyn DbService>>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // Execute database operation in transaction
    let result = db.transaction(|tx| Box::pin(async move {
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id",
            payload.name,
            payload.email
        )
        .fetch_one(tx)
        .await?;
        
        Ok(user_id)
    })).await;
    
    match result {
        Ok(user_id) => (StatusCode::CREATED, Json(json!({ "id": user_id }))).into_response(),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
```

## References
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [Postgres Rust Driver](https://docs.rs/postgres/latest/postgres/)
- [Tokio Postgres](https://docs.rs/tokio-postgres/latest/tokio_postgres/)
- [Deadpool Postgres](https://docs.rs/deadpool-postgres/latest/deadpool_postgres/) 