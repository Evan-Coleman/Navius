use crate::error::{DatabaseError, DatabaseResult};
use crate::pool::{DatabaseRowSet, DatabaseTransaction};
use async_trait::async_trait;
use futures::future::BoxFuture;
use std::fmt::{self, Debug, Display};
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, error, info, instrument, warn};

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
///
/// # Nested Transactions
///
/// You can use the `nested` method to create a nested transaction using savepoints.
/// This allows you to have a transaction within a transaction, with the ability to
/// rollback only the nested part if needed.
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn complex_nested_operation(db: &DatabaseConnectionManager) -> Result<(), DatabaseError> {
///     db.transaction(|mut tx| async move {
///         // Main transaction operations
///         tx.execute("INSERT INTO logs (message) VALUES ('Starting parent operation')").await?;
///         
///         // Begin a nested transaction
///         tx.nested(|| async {
///             // Operations in nested transaction
///             tx.execute("INSERT INTO logs (message) VALUES ('Starting nested operation')").await?;
///             
///             // This will only be rolled back if the nested transaction fails
///             let result = tx.execute("UPDATE accounts SET status = 'PROCESSING' WHERE id = 123").await;
///             
///             if let Err(e) = result {
///                 // Return the error, which will rollback the nested transaction
///                 return Err(e);
///             }
///             
///             // This completes successfully, so the nested transaction will be committed
///             tx.execute("INSERT INTO logs (message) VALUES ('Nested operation successful')").await
///         }).await?;
///         
///         // Continue with parent transaction, even if nested transaction failed
///         tx.execute("INSERT INTO logs (message) VALUES ('Parent operation continuing')").await?;
///         
///         Ok(())
///     }).await
/// }
/// ```
///
/// # Nested Transaction Sequence
///
/// You can execute multiple nested transactions sequentially using the `nested_sequence` method.
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn complex_nested_sequence(db: &DatabaseConnectionManager) -> Result<(), DatabaseError> {
///     db.transaction(|mut tx| async move {
///         // Level 1
///         tx.deep_nested(1, |mut tx| async move {
///             tx.execute("INSERT INTO logs (message) VALUES ('Level 1')").await?;
///             
///             // Level 2
///             tx.deep_nested(2, |mut tx| async move {
///                 tx.execute("INSERT INTO logs (message) VALUES ('Level 2')").await?;
///                 
///                 // Level 3 - will fail
///                 let level3_result = tx.deep_nested(3, |mut tx| async move {
///                     tx.execute("INSERT INTO logs (message) VALUES ('Level 3')").await?;
///                     Err(DatabaseError::ValidationError("Level 3 error".to_string()))
///                 }).await;
///                 
///                 // Level 3 failed but Level 2 continues
///                 assert!(level3_result.is_err());
///                 tx.execute("INSERT INTO logs (message) VALUES ('Level 2 continues')").await
///             }).await?;
///             
///             tx.execute("INSERT INTO logs (message) VALUES ('Level 1 continues')").await
///         }).await
///     }).await
/// }
/// ```
///
/// # Deep Nested Transactions
///
/// You can execute deeply nested transactions with proper cleanup using the `deep_nested` method.
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn complex_deep_nested(db: &DatabaseConnectionManager) -> Result<(), DatabaseError> {
///     db.transaction(|mut tx| async move {
///         // Level 1
///         tx.deep_nested(1, |mut tx| async move {
///             tx.execute("INSERT INTO logs (message) VALUES ('Level 1')").await?;
///             
///             // Level 2
///             tx.deep_nested(2, |mut tx| async move {
///                 tx.execute("INSERT INTO logs (message) VALUES ('Level 2')").await?;
///                 
///                 // Level 3 - will fail
///                 let level3_result = tx.deep_nested(3, |mut tx| async move {
///                     tx.execute("INSERT INTO logs (message) VALUES ('Level 3')").await?;
///                     Err(DatabaseError::ValidationError("Level 3 error".to_string()))
///                 }).await;
///                 
///                 // Level 3 failed but Level 2 continues
///                 assert!(level3_result.is_err());
///                 tx.execute("INSERT INTO logs (message) VALUES ('Level 2 continues')").await
///             }).await?;
///             
///             tx.execute("INSERT INTO logs (message) VALUES ('Level 1 continues')").await
///         }).await
///     }).await
/// }
/// ```
///
/// # Deep Nested Transactions
///
/// This method supports creating multiple levels of nested transactions.
/// Each level creates a new savepoint that can be rolled back independently.
///
/// # Examples
///
/// ```rust
/// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
///
/// async fn deep_nested_example(db: &DatabaseConnectionManager) -> Result<i32, DatabaseError> {
///     db.transaction(|mut tx| async move {
///         // Start a level 1 transaction
///         let result = tx.deep_nested(1, Box::new(|mut tx| async move {
///             // Do level 1 operations
///             let level1_value = 10;
///             
///             // Start a level 2 transaction
///             let level2_result = tx.deep_nested(2, Box::new(|mut tx| async move {
///                 // Do level 2 operations
///                 let level2_value = 20;
///                 
///                 // If this returns an error, only level 2 is rolled back
///                 Ok(level2_value)
///             })).await?;
///             
///             // Continue with level 1 operations
///             Ok(level1_value + level2_result)
///         })).await?;
///         
///         Ok(result)
///     }).await
/// }
/// ```
///
pub struct Transaction<'a> {
    tx: Option<Box<dyn DatabaseTransaction>>,
    _lifetime: std::marker::PhantomData<&'a ()>,
    savepoint_counter: AtomicUsize,
    /// Stack of active savepoints
    savepoint_stack: Vec<String>,
    /// Transaction nesting level - incremented for each nested transaction
    nesting_level: usize,
}

impl<'a> Transaction<'a> {
    /// Create a new transaction
    pub(crate) fn new(tx: Box<dyn DatabaseTransaction>) -> Self {
        Self {
            tx: Some(tx),
            _lifetime: std::marker::PhantomData,
            savepoint_counter: AtomicUsize::new(0),
            savepoint_stack: Vec::new(),
            nesting_level: 0,
        }
    }

    /// Check if the transaction is still active
    fn check_active(&self) -> DatabaseResult<()> {
        if self.tx.is_none() {
            return Err(DatabaseError::TransactionFinished);
        }
        Ok(())
    }

    /// Execute a query that returns no rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query))]
    pub async fn execute(&mut self, query: &str) -> DatabaseResult<u64> {
        debug!(nesting_level = self.nesting_level, "Executing query");

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.execute(query, &[]).await
        } else {
            Err(DatabaseError::TransactionFinished)
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
        debug!(
            nesting_level = self.nesting_level,
            "Executing query with parameters"
        );

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.execute(query, params).await
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }

    /// Execute a query that returns rows
    #[cfg(feature = "postgres")]
    #[instrument(skip(self, query))]
    pub async fn query(&mut self, query: &str) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        debug!(
            nesting_level = self.nesting_level,
            "Executing query returning rows"
        );

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.query(query, &[]).await
        } else {
            Err(DatabaseError::TransactionFinished)
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
        debug!(
            nesting_level = self.nesting_level,
            "Executing query with parameters returning rows"
        );

        self.check_active()?;
        if let Some(tx) = self.tx.as_mut() {
            tx.query(query, params).await
        } else {
            Err(DatabaseError::TransactionFinished)
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
        debug!(
            savepoint = name,
            nesting_level = self.nesting_level,
            "Creating savepoint"
        );

        self.check_active()?;

        // Validate savepoint name
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DatabaseError::savepoint_error(format!(
                "Invalid savepoint name: {}. Only alphanumeric characters and underscores are allowed.",
                name
            )));
        }

        if let Some(tx) = self.tx.as_mut() {
            tx.savepoint(name).await?;
            self.savepoint_stack.push(name.to_string());
            Ok(())
        } else {
            Err(DatabaseError::TransactionFinished)
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
        debug!(
            savepoint = name,
            nesting_level = self.nesting_level,
            "Rolling back to savepoint"
        );

        self.check_active()?;

        // Find the savepoint in the stack
        let pos = self.savepoint_stack.iter().rposition(|s| s == name);

        if pos.is_none() {
            return Err(DatabaseError::savepoint_error(format!(
                "No savepoint with name '{}' found in this transaction",
                name
            )));
        }

        if let Some(tx) = self.tx.as_mut() {
            tx.rollback_to_savepoint(name).await?;

            // Remove all savepoints that were created after this one
            let pos = pos.unwrap();
            self.savepoint_stack.truncate(pos + 1);

            Ok(())
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }

    /// Release a savepoint
    ///
    /// This releases a previously created savepoint. After a savepoint is released,
    /// you can no longer roll back to it. All changes made after the savepoint are
    /// kept in the transaction.
    #[instrument(skip(self))]
    pub async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        debug!(
            savepoint = name,
            nesting_level = self.nesting_level,
            "Releasing savepoint"
        );

        self.check_active()?;

        // Find the savepoint in the stack
        let pos = self.savepoint_stack.iter().rposition(|s| s == name);

        if pos.is_none() {
            return Err(DatabaseError::savepoint_error(format!(
                "No savepoint with name '{}' found in this transaction",
                name
            )));
        }

        if let Some(tx) = self.tx.as_mut() {
            tx.release_savepoint(name).await?;

            // Remove the savepoint from the stack
            let pos = pos.unwrap();
            self.savepoint_stack.remove(pos);

            Ok(())
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }

    /// Execute an operation in a nested transaction (savepoint).
    ///
    /// This method creates a savepoint, executes the provided operation, and then either
    /// releases or rolls back the savepoint based on the success or failure of the operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
    ///
    /// async fn nested_example(db: &DatabaseConnectionManager) -> Result<i32, DatabaseError> {
    ///     db.transaction(|mut tx| async move {
    ///         // Execute an operation in a nested transaction
    ///         let result = tx.nested(Box::new(|tx| async move {
    ///             // If this fails, only this operation is rolled back
    ///             tx.execute("INSERT INTO users (name) VALUES ('Alice')").await?;
    ///             Ok(42)
    ///         })).await?;
    ///
    ///         Ok(result)
    ///     }).await
    /// }
    /// ```
    ///
    /// If the operation fails, the savepoint is rolled back and the transaction can continue.
    ///
    #[instrument(skip(self, operation))]
    pub async fn nested<T, E, F>(
        &mut self,
        operation: Box<dyn FnOnce(&mut Transaction) -> F + Send + Sync>,
    ) -> Result<T, E>
    where
        T: Send + 'static,
        E: From<DatabaseError> + Send + Display + 'static,
        F: Future<Output = Result<T, E>> + Send,
    {
        let savepoint_name = format!(
            "sp_{}",
            self.savepoint_counter.fetch_add(1, Ordering::SeqCst)
        );

        debug!(
            savepoint = savepoint_name,
            "Creating savepoint for nested transaction"
        );

        // Create a savepoint
        match self.savepoint(&savepoint_name).await {
            Ok(_) => {
                self.savepoint_stack.push(savepoint_name.clone());
            }
            Err(err) => {
                return Err(err
                    .with_context(format!("Failed to create savepoint for nested transaction"))
                    .into());
            }
        }

        // Execute the operation
        match operation(self).await {
            Ok(value) => {
                // Release the savepoint on success
                debug!(
                    savepoint = savepoint_name,
                    "Releasing savepoint after successful operation"
                );

                match self.release_savepoint(&savepoint_name).await {
                    Ok(_) => {
                        // Remove from stack
                        if let Some(idx) = self
                            .savepoint_stack
                            .iter()
                            .position(|s| s == &savepoint_name)
                        {
                            self.savepoint_stack.remove(idx);
                        }
                        Ok(value)
                    }
                    Err(err) => {
                        // Just log if release fails, operation succeeded
                        warn!(
                            error = %err,
                            savepoint = savepoint_name,
                            "Failed to release savepoint after successful operation"
                        );

                        // Remove from stack anyway
                        if let Some(idx) = self
                            .savepoint_stack
                            .iter()
                            .position(|s| s == &savepoint_name)
                        {
                            self.savepoint_stack.remove(idx);
                        }
                        Ok(value)
                    }
                }
            }
            Err(err) => {
                info!(
                    savepoint = savepoint_name,
                    "Rolling back savepoint due to operation failure"
                );

                // Try to rollback to the savepoint
                let rollback_result = self.rollback_to_savepoint(&savepoint_name).await;

                // Remove from stack regardless of result
                if let Some(idx) = self
                    .savepoint_stack
                    .iter()
                    .position(|s| s == &savepoint_name)
                {
                    self.savepoint_stack.remove(idx);
                }

                // If rollback failed, return a compound error
                if let Err(rollback_err) = rollback_result {
                    error!(
                        rollback_error = %rollback_err,
                        savepoint = savepoint_name,
                        "Failed to rollback savepoint after operation failure"
                    );

                    return Err(DatabaseError::with_context(
                        rollback_err,
                        format!("Failed to rollback savepoint: {}", err),
                    )
                    .into());
                }

                // Return the original error
                Err(err)
            }
        }
    }

    /// Execute a function with retries using savepoints
    ///
    /// This creates a savepoint, executes the provided closure, and either
    /// releases the savepoint if the closure succeeds or rolls back to the
    /// savepoint and retries if the closure fails.
    ///
    /// This is useful for operations that might fail due to transient issues
    /// like deadlocks or serialization failures.
    #[instrument(skip(self, f))]
    pub async fn with_retry<F, Fut, T>(&mut self, max_attempts: usize, f: F) -> DatabaseResult<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = DatabaseResult<T>> + Send,
    {
        debug!(max_attempts, "Executing function with retries");

        self.check_active()?;

        if max_attempts == 0 {
            return Err(DatabaseError::ValidationError(
                "max_attempts must be greater than 0".to_string(),
            ));
        }

        // Create a savepoint with an auto-generated name
        let savepoint_name = self.auto_savepoint().await?;

        let mut attempt = 0;
        let mut last_error = None;

        while attempt < max_attempts {
            attempt += 1;
            debug!(attempt, max_attempts, "Retry attempt");

            match f().await {
                Ok(result) => {
                    // Release the savepoint on success
                    self.release_savepoint(&savepoint_name).await?;
                    return Ok(result);
                }
                Err(err) => {
                    if attempt < max_attempts {
                        // Log the error and retry
                        warn!(
                            error = %err,
                            attempt,
                            max_attempts,
                            "Operation failed, rolling back to savepoint and retrying"
                        );

                        // Roll back to the savepoint
                        self.rollback_to_savepoint(&savepoint_name).await?;
                    }

                    last_error = Some(err);
                }
            }
        }

        // If we get here, all attempts failed
        let err = last_error.unwrap_or_else(|| {
            DatabaseError::UnexpectedStateError(
                "No error was recorded but all retry attempts failed".to_string(),
            )
        });

        error!(
            error = %err,
            attempts = attempt,
            "All retry attempts failed"
        );

        // Clean up the savepoint
        let _ = self.rollback_to_savepoint(&savepoint_name).await;

        Err(err)
    }

    /// Commit the transaction
    #[instrument(skip(self))]
    pub async fn commit(mut self) -> DatabaseResult<()> {
        debug!(
            nesting_level = self.nesting_level,
            savepoints = self.savepoint_stack.len(),
            "Committing transaction"
        );

        self.check_active()?;

        // Check if there are any unreleased savepoints
        if !self.savepoint_stack.is_empty() {
            warn!(
                savepoints = ?self.savepoint_stack,
                "Committing transaction with unreleased savepoints"
            );
        }

        if let Some(tx) = self.tx.take() {
            tx.commit().await
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }

    /// Rollback the transaction
    #[instrument(skip(self))]
    pub async fn rollback(mut self) -> DatabaseResult<()> {
        debug!(
            nesting_level = self.nesting_level,
            savepoints = self.savepoint_stack.len(),
            "Rolling back transaction"
        );

        self.check_active()?;
        if let Some(tx) = self.tx.take() {
            tx.rollback().await
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }

    /// Get the current nesting level of the transaction
    pub fn nesting_level(&self) -> usize {
        self.nesting_level
    }

    /// Get the active savepoints in this transaction
    pub fn active_savepoints(&self) -> &[String] {
        &self.savepoint_stack
    }

    /// Run a sequence of operations in nested transactions, collecting their results.
    ///
    /// This method executes each operation in its own savepoint, collecting the results of successful
    /// operations. If any operation fails, the sequence is halted and the error is returned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use navius_db::{DatabaseConnectionManager, PgPool, PoolOptions, DatabaseError};
    ///
    /// async fn sequence_example(db: &DatabaseConnectionManager) -> Result<Vec<i32>, DatabaseError> {
    ///     db.transaction(|mut tx| async move {
    ///         // Execute a sequence of nested operations
    ///         let results = tx.nested_sequence(vec![
    ///             Box::new(|| async { Ok::<i32, DatabaseError>(10) }),
    ///             Box::new(|| async { Ok::<i32, DatabaseError>(20) }),
    ///             Box::new(|| async { Ok::<i32, DatabaseError>(30) }),
    ///         ]).await?;
    ///
    ///         // results will be [10, 20, 30]
    ///         Ok(results)
    ///     }).await
    /// }
    /// ```
    #[instrument(skip(self, operations))]
    pub async fn nested_sequence<T, E, F>(
        &mut self,
        operations: Vec<Box<dyn FnOnce() -> F + Send + Sync>>,
    ) -> Result<Vec<T>, E>
    where
        T: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
        F: Future<Output = Result<T, E>> + Send,
    {
        let mut results = Vec::with_capacity(operations.len());

        for (i, operation) in operations.into_iter().enumerate() {
            let savepoint_name = format!(
                "sp_{}",
                self.savepoint_counter.fetch_add(1, Ordering::SeqCst)
            );

            debug!(
                savepoint = savepoint_name,
                operation_index = i,
                "Creating savepoint for nested transaction sequence"
            );

            // Create a savepoint for this operation
            match self.savepoint(&savepoint_name).await {
                Ok(_) => {
                    self.savepoint_stack.push(savepoint_name.clone());
                }
                Err(err) => {
                    return Err(err
                        .with_context(format!(
                            "Failed to create savepoint for operation {} in nested sequence",
                            i
                        ))
                        .into());
                }
            }

            // Execute the operation
            match operation().await {
                Ok(value) => {
                    // Add result to collection
                    results.push(value);

                    // Release the savepoint
                    debug!(
                        savepoint = savepoint_name,
                        operation_index = i,
                        "Releasing savepoint after successful operation"
                    );

                    match self.release_savepoint(&savepoint_name).await {
                        Ok(_) => {
                            // Remove from stack
                            if let Some(idx) = self
                                .savepoint_stack
                                .iter()
                                .position(|s| s == &savepoint_name)
                            {
                                self.savepoint_stack.remove(idx);
                            }
                        }
                        Err(err) => {
                            warn!(
                                error = %err,
                                savepoint = savepoint_name,
                                operation_index = i,
                                "Failed to release savepoint after successful operation"
                            );

                            // Remove from stack anyway
                            if let Some(idx) = self
                                .savepoint_stack
                                .iter()
                                .position(|s| s == &savepoint_name)
                            {
                                self.savepoint_stack.remove(idx);
                            }
                        }
                    }
                }
                Err(err) => {
                    info!(
                        savepoint = savepoint_name,
                        operation_index = i,
                        "Rolling back savepoint due to operation failure"
                    );

                    let rollback_result = self.rollback_to_savepoint(&savepoint_name).await;

                    // Remove from stack regardless of rollback result
                    if let Some(idx) = self
                        .savepoint_stack
                        .iter()
                        .position(|s| s == &savepoint_name)
                    {
                        self.savepoint_stack.remove(idx);
                    }

                    // If rollback failed, wrap the error
                    if let Err(rollback_err) = rollback_result {
                        error!(
                            rollback_error = %rollback_err,
                            savepoint = savepoint_name,
                            operation_index = i,
                            "Failed to rollback savepoint after operation failure"
                        );

                        return Err(DatabaseError::with_context(
                            rollback_err,
                            format!("Failed to rollback savepoint after operation {} failed", i),
                        )
                        .into());
                    }

                    // Return the original error
                    return Err(err);
                }
            }
        }

        Ok(results)
    }

    /// Execute a transaction with a specific nesting level.
    ///
    /// This is useful for when you need to have predefined nesting levels
    /// or to enforce a specific hierarchy of savepoints in your transactions.
    ///
    /// # Example
    ///
    /// ```
    /// use navius_db::{PgPool, PoolOptions, DatabaseError, DatabaseConnectionManager};
    ///
    /// async fn deep_nested_example(db: &DatabaseConnectionManager) -> Result<i32, DatabaseError> {
    ///     db.transaction(|mut tx| async move {
    ///         // Start a level 1 transaction
    ///         let result = tx.deep_nested(1, Box::new(|mut tx| async move {
    ///             // Do level 1 operations
    ///             let level1_value = 10;
    ///             
    ///             // Start a level 2 transaction
    ///             let level2_result = tx.deep_nested(2, Box::new(|mut tx| async move {
    ///                 // Do level 2 operations
    ///                 let level2_value = 20;
    ///                 
    ///                 // If this returns an error, only level 2 is rolled back
    ///                 Ok(level2_value)
    ///             })).await?;
    ///             
    ///             // Continue with level 1 operations
    ///             Ok(level1_value + level2_result)
    ///         })).await?;
    ///         
    ///         Ok(result)
    ///     }).await
    /// }
    /// ```
    ///
    #[instrument(skip(self, operation))]
    pub async fn deep_nested<T, E, F>(
        &mut self,
        level: usize,
        operation: Box<dyn FnOnce(&mut Transaction) -> F + Send + Sync>,
    ) -> Result<T, E>
    where
        T: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
        F: Future<Output = Result<T, E>> + Send,
    {
        let savepoint_name = format!(
            "deep_{}_sp_{}",
            level,
            self.savepoint_counter.fetch_add(1, Ordering::SeqCst)
        );

        // Create a savepoint with the specified name
        debug!(
            level = level,
            savepoint = savepoint_name,
            "Creating savepoint for deep nested transaction"
        );

        self.savepoint(&savepoint_name).await.map_err(|e| {
            error!(
                error = %e,
                level = level,
                savepoint = savepoint_name,
                "Failed to create savepoint for deep nested transaction"
            );
            DatabaseError::with_context(
                e,
                format!(
                    "Failed to create savepoint '{}' for deep nested transaction at level {}",
                    savepoint_name, level
                ),
            )
            .into()
        })?;

        // Track savepoint in stack
        self.savepoint_stack.push(savepoint_name.clone());

        // Execute the operation
        match operation(self).await {
            Ok(value) => {
                // Release the savepoint on success
                debug!(
                    level = level,
                    savepoint = savepoint_name,
                    "Releasing savepoint after successful deep nested transaction"
                );

                match self.release_savepoint(&savepoint_name).await {
                    Ok(_) => {
                        // Remove from stack on success
                        if let Some(idx) = self
                            .savepoint_stack
                            .iter()
                            .position(|s| s == &savepoint_name)
                        {
                            self.savepoint_stack.remove(idx);
                        }
                        Ok(value)
                    }
                    Err(err) => {
                        // Just log if release fails, operation succeeded
                        warn!(
                            error = %err,
                            savepoint = savepoint_name,
                            level = level,
                            "Failed to release savepoint after successful deep nested transaction"
                        );

                        // Remove from stack even if release failed
                        if let Some(idx) = self
                            .savepoint_stack
                            .iter()
                            .position(|s| s == &savepoint_name)
                        {
                            self.savepoint_stack.remove(idx);
                        }
                        Ok(value)
                    }
                }
            }
            Err(err) => {
                info!(
                    error = "Transaction failed",
                    savepoint = savepoint_name,
                    level = level,
                    "Rolling back deep nested transaction due to error"
                );

                // Try to rollback to the savepoint
                let rollback_result = self.rollback_to_savepoint(&savepoint_name).await;

                // Remove from stack regardless of result
                if let Some(idx) = self
                    .savepoint_stack
                    .iter()
                    .position(|s| s == &savepoint_name)
                {
                    self.savepoint_stack.remove(idx);
                }

                // If rollback failed, return a compound error
                if let Err(rollback_err) = rollback_result {
                    error!(
                        original_error = "Original error occurred",
                        rollback_error = %rollback_err,
                        savepoint = savepoint_name,
                        level = level,
                        "Failed to rollback deep nested transaction after error"
                    );

                    // Wrap the original error with the rollback error for context
                    return Err(DatabaseError::with_context(
                        rollback_err,
                        format!(
                            "Failed to rollback savepoint '{}' at level {}: {}",
                            savepoint_name, level, err
                        ),
                    )
                    .into());
                }

                // Return the original error
                Err(DatabaseError::with_context(
                    DatabaseError::TransactionError(format!(
                        "Transaction error at level {}",
                        level
                    )),
                    format!("Error in deep nested transaction at level {}", level),
                )
                .into())
            }
        }
    }
}

impl<'a> Drop for Transaction<'a> {
    fn drop(&mut self) {
        if self.tx.is_some() {
            if !self.savepoint_stack.is_empty() {
                debug!(
                    savepoints = ?self.savepoint_stack,
                    "Transaction dropped with unreleased savepoints"
                );
            }

            debug!(
                "Transaction dropped without commit or rollback, will be rolled back automatically"
            );
            // We can't run async code in Drop, but we can log a warning
            // The underlying database driver will handle the rollback in its own Drop implementation
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::DatabaseError;
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};

    // Basic mock transaction that records operations
    struct TestTx {
        savepoints: Arc<Mutex<Vec<String>>>,
        releases: Arc<Mutex<Vec<String>>>,
        rollbacks: Arc<Mutex<Vec<String>>>,
        should_fail: bool,
    }

    impl TestTx {
        fn new(should_fail: bool) -> Self {
            Self {
                savepoints: Arc::new(Mutex::new(Vec::new())),
                releases: Arc::new(Mutex::new(Vec::new())),
                rollbacks: Arc::new(Mutex::new(Vec::new())),
                should_fail,
            }
        }

        fn get_savepoints(&self) -> Vec<String> {
            self.savepoints.lock().unwrap().clone()
        }

        fn get_releases(&self) -> Vec<String> {
            self.releases.lock().unwrap().clone()
        }

        fn get_rollbacks(&self) -> Vec<String> {
            self.rollbacks.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl DatabaseTransaction for TestTx {
        async fn savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
            self.savepoints.lock().unwrap().push(name.to_string());
            Ok(())
        }

        async fn release_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
            self.releases.lock().unwrap().push(name.to_string());
            Ok(())
        }

        async fn rollback_to_savepoint(&mut self, name: &str) -> Result<(), DatabaseError> {
            self.rollbacks.lock().unwrap().push(name.to_string());
            Ok(())
        }

        async fn commit(self: Box<Self>) -> Result<(), DatabaseError> {
            if self.should_fail {
                Err(DatabaseError::TransactionError("Commit failed".to_string()))
            } else {
                Ok(())
            }
        }

        async fn rollback(self: Box<Self>) -> Result<(), DatabaseError> {
            Ok(())
        }

        async fn execute<'a>(
            &mut self,
            _query: &str,
            _params: &'a [&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
        ) -> Result<u64, DatabaseError> {
            if self.should_fail {
                Err(DatabaseError::QueryError("Execute failed".to_string()))
            } else {
                Ok(1)
            }
        }

        async fn query<'a, T>(
            &mut self,
            _query: &str,
            _params: &'a [&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
        ) -> Result<Vec<T>, DatabaseError>
        where
            T: for<'b> sqlx::FromRow<'b, sqlx::postgres::PgRow> + Send + Unpin,
        {
            if self.should_fail {
                Err(DatabaseError::QueryError("Query failed".to_string()))
            } else {
                // We can't actually return a T here, but that's ok since our tests don't use this method
                Ok(Vec::new())
            }
        }
    }

    #[tokio::test]
    async fn test_nested_transaction() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let releases = test_tx.releases.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test the nested function
        let result = tx
            .nested(Box::new(|_tx| async move { Ok::<_, DatabaseError>(42) }))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Verify savepoints and releases
        let savepoints = savepoints.lock().unwrap();
        let releases = releases.lock().unwrap();
        assert_eq!(savepoints.len(), 1);
        assert_eq!(releases.len(), 1);
        assert!(savepoints[0].starts_with("sp_"));
        assert_eq!(savepoints[0], releases[0]);
    }

    #[tokio::test]
    async fn test_nested_transaction_error() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let rollbacks = test_tx.rollbacks.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test the nested function with an error result
        let result = tx
            .nested(Box::new(|_tx| async move {
                Err::<i32, DatabaseError>(DatabaseError::QueryError("Test error".to_string()))
            }))
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DatabaseError::QueryError(msg) => {
                    assert_eq!(msg, "Test error");
                }
                _ => panic!("Expected QueryError but got: {:?}", e),
            }
        }

        // Verify savepoints and rollbacks
        let savepoints = savepoints.lock().unwrap();
        let rollbacks = rollbacks.lock().unwrap();
        assert_eq!(savepoints.len(), 1);
        assert_eq!(rollbacks.len(), 1);
        assert!(savepoints[0].starts_with("sp_"));
        assert_eq!(savepoints[0], rollbacks[0]);
    }

    // Helper function to create async blocks of the same type for success cases
    fn success_closure(
        value: i32,
    ) -> Box<
        dyn FnOnce() -> futures::future::BoxFuture<'static, Result<i32, DatabaseError>>
            + Send
            + Sync,
    > {
        Box::new(move || Box::pin(async move { Ok::<i32, DatabaseError>(value) }))
    }

    // Helper function for error cases
    fn error_closure(
        msg: String,
    ) -> Box<
        dyn FnOnce() -> futures::future::BoxFuture<'static, Result<i32, DatabaseError>>
            + Send
            + Sync,
    > {
        Box::new(move || {
            Box::pin(async move { Err::<i32, DatabaseError>(DatabaseError::QueryError(msg)) })
        })
    }

    #[tokio::test]
    async fn test_nested_sequence() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let releases = test_tx.releases.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test the nested_sequence function
        let result = tx
            .nested_sequence(vec![success_closure(10), success_closure(20)])
            .await;

        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], 10);
        assert_eq!(values[1], 20);

        // Verify savepoints and releases
        let savepoints = savepoints.lock().unwrap();
        let releases = releases.lock().unwrap();
        assert_eq!(savepoints.len(), 2);
        assert_eq!(releases.len(), 2);
    }

    #[tokio::test]
    async fn test_nested_sequence_failure() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let releases = test_tx.releases.clone();
        let rollbacks = test_tx.rollbacks.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test the nested_sequence function with the second operation failing
        let result = tx
            .nested_sequence(vec![
                success_closure(10),
                error_closure("Failed in second".to_string()),
            ])
            .await;

        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DatabaseError::QueryError(msg) => {
                    assert_eq!(msg, "Failed in second");
                }
                _ => panic!("Expected QueryError but got: {:?}", e),
            }
        }

        // Verify savepoints, releases, and rollbacks
        let savepoints = savepoints.lock().unwrap();
        let releases = releases.lock().unwrap();
        let rollbacks = rollbacks.lock().unwrap();
        assert_eq!(savepoints.len(), 2);
        assert_eq!(releases.len(), 1);
        assert_eq!(rollbacks.len(), 1);
    }

    #[tokio::test]
    async fn test_deep_nested() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let releases = test_tx.releases.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test deep_nested with 2 levels
        let result = tx
            .deep_nested(
                1,
                Box::new(|tx| async move {
                    // Level 1 operation
                    let level1 = 10;

                    // Level 2 operation
                    let level2 = tx
                        .deep_nested(2, Box::new(|_| async move { Ok::<i32, DatabaseError>(20) }))
                        .await?;

                    Ok::<i32, DatabaseError>(level1 + level2)
                }),
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 30);

        // Verify savepoints and releases
        let savepoints = savepoints.lock().unwrap();
        let releases = releases.lock().unwrap();
        assert_eq!(savepoints.len(), 2);
        assert_eq!(releases.len(), 2);
        assert!(savepoints[0].starts_with("deep_1_sp_"));
        assert!(savepoints[1].starts_with("deep_2_sp_"));
    }

    #[tokio::test]
    async fn test_deep_nested_inner_failure() {
        let test_tx = TestTx::new(false);
        let savepoints = test_tx.savepoints.clone();
        let releases = test_tx.releases.clone();
        let rollbacks = test_tx.rollbacks.clone();

        let mut tx = Transaction::new(Box::new(test_tx));

        // Test deep_nested with inner failure
        let result = tx
            .deep_nested(
                1,
                Box::new(|tx| async move {
                    // Level 1 operation
                    let level1 = 10;

                    // Level 2 operation - will fail
                    let level2 = tx
                        .deep_nested(
                            2,
                            Box::new(|_| async move {
                                Err::<i32, DatabaseError>(DatabaseError::QueryError(
                                    "Level 2 failed".to_string(),
                                ))
                            }),
                        )
                        .await?;

                    Ok::<i32, DatabaseError>(level1 + level2)
                }),
            )
            .await;

        assert!(result.is_err());
        // Verify the error is from level 2
        if let Err(e) = result {
            assert!(e.to_string().contains("Level 2 failed") || e.to_string().contains("level 2"));
        }

        // Verify savepoints, releases, and rollbacks
        let savepoints = savepoints.lock().unwrap();
        let releases = releases.lock().unwrap();
        let rollbacks = rollbacks.lock().unwrap();
        assert_eq!(savepoints.len(), 2);
        assert_eq!(releases.len(), 0);
        assert_eq!(rollbacks.len(), 2);
        assert!(savepoints[0].starts_with("deep_1_sp_"));
        assert!(savepoints[1].starts_with("deep_2_sp_"));
    }
}
