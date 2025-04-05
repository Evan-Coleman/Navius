use async_trait::async_trait;
use navius_core::error::{Error, Result};
use sqlx::postgres::PgPool; // Use the concrete PgPool type
use std::sync::Arc;

/// Trait for database repository operations.
#[async_trait]
pub trait DbRepository: Send + Sync {
    /// Get the database version string.
    async fn get_db_version(&self) -> Result<String>;

    /// Get the current database timestamp.
    async fn get_current_time(&self) -> Result<chrono::DateTime<chrono::Utc>>;
}

/// PostgreSQL implementation of the DbRepository.
#[derive(Debug, Clone)]
pub struct PostgresRepository {
    // Hold the concrete sqlx PgPool directly
    pool: Arc<PgPool>,
}

impl PostgresRepository {
    /// Create a new PostgresRepository.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DbRepository for PostgresRepository {
    async fn get_db_version(&self) -> Result<String> {
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(&*self.pool) // Deref Arc<PgPool> to &PgPool
            .await
            .map_err(|e| Error::database(format!("DB query failed: {}", e)))?;
        Ok(version.0)
    }

    async fn get_current_time(&self) -> Result<chrono::DateTime<chrono::Utc>> {
        let now: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(&*self.pool) // Deref Arc<PgPool> to &PgPool
            .await
            .map_err(|e| Error::database(format!("DB query failed: {}", e)))?;
        Ok(now.0)
    }
}

// Type alias still useful for dependency injection
pub type DynDbRepository = Arc<dyn DbRepository>;
