use crate::error::DatabaseError;
use crate::transaction::{DatabaseTransaction, PgTransaction};
use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{Acquire, Executor};

/// A generic database connection trait for all database implementations.
/// This trait defines the common operations that can be performed on a connection.
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Start a new transaction.
    async fn begin<'life0>(
        &'life0 mut self,
    ) -> Result<Box<dyn DatabaseTransaction + 'life0>, DatabaseError>
    where
        Self: 'life0;

    /// Execute a SQL query and return the number of affected rows.
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<u64, DatabaseError>;

    /// Execute a SQL query and return the results.
    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<Vec<PgRow>, DatabaseError>;
}

/// PostgreSQL implementation of the database connection.
pub struct PgConnection {
    conn: sqlx::PgConnection,
}

impl PgConnection {
    /// Create a new PostgreSQL connection.
    pub fn new(conn: sqlx::PgConnection) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl DatabaseConnection for PgConnection {
    async fn begin<'life0>(
        &'life0 mut self,
    ) -> Result<Box<dyn DatabaseTransaction + 'life0>, DatabaseError>
    where
        Self: 'life0,
    {
        let tx = Acquire::begin(&mut self.conn)
            .await
            .map_err(|e| DatabaseError::TransactionError(e.to_string()))?;

        Ok(Box::new(PgTransaction::new(tx)))
    }

    async fn execute(
        &mut self,
        query: &str,
        _params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<u64, DatabaseError> {
        // Use the connection directly to execute the query
        // For now, we ignore parameters to simplify execution
        self.conn
            .execute(query)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
            .map(|result| result.rows_affected())
    }

    async fn query(
        &mut self,
        query: &str,
        _params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<Vec<PgRow>, DatabaseError> {
        // Use the connection directly to fetch results
        // For now, we ignore parameters to simplify execution
        self.conn
            .fetch_all(query)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }
}
