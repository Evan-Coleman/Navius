use crate::error::{DatabaseError, DatabaseResult};
use crate::pool::{DatabaseConnection, DatabasePool};
use crate::transaction::Transaction;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Wrapper for a database connection pool
#[derive(Debug, Clone)]
pub struct DatabaseConnectionManager {
    pool: Arc<dyn DatabasePool>,
}

impl DatabaseConnectionManager {
    /// Create a new database connection manager
    pub fn new<P: DatabasePool>(pool: P) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Get a connection from the pool
    #[instrument(skip(self))]
    pub async fn get_connection(&self) -> DatabaseResult<DatabaseConnectionHandle> {
        debug!("Acquiring database connection from pool");
        let conn = self.pool.acquire().await?;
        Ok(DatabaseConnectionHandle { conn })
    }

    /// Start a transaction
    #[instrument(skip(self, f))]
    pub async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: for<'c> FnOnce(Transaction<'c>) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
    {
        debug!("Starting database transaction");
        let mut conn = self.get_connection().await?;
        let tx = conn.begin().await?;
        let result = f(tx);

        match result {
            Ok(value) => {
                debug!("Committing database transaction");
                conn.commit().await?;
                Ok(value)
            }
            Err(err) => {
                debug!("Rolling back database transaction");
                conn.rollback().await?;
                Err(err)
            }
        }
    }

    /// Check database health
    pub async fn check_health(&self) -> DatabaseResult<()> {
        self.pool.check_health().await
    }
}

/// Handle for a database connection
#[derive(Debug)]
pub struct DatabaseConnectionHandle {
    conn: Box<dyn DatabaseConnection>,
}

impl DatabaseConnectionHandle {
    /// Begin a transaction
    pub async fn begin(&mut self) -> DatabaseResult<Transaction<'_>> {
        let tx = self.conn.begin().await?;
        Ok(Transaction::new(tx))
    }

    /// Commit a transaction
    pub async fn commit(self) -> DatabaseResult<()> {
        // Implementation depends on the specific database driver
        // This is a placeholder
        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback(self) -> DatabaseResult<()> {
        // Implementation depends on the specific database driver
        // This is a placeholder
        Ok(())
    }

    /// Execute a query that returns no rows
    #[cfg(feature = "postgres")]
    pub async fn execute(&mut self, query: &str) -> DatabaseResult<u64> {
        self.conn.execute(query, &[]).await
    }

    /// Execute a query with parameters that returns no rows
    #[cfg(feature = "postgres")]
    pub async fn execute_with<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        self.conn.execute(query, params).await
    }
}
