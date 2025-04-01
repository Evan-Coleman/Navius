use crate::connection::DatabaseConnection;
use crate::error::DatabaseError;
use crate::transaction::DatabaseTransaction;
use async_trait::async_trait;
use sqlx::postgres::PgRow;

/// A repository trait for database access.
/// This trait provides a common interface for database operations.
#[async_trait]
pub trait Repository: Send + Sync {
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

    /// Start a new transaction.
    async fn begin<'a>(&'a mut self) -> Result<Box<dyn DatabaseTransaction + 'a>, DatabaseError>;

    /// Execute a function within a transaction.
    /// The transaction will be automatically committed if the function returns Ok,
    /// or rolled back if it returns Err.
    async fn with_transaction<F, T, E>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(
                Box<dyn DatabaseTransaction + 'c>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        E: From<DatabaseError> + Send,
        T: Send;
}

/// A repository implementation for a database connection.
pub struct ConnectionRepository<C: DatabaseConnection> {
    conn: C,
}

impl<C: DatabaseConnection> ConnectionRepository<C> {
    /// Create a new repository from a database connection.
    pub fn new(conn: C) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl<C: DatabaseConnection> Repository for ConnectionRepository<C> {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<u64, DatabaseError> {
        self.conn.execute(query, params).await
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync + Send)],
    ) -> Result<Vec<PgRow>, DatabaseError> {
        self.conn.query(query, params).await
    }

    async fn begin<'a>(&'a mut self) -> Result<Box<dyn DatabaseTransaction + 'a>, DatabaseError> {
        self.conn.begin().await
    }

    async fn with_transaction<F, T, E>(&mut self, f: F) -> Result<T, E>
    where
        F: for<'c> FnOnce(
                Box<dyn DatabaseTransaction + 'c>,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<T, E>> + Send + 'c>,
            > + Send,
        E: From<DatabaseError> + Send,
        T: Send,
    {
        let tx = self.begin().await.map_err(E::from)?;

        let result = f(tx).await;

        match result {
            Ok(value) => Ok(value),
            Err(err) => Err(err),
        }
    }
}
