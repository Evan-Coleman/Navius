use crate::error::{DatabaseError, DatabaseResult};
use crate::pool::{DatabaseRowSet, DatabaseTransaction};
use tracing::{debug, instrument};

/// Database transaction wrapper that provides operations to execute queries within a transaction
/// and commit or rollback the transaction.
///
/// # Examples
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn transfer_funds(
///     db: &DatabaseConnectionManager,
///     from_account: &str,
///     to_account: &str,
///     amount: f64,
/// ) -> Result<(), DatabaseError> {
///     // Use a transaction to ensure both operations succeed or fail together
///     db.transaction(|mut tx| async move {
///         // Deduct from source account
///         let from_query = "UPDATE accounts SET balance = balance - $1 WHERE account_id = $2 AND balance >= $1";
///         let rows = tx.execute_with(from_query, &[&amount, &from_account]).await?;
///         
///         if rows == 0 {
///             // No rows updated, likely insufficient funds
///             return Err(DatabaseError::ValidationError("Insufficient funds".to_string()));
///         }
///         
///         // Add to destination account
///         let to_query = "UPDATE accounts SET balance = balance + $1 WHERE account_id = $2";
///         tx.execute_with(to_query, &[&amount, &to_account]).await?;
///         
///         // Transaction automatically commits on success
///         Ok(())
///     }).await
/// }
/// ```
///
/// # Transaction Lifecycle
///
/// 1. Begin transaction with `DatabaseConnectionHandle::begin`
/// 2. Execute queries with `execute`, `execute_with`, `query`, or `query_with`
/// 3. End transaction with either:
///    - `commit` - Save all changes to the database
///    - `rollback` - Discard all changes
///    - Drop the transaction (causes automatic rollback)
///
pub struct Transaction<'a> {
    tx: Option<Box<dyn DatabaseTransaction>>,
    _lifetime: std::marker::PhantomData<&'a ()>,
}

impl<'a> Transaction<'a> {
    /// Create a new transaction
    pub(crate) fn new(tx: Box<dyn DatabaseTransaction>) -> Self {
        Self {
            tx: Some(tx),
            _lifetime: std::marker::PhantomData,
        }
    }

    /// Execute a query that returns no rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query))]
    pub async fn execute(&mut self, query: &str) -> DatabaseResult<u64> {
        debug!("Executing query");

        if let Some(tx) = self.tx.as_mut() {
            tx.execute(query, &[]).await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Execute a query with parameters that returns no rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query, params))]
    pub async fn execute_with<'b>(
        &mut self,
        query: &str,
        params: &[&'b (dyn sqlx::Encode<'b, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        debug!("Executing query with parameters");

        if let Some(tx) = self.tx.as_mut() {
            tx.execute(query, params).await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Execute a query that returns rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query))]
    pub async fn query(&mut self, query: &str) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        debug!("Executing query returning rows");

        if let Some(tx) = self.tx.as_mut() {
            tx.query(query, &[]).await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Execute a query with parameters that returns rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query, params))]
    pub async fn query_with<'b>(
        &mut self,
        query: &str,
        params: &[&'b (dyn sqlx::Encode<'b, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        debug!("Executing query with parameters returning rows");

        if let Some(tx) = self.tx.as_mut() {
            tx.query(query, params).await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Commit the transaction
    #[instrument(skip(self))]
    pub async fn commit(mut self) -> DatabaseResult<()> {
        debug!("Committing transaction");

        if let Some(tx) = self.tx.take() {
            tx.commit().await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Rollback the transaction
    #[instrument(skip(self))]
    pub async fn rollback(mut self) -> DatabaseResult<()> {
        debug!("Rolling back transaction");

        if let Some(tx) = self.tx.take() {
            tx.rollback().await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.tx.is_some() {
            debug!(
                "Transaction dropped without commit or rollback, will be rolled back automatically"
            );
            // We can't run async code in Drop, but we can log a warning
            // The underlying database driver will handle the rollback in its own Drop implementation
        }
    }
}
