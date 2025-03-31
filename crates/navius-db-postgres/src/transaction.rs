//! PostgreSQL transaction implementation.
//!
//! This module provides transaction support for PostgreSQL.

use std::sync::Arc;

use async_trait::async_trait;
use navius_db::error::DatabaseError;
use navius_db::transaction::{Transaction, TransactionManager, TransactionOptions};
use sqlx::postgres::{PgPool, PgTransaction as SqlxPgTransaction};
use tracing::{debug, error, instrument, warn};

use crate::error::PgError;

/// A PostgreSQL transaction
pub struct PgTransaction {
    /// The underlying SQLx transaction
    inner: SqlxPgTransaction<'static>,
    /// Whether the transaction has been committed
    committed: bool,
    /// Whether the transaction has been rolled back
    rolled_back: bool,
    /// The nesting level for savepoints
    nesting_level: u32,
}

impl PgTransaction {
    /// Create a new transaction from an SQLx transaction
    pub fn new(tx: SqlxPgTransaction<'static>) -> Self {
        Self {
            inner: tx,
            committed: false,
            rolled_back: false,
            nesting_level: 0,
        }
    }

    /// Get a mutable reference to the inner SQLx transaction
    pub fn inner_mut(&mut self) -> &mut SqlxPgTransaction<'static> {
        &mut self.inner
    }

    /// Check if the transaction has been committed
    pub fn is_committed(&self) -> bool {
        self.committed
    }

    /// Check if the transaction has been rolled back
    pub fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }

    /// Get the current nesting level
    pub fn nesting_level(&self) -> u32 {
        self.nesting_level
    }

    /// Create a savepoint with the given name
    pub async fn create_savepoint(&mut self, name: &str) -> Result<(), PgError> {
        let query = format!("SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PgError::Transaction(format!("Failed to create savepoint: {}", e)))?;
        Ok(())
    }

    /// Release a savepoint with the given name
    pub async fn release_savepoint(&mut self, name: &str) -> Result<(), PgError> {
        let query = format!("RELEASE SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PgError::Transaction(format!("Failed to release savepoint: {}", e)))?;
        Ok(())
    }

    /// Rollback to a savepoint with the given name
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), PgError> {
        let query = format!("ROLLBACK TO SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PgError::Transaction(format!("Failed to rollback to savepoint: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl Transaction for PgTransaction {
    #[instrument(skip(self), level = "debug")]
    async fn commit(mut self: Box<Self>) -> Result<(), DatabaseError> {
        if self.committed || self.rolled_back {
            return Err(DatabaseError::TransactionError(
                "Transaction already committed or rolled back".to_string(),
            ));
        }

        self.inner.commit().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to commit transaction: {}", e))
        })?;

        debug!("Transaction committed");
        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn rollback(mut self: Box<Self>) -> Result<(), DatabaseError> {
        if self.committed || self.rolled_back {
            return Err(DatabaseError::TransactionError(
                "Transaction already committed or rolled back".to_string(),
            ));
        }

        self.inner.rollback().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to rollback transaction: {}", e))
        })?;

        debug!("Transaction rolled back");
        Ok(())
    }

    async fn execute_query(&mut self, query: &str) -> Result<u64, DatabaseError> {
        if self.committed || self.rolled_back {
            return Err(DatabaseError::TransactionError(
                "Transaction already committed or rolled back".to_string(),
            ));
        }

        let result = sqlx::query(query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| {
                DatabaseError::ExecutionError(format!("Failed to execute query: {}", e))
            })?;

        Ok(result.rows_affected())
    }

    async fn create_nested_transaction(&mut self) -> Result<Box<dyn Transaction>, DatabaseError> {
        if self.committed || self.rolled_back {
            return Err(DatabaseError::TransactionError(
                "Transaction already committed or rolled back".to_string(),
            ));
        }

        // Create a savepoint for the nested transaction
        let nesting_level = self.nesting_level + 1;
        let savepoint_name = format!("sp_{}", nesting_level);

        self.create_savepoint(&savepoint_name)
            .await
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        // Create a new transaction object that wraps the savepoint
        let mut nested_tx = PgTransaction::new(self.inner.clone());
        nested_tx.nesting_level = nesting_level;

        Ok(Box::new(nested_tx))
    }
}

/// A PostgreSQL transaction manager
pub struct PgTransactionManager {
    /// The PostgreSQL connection pool
    pool: PgPool,
}

impl PgTransactionManager {
    /// Create a new transaction manager with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionManager for PgTransactionManager {
    #[instrument(skip(self), level = "debug")]
    async fn begin_transaction(
        &self,
        options: Option<TransactionOptions>,
    ) -> Result<Box<dyn Transaction>, DatabaseError> {
        // Parse transaction options
        let isolation_level = if let Some(options) = &options {
            match options.isolation_level.as_deref() {
                Some("read_uncommitted") => Some("READ UNCOMMITTED"),
                Some("read_committed") => Some("READ COMMITTED"),
                Some("repeatable_read") => Some("REPEATABLE READ"),
                Some("serializable") => Some("SERIALIZABLE"),
                _ => None,
            }
        } else {
            None
        };

        let read_only = if let Some(options) = &options {
            options.read_only
        } else {
            false
        };

        // Begin the transaction
        let mut tx = self.pool.begin().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to begin transaction: {}", e))
        })?;

        // Set transaction options
        if let Some(isolation_level) = isolation_level {
            sqlx::query(&format!(
                "SET TRANSACTION ISOLATION LEVEL {}",
                isolation_level
            ))
            .execute(&mut tx)
            .await
            .map_err(|e| {
                DatabaseError::TransactionError(format!(
                    "Failed to set transaction isolation level: {}",
                    e
                ))
            })?;
        }

        if read_only {
            sqlx::query("SET TRANSACTION READ ONLY")
                .execute(&mut tx)
                .await
                .map_err(|e| {
                    DatabaseError::TransactionError(format!(
                        "Failed to set transaction read only: {}",
                        e
                    ))
                })?;
        }

        debug!("Transaction started");
        Ok(Box::new(PgTransaction::new(tx)))
    }

    #[instrument(skip(self, callback), level = "debug")]
    async fn with_transaction<F, T, E>(&self, callback: F) -> Result<T, DatabaseError>
    where
        F: for<'t> FnOnce(
                Box<dyn Transaction>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 't>,
            > + Send
            + 'static,
        T: Send + 'static,
        E: Into<DatabaseError> + Send + 'static,
    {
        let tx = self.begin_transaction(None).await?;

        match callback(tx.clone()).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(err) => {
                // Try to roll back the transaction
                if let Err(rollback_err) = tx.rollback().await {
                    // Log rollback error, but return the original error
                    error!("Failed to rollback transaction: {}", rollback_err);
                }
                Err(err.into())
            }
        }
    }
}
