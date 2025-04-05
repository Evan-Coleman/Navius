use async_trait::async_trait;
use navius_core::error::{Error, Result};
use navius_db::pool::Pool;
use sqlx::postgres::PgPool; // Need the specific pool type
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
    pool: Arc<Pool>, // Use Arc<Pool> for sharing
}

impl PostgresRepository {
    /// Create a new PostgresRepository.
    pub fn new(pool: Arc<Pool>) -> Self {
        Self { pool }
    }

    /// Helper to get the underlying PgPool.
    fn pg_pool(&self) -> Result<&PgPool> {
        self.pool.inner::<PgPool>()
    }
}

#[async_trait]
impl DbRepository for PostgresRepository {
    async fn get_db_version(&self) -> Result<String> {
        let version: (String,) = sqlx::query_as("SELECT version()")
            .fetch_one(self.pg_pool()?)
            .await
            .map_err(|e| Error::database(format!("DB query failed: {}", e)))?;
        Ok(version.0)
    }

    async fn get_current_time(&self) -> Result<chrono::DateTime<chrono::Utc>> {
        let now: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(self.pg_pool()?)
            .await
            .map_err(|e| Error::database(format!("DB query failed: {}", e)))?;
        Ok(now.0)
    }
}

// Optional: Define an alias for the concrete repository type if needed elsewhere
pub type DynDbRepository = Arc<dyn DbRepository>;
