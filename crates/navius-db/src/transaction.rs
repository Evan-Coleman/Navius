use crate::error::{DatabaseError, DatabaseResult};
use crate::pool::{DatabaseRowSet, DatabaseTransaction};
use std::sync::atomic::{AtomicUsize, Ordering};
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
/// # Savepoints
///
/// Savepoints allow you to create checkpoints within a transaction that you can later
/// roll back to if needed. This is useful for complex transactions where you might want
/// to retry parts of a transaction without restarting the entire transaction.
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn complex_operation(db: &DatabaseConnectionManager) -> Result<(), DatabaseError> {
///     db.transaction(|mut tx| async move {
///         // First part of transaction
///         tx.execute("INSERT INTO logs (message) VALUES ('Starting operation')").await?;
///         
///         // Create a savepoint before risky operation
///         tx.savepoint("before_risky_part").await?;
///         
///         // Try risky operation
///         let result = tx.execute("UPDATE accounts SET status = 'PROCESSING' WHERE id = 123").await;
///         
///         if result.is_err() {
///             // Roll back to savepoint if the operation failed
///             tx.rollback_to_savepoint("before_risky_part").await?;
///             
///             // Log the failure
///             tx.execute("INSERT INTO logs (message) VALUES ('Operation failed, rolled back')").await?;
///         } else {
///             // Operation succeeded, release the savepoint
///             tx.release_savepoint("before_risky_part").await?;
///             
///             // Complete the rest of the transaction
///             tx.execute("INSERT INTO logs (message) VALUES ('Operation completed')").await?;
///         }
///         
///         Ok(())
///     }).await
/// }
/// ```
pub struct Transaction<'a> {
    tx: Option<Box<dyn DatabaseTransaction>>,
    _lifetime: std::marker::PhantomData<&'a ()>,
    savepoint_counter: AtomicUsize,
}

impl<'a> Transaction<'a> {
    /// Create a new transaction
    pub(crate) fn new(tx: Box<dyn DatabaseTransaction>) -> Self {
        Self {
            tx: Some(tx),
            _lifetime: std::marker::PhantomData,
            savepoint_counter: AtomicUsize::new(0),
        }
    }

    /// Check if the transaction is still active
    fn check_active(&self) -> DatabaseResult<()> {
        if self.tx.is_none() {
            return Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ));
        }
        Ok(())
    }

    /// Execute a query that returns no rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query))]
    pub async fn execute(&mut self, query: &str) -> DatabaseResult<u64> {
        debug!("Executing query");

        self.check_active()?;
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

        self.check_active()?;
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

        self.check_active()?;
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

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.query(query, params).await
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Create a savepoint within the transaction
    ///
    /// Savepoints allow you to create checkpoints within a transaction that you can
    /// later roll back to if needed. This is useful for complex transactions where
    /// you might want to retry parts of a transaction without restarting the entire
    /// transaction.
    #[instrument(skip(self))]
    pub async fn savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        debug!(savepoint = name, "Creating savepoint");

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.execute(&format!("SAVEPOINT {}", name), &[]).await?;
            Ok(())
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Create an automatically named savepoint
    ///
    /// This creates a savepoint with an automatically generated name that
    /// is guaranteed to be unique within this transaction.
    #[instrument(skip(self))]
    pub async fn auto_savepoint(&mut self) -> DatabaseResult<String> {
        let counter = self.savepoint_counter.fetch_add(1, Ordering::SeqCst);
        let name = format!("sp_{}", counter);

        self.savepoint(&name).await?;
        Ok(name)
    }

    /// Roll back to a savepoint
    ///
    /// This rolls back all changes made after the savepoint was created.
    /// The savepoint remains valid and can be used again.
    #[instrument(skip(self))]
    pub async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        debug!(savepoint = name, "Rolling back to savepoint");

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.execute(&format!("ROLLBACK TO SAVEPOINT {}", name), &[])
                .await?;
            Ok(())
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Release a savepoint
    ///
    /// This releases a previously created savepoint. After a savepoint is released,
    /// you can no longer roll back to it. All changes made after the savepoint are
    /// kept in the transaction.
    #[instrument(skip(self))]
    pub async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        debug!(savepoint = name, "Releasing savepoint");

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.execute(&format!("RELEASE SAVEPOINT {}", name), &[])
                .await?;
            Ok(())
        } else {
            Err(DatabaseError::TransactionError(
                "Transaction has already been committed or rolled back".to_string(),
            ))
        }
    }

    /// Execute a nested transaction
    ///
    /// This creates a savepoint, executes the provided closure within that savepoint,
    /// and then either commits or rolls back the savepoint based on the result.
    ///
    /// If the closure returns Ok, the savepoint is released (committed).
    /// If the closure returns Err, the transaction is rolled back to the savepoint.
    ///
    /// This allows you to have transaction-like semantics within a transaction.
    #[instrument(skip(self, f))]
    pub async fn nested<F, Fut, T>(&mut self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        debug!("Starting nested transaction");

        self.check_active()?;

        // Create a savepoint with an auto-generated name
        let savepoint_name = self.auto_savepoint().await?;

        // Execute the closure
        match f().await {
            Ok(result) => {
                // Release the savepoint on success
                self.release_savepoint(&savepoint_name).await?;
                Ok(result)
            }
            Err(err) => {
                // Roll back to the savepoint on error
                self.rollback_to_savepoint(&savepoint_name).await?;
                Err(err)
            }
        }
    }

    /// Commit the transaction
    #[instrument(skip(self))]
    pub async fn commit(mut self) -> DatabaseResult<()> {
        debug!("Committing transaction");

        self.check_active()?;
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

        self.check_active()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DatabaseError;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;

    mock! {
        pub DatabaseTx {}

        impl DatabaseTransaction for DatabaseTx {
            async fn execute(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<u64>;
            async fn query(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<Box<dyn DatabaseRowSet>>;
            async fn commit(self: Box<Self>) -> DatabaseResult<()>;
            async fn rollback(self: Box<Self>) -> DatabaseResult<()>;
        }
    }

    #[tokio::test]
    async fn test_savepoint_creation() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect savepoint creation
        mock_tx
            .expect_execute()
            .with(
                eq("SAVEPOINT test_point"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx.savepoint("test_point").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rollback_to_savepoint() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect rollback to savepoint
        mock_tx
            .expect_execute()
            .with(
                eq("ROLLBACK TO SAVEPOINT test_point"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx.rollback_to_savepoint("test_point").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_release_savepoint() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect savepoint release
        mock_tx
            .expect_execute()
            .with(
                eq("RELEASE SAVEPOINT test_point"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx.release_savepoint("test_point").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_auto_savepoint() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect savepoint creation with auto-generated name
        mock_tx
            .expect_execute()
            .with(
                eq("SAVEPOINT sp_0"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx.auto_savepoint().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sp_0");
    }

    #[tokio::test]
    async fn test_nested_transaction_success() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect auto savepoint
        mock_tx
            .expect_execute()
            .with(
                eq("SAVEPOINT sp_0"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        // Expect release of savepoint (success case)
        mock_tx
            .expect_execute()
            .with(
                eq("RELEASE SAVEPOINT sp_0"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx.nested(|| async { Ok(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_nested_transaction_failure() {
        let mut mock_tx = MockDatabaseTx::new();

        // Expect auto savepoint
        mock_tx
            .expect_execute()
            .with(
                eq("SAVEPOINT sp_0"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        // Expect rollback to savepoint (failure case)
        mock_tx
            .expect_execute()
            .with(
                eq("ROLLBACK TO SAVEPOINT sp_0"),
                eq(&[] as &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]),
            )
            .returning(|_, _| Ok(1));

        let mut tx = Transaction::new(Box::new(mock_tx));
        let result = tx
            .nested(|| async { Err(DatabaseError::QueryError("test error".to_string())) })
            .await;

        assert!(result.is_err());
        match result {
            Err(DatabaseError::QueryError(msg)) => assert_eq!(msg, "test error"),
            _ => panic!("Unexpected error type"),
        }
    }
}
