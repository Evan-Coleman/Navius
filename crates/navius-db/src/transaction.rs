use crate::error::DatabaseError;
use async_trait::async_trait;
use sqlx::postgres::PgRow;
use sqlx::{Executor, Postgres, Transaction};

/// A wrapper for [`Transaction`] to provide a common interface
/// across different database implementations.
pub struct PgTransaction<'t> {
    conn: Transaction<'t, Postgres>,
}

impl<'t> PgTransaction<'t> {
    pub fn new(tx: Transaction<'t, Postgres>) -> Self {
        Self { conn: tx }
    }

    /// Inner access for direct sqlx interaction
    /// This can be useful when you need access to specific sqlx functionality
    pub fn inner_transaction(&mut self) -> &mut Transaction<'t, Postgres> {
        &mut self.conn
    }
}

/// A generic database transaction trait for all database implementations.
/// This trait defines the common operations that can be performed on a transaction.
#[async_trait]
pub trait DatabaseTransaction: Send + Sync {
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

    /// Create a savepoint for the transaction.
    async fn savepoint(&mut self, name: &str) -> Result<(), DatabaseError>;

    /// Rollback to a savepoint in the transaction.
    async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError>;

    /// Commit the transaction.
    async fn commit(self: Box<Self>) -> Result<(), DatabaseError>;

    /// Rollback the transaction.
    async fn rollback(self: Box<Self>) -> Result<(), DatabaseError>;
}

/// Implementation of the transaction trait for PostgreSQL.
#[async_trait]
impl<'t> DatabaseTransaction for PgTransaction<'t> {
    async fn execute(
        &mut self,
        query: &str,
        _params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<u64, DatabaseError> {
        // Use the Transaction executor to execute the query
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
        // Use the Transaction executor to execute the query and fetch results
        // For now, we ignore parameters to simplify execution
        self.conn
            .fetch_all(query)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    async fn savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
        let savepoint_query = format!("SAVEPOINT {}", name);
        self.conn
            .execute(&*savepoint_query)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
            .map(|_| ())
    }

    async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
        let rollback_query = format!("ROLLBACK TO SAVEPOINT {}", name);
        self.conn
            .execute(&*rollback_query)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
            .map(|_| ())
    }

    async fn commit(self: Box<Self>) -> Result<(), DatabaseError> {
        // Unbox the transaction
        let transaction = *self;
        transaction
            .conn
            .commit()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }

    async fn rollback(self: Box<Self>) -> Result<(), DatabaseError> {
        // Unbox the transaction
        let transaction = *self;
        transaction
            .conn
            .rollback()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))
    }
}

/// A test example that demonstrates how to use transactions with savepoints
#[cfg(test)]
mod examples {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn transaction_savepoints_example() -> Result<(), DatabaseError> {
        // In a real application, you would get the connection from a connection pool
        // This is just an example to show the usage of transactions and savepoints
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/postgres".to_string());

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        let mut conn = pool
            .acquire()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;

        // Create a transaction
        let pg_tx = conn
            .begin()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let mut tx = PgTransaction::new(pg_tx);

        // Create a table for the example
        tx.execute(
            "CREATE TABLE IF NOT EXISTS transaction_test (id SERIAL PRIMARY KEY, data TEXT, created_at TIMESTAMPTZ)",
            &[],
        )
        .await?;

        // Insert initial data
        let now = Utc::now();
        tx.execute(
            "INSERT INTO transaction_test (data, created_at) VALUES ($1, $2)",
            &[&"Initial data", &now],
        )
        .await?;

        // Create a savepoint
        tx.savepoint("sp1").await?;

        // Insert more data after savepoint
        tx.execute(
            "INSERT INTO transaction_test (data, created_at) VALUES ($1, $2)",
            &[&"Data after savepoint", &now],
        )
        .await?;

        // Query to see both rows
        let rows = tx
            .query("SELECT * FROM transaction_test ORDER BY id", &[])
            .await?;
        assert_eq!(rows.len(), 2);

        // Rollback to the savepoint
        tx.rollback_to_savepoint("sp1").await?;

        // Now we should only have the initial data
        let rows = tx
            .query("SELECT * FROM transaction_test ORDER BY id", &[])
            .await?;
        assert_eq!(rows.len(), 1);

        // Create another savepoint
        tx.savepoint("sp2").await?;

        // Insert different data
        tx.execute(
            "INSERT INTO transaction_test (data, created_at) VALUES ($1, $2)",
            &[&"Different data", &now],
        )
        .await?;

        // This time we'll keep the savepoint and commit the transaction
        Box::new(tx).commit().await?;

        // Clean up (in a new transaction)
        let pg_tx = conn
            .begin()
            .await
            .map_err(|e| DatabaseError::QueryError(e.to_string()))?;
        let mut cleanup_tx = PgTransaction::new(pg_tx);
        cleanup_tx
            .execute("DROP TABLE transaction_test", &[])
            .await?;
        Box::new(cleanup_tx).commit().await?;

        Ok(())
    }
}
