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
    pub async fn transaction<F, Fut, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(Transaction<'_>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, E>> + Send + 'static,
        R: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
    {
        debug!("Starting database transaction");
        let mut conn = self.get_connection().await.map_err(E::from)?;
        let tx = conn.begin().await.map_err(E::from)?;

        match f(tx).await {
            Ok(value) => {
                debug!("Transaction completed successfully, committing changes");
                // Since the transaction was returned to us and consumed in the closure,
                // we don't need to commit it explicitly here.
                // The dropped connection will be returned to the pool.
                Ok(value)
            }
            Err(err) => {
                debug!("Transaction failed, rolling back changes");
                // Since the transaction was returned to us and consumed in the closure,
                // a rollback will be automatically performed by the Drop implementation.
                // The dropped connection will be returned to the pool.
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
        // Implementation note: This method is typically not used directly.
        // Transactions are committed using the Transaction::commit method.
        // This is provided primarily for the transaction() method on DatabaseConnectionManager.
        // Since we consume self, there's not much to do here.
        Ok(())
    }

    /// Rollback a transaction
    pub async fn rollback(self) -> DatabaseResult<()> {
        // Implementation note: This method is typically not used directly.
        // Transactions are rolled back using the Transaction::rollback method.
        // This is provided primarily for the transaction() method on DatabaseConnectionManager.
        // Since we consume self, there's not much to do here.
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
