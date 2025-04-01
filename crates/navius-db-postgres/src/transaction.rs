//! PostgreSQL transaction implementation.
//!
//! This module provides transaction support for PostgreSQL.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::postgres::PgConnection;
use sqlx::{Executor, Postgres, Transaction};
use tracing::{debug, error, instrument};

use navius_db::error::DatabaseError;
use navius_db::transaction::DatabaseTransaction;

use crate::error::PostgresError;

/// A PostgreSQL transaction
pub struct PostgresTransaction {
    /// The underlying SQLx transaction
    inner: Transaction<'static, Postgres>,
}

impl PostgresTransaction {
    /// Create a new PostgreSQL transaction
    pub(crate) fn new(transaction: Transaction<'static, Postgres>) -> Self {
        Self { inner: transaction }
    }

    /// Get a reference to the underlying SQLx transaction
    pub fn inner(&self) -> &Transaction<'static, Postgres> {
        &self.inner
    }

    /// Get a mutable reference to the underlying SQLx transaction
    pub fn inner_mut(&mut self) -> &mut Transaction<'static, Postgres> {
        &mut self.inner
    }
}

#[async_trait]
impl DatabaseTransaction for PostgresTransaction {
    async fn commit(self: Arc<Self>) -> Result<(), DatabaseError> {
        let mut transaction = self.inner;
        transaction
            .commit()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }

    async fn rollback(self: Arc<Self>) -> Result<(), DatabaseError> {
        let mut transaction = self.inner;
        transaction
            .rollback()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }

    async fn execute<Q: Send + Sync>(&self, query: Q) -> Result<u64, DatabaseError>
    where
        Q: AsRef<str>,
    {
        let result = self
            .inner
            .execute(query.as_ref())
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(result.rows_affected())
    }
}
