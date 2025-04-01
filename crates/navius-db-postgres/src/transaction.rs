//! PostgreSQL transaction implementation.
//!
//! This module provides transaction support for PostgreSQL.

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::postgres::{PgConnection, PgRow};
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
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<u64, DatabaseError> {
        let result = sqlx::query(query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(result.rows_affected())
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<Vec<PgRow>, DatabaseError> {
        let result = sqlx::query(query)
            .fetch_all(&mut self.inner)
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(result)
    }

    async fn savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
        let query = format!("SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }

    async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
        let query = format!("ROLLBACK TO SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.inner)
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }

    async fn commit(self: Box<Self>) -> Result<(), DatabaseError> {
        let mut transaction = *self;
        transaction
            .inner
            .commit()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), DatabaseError> {
        let mut transaction = *self;
        transaction
            .inner
            .rollback()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(())
    }
}
