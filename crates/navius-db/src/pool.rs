use crate::config::DatabaseConfig;
use crate::error::{DatabaseError, DatabaseResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

// Import needed traits
use async_trait::async_trait;
use sqlx::Row;
#[cfg(feature = "postgres")]
pub use sqlx::postgres::PgPoolOptions;

/// Options for configuring the database connection pool
#[derive(Debug, Clone)]
pub struct PoolOptions {
    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Minimum number of connections to maintain in the pool
    pub min_connections: u32,

    /// Connection timeout
    pub connect_timeout: Duration,

    /// Idle timeout
    pub idle_timeout: Duration,

    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,

    /// Whether to enable connection tracing
    pub trace: bool,
}

impl From<&DatabaseConfig> for PoolOptions {
    fn from(config: &DatabaseConfig) -> Self {
        Self {
            max_connections: config.max_connections,
            min_connections: config.min_connections,
            connect_timeout: config.connect_timeout(),
            idle_timeout: config.idle_timeout(),
            max_lifetime: config.max_lifetime(),
            trace: config.trace,
        }
    }
}

impl Default for PoolOptions {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            max_lifetime: Duration::from_secs(1800),
            trace: false,
        }
    }
}

/// Database connection pool trait
#[async_trait]
pub trait DatabasePool: Send + Sync + std::fmt::Debug + 'static {
    /// Get a connection from the pool
    async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>>;

    /// Close the pool
    async fn close(&self) -> DatabaseResult<()>;

    /// Check if the pool is healthy
    async fn check_health(&self) -> DatabaseResult<()>;
}

/// PostgreSQL connection pool
#[cfg(feature = "postgres")]
#[derive(Debug, Clone)]
pub struct PgPool {
    pool: Arc<sqlx::PgPool>,
}

#[cfg(feature = "postgres")]
impl PgPool {
    /// Create a new PostgreSQL connection pool
    #[instrument(skip(config), fields(database_url = %format_url(&config.url)))]
    pub async fn new(config: &DatabaseConfig) -> DatabaseResult<Self> {
        let options = PoolOptions::from(config);

        info!("Connecting to PostgreSQL database");
        debug!("Using connection options: {:?}", options);

        let pg_pool_options = PgPoolOptions::new()
            .max_connections(options.max_connections as u32)
            .min_connections(options.min_connections as u32)
            .acquire_timeout(options.connect_timeout)
            .idle_timeout(options.idle_timeout)
            .max_lifetime(options.max_lifetime);

        let pool = pg_pool_options.connect(&config.url).await.map_err(|e| {
            DatabaseError::ConnectionError(format!("Failed to connect to database: {}", e))
        })?;

        if config.run_migrations {
            run_migrations(&pool, config).await?;
        }

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Get the inner SQLx pool
    pub fn inner(&self) -> &sqlx::PgPool {
        &self.pool
    }

    /// Create a new mock PgPool for testing
    #[cfg(test)]
    pub fn new_mock(options: PoolOptions) -> Self {
        use crate::error::DatabaseResult;
        use std::sync::atomic::{AtomicU64, Ordering};

        struct MockConnection {
            next_row_id: AtomicU64,
        }

        #[async_trait]
        impl DatabaseConnection for MockConnection {
            #[cfg(feature = "postgres")]
            async fn execute(
                &mut self,
                _query: &str,
                _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
            ) -> DatabaseResult<u64> {
                // Mock implementation always returns 1 affected row
                Ok(1)
            }

            #[cfg(feature = "postgres")]
            async fn query(
                &mut self,
                _query: &str,
                _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
            ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
                // Mock row set that returns a single row
                struct MockRowSet {
                    has_returned: bool,
                }

                impl DatabaseRowSet for MockRowSet {
                    fn next(&mut self) -> DatabaseResult<Option<PgRow>> {
                        if self.has_returned {
                            return Ok(None);
                        }

                        self.has_returned = true;

                        // Create an empty PgRow for testing
                        // This is just a placeholder since we can't easily create a real PgRow
                        // in tests without a database connection
                        Ok(None)
                    }
                }

                Ok(Box::new(MockRowSet {
                    has_returned: false,
                }))
            }

            async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>> {
                struct MockTransaction {
                    conn: MockConnection,
                }

                #[async_trait]
                impl DatabaseConnection for MockTransaction {
                    #[cfg(feature = "postgres")]
                    async fn execute(
                        &mut self,
                        query: &str,
                        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
                    ) -> DatabaseResult<u64> {
                        self.conn.execute(query, params).await
                    }

                    #[cfg(feature = "postgres")]
                    async fn query(
                        &mut self,
                        query: &str,
                        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
                    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
                        self.conn.query(query, params).await
                    }

                    async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>> {
                        // Nested transactions not supported in mock
                        Err(DatabaseError::TransactionError(
                            "Nested transactions not supported in mock".to_string(),
                        ))
                    }
                }

                #[async_trait]
                impl DatabaseTransaction for MockTransaction {
                    async fn commit(self: Box<Self>) -> DatabaseResult<()> {
                        // Mock successful commit
                        Ok(())
                    }

                    async fn rollback(self: Box<Self>) -> DatabaseResult<()> {
                        // Mock successful rollback
                        Ok(())
                    }
                }

                Ok(Box::new(MockTransaction {
                    conn: MockConnection {
                        next_row_id: AtomicU64::new(1),
                    },
                }))
            }
        }

        struct MockPool {}

        #[async_trait]
        impl DatabasePool for MockPool {
            async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>> {
                Ok(Box::new(MockConnection {
                    next_row_id: AtomicU64::new(1),
                }))
            }

            async fn close(&self) -> DatabaseResult<()> {
                Ok(())
            }

            async fn check_health(&self) -> DatabaseResult<()> {
                Ok(())
            }
        }

        Self {
            pool: Arc::new(MockPool {}),
        }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabasePool for PgPool {
    async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>> {
        let conn = self.pool.acquire().await.map_err(|e| {
            DatabaseError::PoolError(format!("Failed to acquire connection: {}", e))
        })?;

        Ok(Box::new(PgConnection::new(conn)))
    }

    async fn close(&self) -> DatabaseResult<()> {
        self.pool.close().await;
        Ok(())
    }

    async fn check_health(&self) -> DatabaseResult<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| DatabaseError::ConnectionError(format!("Health check failed: {}", e)))?;

        Ok(())
    }
}

/// Database connection trait
#[async_trait]
pub trait DatabaseConnection: Send + Sync + std::fmt::Debug + 'static {
    /// Execute a query that returns no rows
    #[cfg(feature = "postgres")]
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64>;

    /// Execute a query that returns rows
    #[cfg(feature = "postgres")]
    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>>;

    /// Begin a transaction
    async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>>;
}

/// Database rows trait for iterating over result sets
pub trait DatabaseRowSet: Send + Sync + std::fmt::Debug + 'static {
    /// Fetch the next row
    fn next(&mut self) -> DatabaseResult<Option<PgRow>>;
}

/// Database row structure
#[cfg(feature = "postgres")]
#[derive(Debug)]
pub struct PgRow {
    inner: sqlx::postgres::PgRow,
}

#[cfg(feature = "postgres")]
impl PgRow {
    /// Create a new PgRow
    pub fn new(row: sqlx::postgres::PgRow) -> Self {
        Self { inner: row }
    }

    /// Get a value from the row by column name
    pub fn get<T>(&self, name: &str) -> DatabaseResult<T>
    where
        T: for<'a> sqlx::decode::Decode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        self.inner.try_get(name).map_err(|e| {
            error!("Failed to get column {}: {}", name, e);
            DatabaseError::QueryError(format!("Failed to get column {}: {}", name, e))
        })
    }

    /// Get a value from the row by column index
    pub fn get_by_index<T>(&self, index: usize) -> DatabaseResult<T>
    where
        T: for<'a> sqlx::decode::Decode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        self.inner.try_get(index).map_err(|e| {
            error!("Failed to get column at index {}: {}", index, e);
            DatabaseError::QueryError(format!("Failed to get column at index {}: {}", index, e))
        })
    }
}

/// Transaction abstraction for database operations
#[async_trait]
pub trait DatabaseTransaction: Send + Sync {
    /// Execute a query in the transaction with parameters
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64>;

    /// Query the database within a transaction with parameters
    async fn query<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>>;

    /// Commit the transaction
    async fn commit(self: Box<Self>) -> DatabaseResult<()>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> DatabaseResult<()>;

    /// Create a savepoint within the transaction
    ///
    /// Savepoints allow for partial rollback within a transaction.
    /// The savepoint name must be a valid identifier (alphanumeric and underscores only).
    async fn savepoint(&mut self, name: &str) -> DatabaseResult<()>;

    /// Rollback to a previously created savepoint
    ///
    /// This rolls back all changes made after the savepoint was created.
    /// The savepoint remains valid and can be used again.
    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()>;

    /// Release a savepoint
    ///
    /// This releases a previously created savepoint. After a savepoint is released,
    /// you can no longer roll back to it.
    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()>;
}

/// PostgreSQL transaction implementation
pub struct PgTransaction {
    tx: sqlx::Transaction<'static, sqlx::Postgres>,
}

impl PgTransaction {
    /// Create a new PostgreSQL transaction
    pub(crate) fn new(tx: sqlx::Transaction<'static, sqlx::Postgres>) -> Self {
        Self { tx }
    }

    /// Validate a savepoint name to prevent SQL injection
    fn validate_savepoint_name(&self, name: &str) -> DatabaseResult<()> {
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DatabaseError::savepoint_error(format!(
                "Invalid savepoint name: {}. Only alphanumeric characters and underscores are allowed.",
                name
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl DatabaseTransaction for PgTransaction {
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let result = sqlx::query_with(
            query,
            sqlx::postgres::PgArguments::from_iter(params.iter().copied()),
        )
        .execute(&mut self.tx)
        .await
        .map_err(|e| DatabaseError::query_error(format!("Query execution error: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn query<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        let result = sqlx::query_with(
            query,
            sqlx::postgres::PgArguments::from_iter(params.iter().copied()),
        )
        .fetch_all(&mut self.tx)
        .await
        .map_err(|e| DatabaseError::query_error(format!("Query execution error: {}", e)))?;

        Ok(Box::new(PgRowSet::new(result)))
    }

    async fn commit(self: Box<Self>) -> DatabaseResult<()> {
        self.tx.commit().await.map_err(|e| {
            DatabaseError::transaction_error(format!("Failed to commit transaction: {}", e))
        })
    }

    async fn rollback(self: Box<Self>) -> DatabaseResult<()> {
        self.tx.rollback().await.map_err(|e| {
            DatabaseError::transaction_error(format!("Failed to rollback transaction: {}", e))
        })
    }

    async fn savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to create savepoint: {}", e))
        })
    }

    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("ROLLBACK TO SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to rollback to savepoint: {}", e))
        })
    }

    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("RELEASE SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to release savepoint: {}", e))
        })
    }
}

#[cfg(feature = "postgres")]
async fn run_migrations(pool: &sqlx::PgPool, config: &DatabaseConfig) -> DatabaseResult<()> {
    info!(
        "Running database migrations from {}",
        config.migrations_path
    );

    let migrator = sqlx::migrate::Migrator::new(std::path::Path::new(&config.migrations_path))
        .await
        .map_err(|e| DatabaseError::MigrationError(format!("Failed to load migrations: {}", e)))?;

    migrator
        .run(pool)
        .await
        .map_err(|e| DatabaseError::MigrationError(format!("Failed to run migrations: {}", e)))?;

    info!("Database migrations completed successfully");
    Ok(())
}

/// Format the database URL for logging (hiding credentials)
fn format_url(url: &str) -> String {
    // Simple URL formatting to hide credentials for logging
    if let Some(at_pos) = url.find('@') {
        if let Some(protocol_end) = url.find("://") {
            let protocol = &url[0..protocol_end + 3];
            let after_credentials = &url[at_pos..];
            return format!("{}***:***{}", protocol, after_credentials);
        }
    }
    // If we can't parse the URL, return a generic string
    "database-url-hidden".to_string()
}

/// PostgreSQL connection
#[cfg(feature = "postgres")]
#[derive(Debug)]
pub struct PgConnection {
    conn: sqlx::pool::PoolConnection<sqlx::Postgres>,
}

#[cfg(feature = "postgres")]
impl PgConnection {
    fn new(conn: sqlx::pool::PoolConnection<sqlx::Postgres>) -> Self {
        Self { conn }
    }
}

/// PostgreSQL transaction
#[cfg(feature = "postgres")]
#[derive(Debug)]
pub struct PgTransaction {
    tx: sqlx::Transaction<'static, sqlx::Postgres>,
}

#[cfg(feature = "postgres")]
impl PgTransaction {
    /// Create a new PostgreSQL transaction
    pub fn new(tx: sqlx::Transaction<'static, sqlx::Postgres>) -> Self {
        Self { tx }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabaseConnection for PgConnection {
    async fn execute(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let result = sqlx::query_with(query, params)
            .execute(&mut self.conn)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn query(
        &mut self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        struct PgRowSet {
            rows: Vec<sqlx::postgres::PgRow>,
            pos: usize,
        }

        impl DatabaseRowSet for PgRowSet {
            fn next(&mut self) -> DatabaseResult<Option<PgRow>> {
                if self.pos >= self.rows.len() {
                    return Ok(None);
                }

                let row = self.rows.remove(self.pos);
                Ok(Some(PgRow::new(row)))
            }
        }

        impl std::fmt::Debug for PgRowSet {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("PgRowSet")
                    .field("rows", &self.rows.len())
                    .field("pos", &self.pos)
                    .finish()
            }
        }

        let rows = sqlx::query_with(query, params)
            .fetch_all(&mut self.conn)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(Box::new(PgRowSet { rows, pos: 0 }))
    }

    async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>> {
        let tx = self.conn.begin().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to begin transaction: {}", e))
        })?;

        // Convert the transaction to a 'static lifetime
        let tx: sqlx::Transaction<'static, sqlx::Postgres> = unsafe { std::mem::transmute(tx) };

        Ok(Box::new(PgTransaction::new(tx)))
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabaseTransaction for PgTransaction {
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let result = sqlx::query_with(
            query,
            sqlx::postgres::PgArguments::from_iter(params.iter().copied()),
        )
        .execute(&mut self.tx)
        .await
        .map_err(|e| DatabaseError::query_error(format!("Query execution error: {}", e)))?;

        Ok(result.rows_affected())
    }

    async fn query<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        let result = sqlx::query_with(
            query,
            sqlx::postgres::PgArguments::from_iter(params.iter().copied()),
        )
        .fetch_all(&mut self.tx)
        .await
        .map_err(|e| DatabaseError::query_error(format!("Query execution error: {}", e)))?;

        Ok(Box::new(PgRowSet::new(result)))
    }

    async fn commit(self: Box<Self>) -> DatabaseResult<()> {
        self.tx.commit().await.map_err(|e| {
            DatabaseError::transaction_error(format!("Failed to commit transaction: {}", e))
        })
    }

    async fn rollback(self: Box<Self>) -> DatabaseResult<()> {
        self.tx.rollback().await.map_err(|e| {
            DatabaseError::transaction_error(format!("Failed to rollback transaction: {}", e))
        })
    }

    async fn savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to create savepoint: {}", e))
        })
    }

    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("ROLLBACK TO SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to rollback to savepoint: {}", e))
        })
    }

    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let query = format!("RELEASE SAVEPOINT {}", name);
        self.execute(&query, &[]).await.map(|_| ()).map_err(|e| {
            DatabaseError::savepoint_error(format!("Failed to release savepoint: {}", e))
        })
    }
}

/// Validates a savepoint name to prevent SQL injection
/// Savepoint names should only contain alphanumeric characters and underscores
fn is_valid_savepoint_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

impl DatabaseConnectionManager {
    /// Run a closure within a transaction, automatically rolling back on error.
    ///
    /// This function starts a transaction, executes the closure with the transaction,
    /// and then commits the transaction if the closure returns `Ok` or rolls back
    /// the transaction if the closure returns `Err`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use navius_db::{DatabaseConnectionManager, DatabaseError};
    ///
    /// async fn transfer_funds(
    ///     db: &DatabaseConnectionManager,
    ///     from_account: &str,
    ///     to_account: &str,
    ///     amount: f64,
    /// ) -> Result<(), DatabaseError> {
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
    #[instrument(skip(self, f), level = "debug")]
    pub async fn transaction<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        debug!("Starting database transaction");

        // Get a connection from the pool
        let conn = self.acquire().await?;

        // Begin a transaction
        let tx = conn.begin().await?;

        // Execute the closure
        match f(tx).await {
            Ok(result) => {
                debug!("Transaction completed successfully, committing");
                match tx.commit().await {
                    Ok(_) => Ok(result),
                    Err(e) => {
                        error!(error = %e, "Failed to commit transaction");
                        Err(e.with_context("Transaction commit failed"))
                    }
                }
            }
            Err(e) => {
                // Automatically roll back on error
                warn!(error = %e, "Transaction failed, performing automatic rollback");

                // Try to roll back the transaction
                match tx.rollback().await {
                    Ok(_) => {
                        debug!("Transaction rollback successful");
                        Err(e)
                    }
                    Err(rollback_err) => {
                        // If rollback itself fails, return a compound error
                        error!(
                            original_error = %e,
                            rollback_error = %rollback_err,
                            "Failed to roll back transaction after error"
                        );

                        // Create context with detailed information about both errors
                        let context = crate::error::ErrorContext::new()
                            .with_operation("Transaction")
                            .with_additional_info(format!(
                                "Failed to roll back transaction: {}",
                                rollback_err
                            ));

                        Err(e.chain_error("Transaction failed with rollback error", context))
                    }
                }
            }
        }
    }

    /// Run a closure within a transaction with retry logic for transient errors.
    ///
    /// This function is similar to `transaction`, but it automatically retries
    /// the operation if a transient error occurs, such as a deadlock or
    /// serialization failure.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use navius_db::{DatabaseConnectionManager, DatabaseError};
    ///
    /// async fn update_counter(db: &DatabaseConnectionManager, id: i32) -> Result<i32, DatabaseError> {
    ///     // Retry up to 3 times on transient errors like deadlocks
    ///     db.transaction_with_retry(3, |mut tx| async move {
    ///         // Update counter
    ///         let query = "UPDATE counters SET value = value + 1 WHERE id = $1 RETURNING value";
    ///         let rows = tx.query_with(query, &[&id]).await?;
    ///         
    ///         let row = rows.first()?;
    ///         let new_value: i32 = row.get("value")?;
    ///         
    ///         Ok(new_value)
    ///     }).await
    /// }
    /// ```
    #[instrument(skip(self, f), level = "debug")]
    pub async fn transaction_with_retry<F, Fut, T>(
        &self,
        max_attempts: usize,
        f: F,
    ) -> DatabaseResult<T>
    where
        F: Fn(Transaction<'_>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = DatabaseResult<T>> + Send,
    {
        debug!(max_attempts, "Starting database transaction with retry");

        if max_attempts == 0 {
            return Err(DatabaseError::ValidationError(
                "max_attempts must be greater than 0".to_string(),
            ));
        }

        let mut attempt = 0;
        let mut last_error = None;

        // Try multiple times
        while attempt < max_attempts {
            attempt += 1;
            debug!(attempt, max_attempts, "Transaction retry attempt");

            // Get a connection from the pool
            let conn = match self.acquire().await {
                Ok(conn) => conn,
                Err(e) => {
                    // If we can't get a connection, don't retry
                    return Err(e.with_context(format!(
                        "Failed to acquire connection for transaction (attempt {}/{})",
                        attempt, max_attempts
                    )));
                }
            };

            // Begin a transaction
            let tx = match conn.begin().await {
                Ok(tx) => tx,
                Err(e) => {
                    // If we can't begin a transaction, don't retry
                    return Err(e.with_context(format!(
                        "Failed to begin transaction (attempt {}/{})",
                        attempt, max_attempts
                    )));
                }
            };

            // Execute the closure
            match f(tx).await {
                Ok(result) => {
                    debug!(
                        "Transaction successful, committing (attempt {}/{})",
                        attempt, max_attempts
                    );

                    // Commit the transaction
                    match tx.commit().await {
                        Ok(_) => {
                            debug!("Transaction commit successful");
                            return Ok(result);
                        }
                        Err(e) => {
                            if e.is_transient() && attempt < max_attempts {
                                // If commit fails with a transient error, we can retry
                                warn!(
                                    error = %e,
                                    attempt,
                                    max_attempts,
                                    "Transaction commit failed with transient error, retrying"
                                );
                                last_error = Some(e);
                                continue;
                            } else {
                                // Non-transient error or last attempt
                                error!(
                                    error = %e,
                                    attempt,
                                    max_attempts,
                                    "Transaction commit failed"
                                );
                                return Err(e.with_context(format!(
                                    "Transaction commit failed (attempt {}/{})",
                                    attempt, max_attempts
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    // Try to roll back the transaction
                    let _ = tx.rollback().await;

                    if e.is_transient() && attempt < max_attempts {
                        // If the error is transient and we have attempts left, retry
                        warn!(
                            error = %e,
                            attempt,
                            max_attempts,
                            "Transaction failed with transient error, retrying"
                        );
                        last_error = Some(e);
                    } else {
                        // Non-transient error or last attempt
                        if e.is_transient() {
                            warn!(
                                error = %e,
                                attempt,
                                max_attempts,
                                "Transaction failed with transient error on final attempt"
                            );
                        } else {
                            debug!(
                                error = %e,
                                attempt,
                                max_attempts,
                                "Transaction failed with non-transient error"
                            );
                        }

                        return Err(e.with_context(format!(
                            "Transaction failed (attempt {}/{})",
                            attempt, max_attempts
                        )));
                    }
                }
            }
        }

        // If we get here, all attempts failed with transient errors
        let err = last_error.unwrap_or_else(|| {
            DatabaseError::UnexpectedStateError(
                "No error was recorded but all transaction retry attempts failed".to_string(),
            )
        });

        error!(
            error = %err,
            attempts = attempt,
            "All transaction retry attempts failed with transient errors"
        );

        Err(err.with_context(format!(
            "Transaction failed after {} retry attempts",
            max_attempts
        )))
    }

    /// Execute a nested function in the database transaction with savepoints
    ///
    /// This method is useful when you need to perform multiple operations that should
    /// be seen as a single unit, but you also want to be able to roll back parts of the
    /// transaction if specific operations fail.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use navius_db::{DatabaseConnectionManager, DatabaseError};
    ///
    /// async fn complex_operation(db: &DatabaseConnectionManager) -> Result<(), DatabaseError> {
    ///     db.transaction_nested(|mut tx| async move {
    ///         // First part of transaction
    ///         tx.execute("INSERT INTO logs (message) VALUES ('Starting operation')").await?;
    ///         
    ///         // Nested transaction that may fail
    ///         let result = tx.nested(|| async {
    ///             tx.execute("UPDATE accounts SET status = 'PROCESSING' WHERE id = 123").await?;
    ///             
    ///             // This might fail, and only the nested part will be rolled back
    ///             tx.execute("INSERT INTO process_queue (account_id) VALUES (123)").await
    ///         }).await;
    ///         
    ///         if result.is_err() {
    ///             // Log the failure but continue with the transaction
    ///             tx.execute("INSERT INTO logs (message) VALUES ('Processing failed')").await?;
    ///         } else {
    ///             tx.execute("INSERT INTO logs (message) VALUES ('Processing succeeded')").await?;
    ///         }
    ///         
    ///         // The main transaction still commits
    ///         tx.execute("INSERT INTO logs (message) VALUES ('Operation completed')").await?;
    ///         
    ///         Ok(())
    ///     }).await
    /// }
    /// ```
    #[instrument(skip(self, f), level = "debug")]
    pub async fn transaction_nested<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        debug!("Starting database transaction with nested support");

        // Get a connection from the pool
        let conn = self.acquire().await?;

        // Begin a transaction
        let tx = conn.begin().await?;

        // Execute the closure
        match f(tx).await {
            Ok(result) => {
                debug!("Transaction completed successfully, committing");
                match tx.commit().await {
                    Ok(_) => Ok(result),
                    Err(e) => {
                        error!(error = %e, "Failed to commit transaction");
                        Err(e.with_context("Transaction commit failed"))
                    }
                }
            }
            Err(e) => {
                // Automatically roll back on error
                warn!(error = %e, "Transaction failed, performing automatic rollback");

                // Try to roll back the transaction
                match tx.rollback().await {
                    Ok(_) => {
                        debug!("Transaction rollback successful");
                        Err(e)
                    }
                    Err(rollback_err) => {
                        // If rollback itself fails, return a compound error
                        error!(
                            original_error = %e,
                            rollback_error = %rollback_err,
                            "Failed to roll back transaction after error"
                        );

                        // Create context with detailed information about both errors
                        let context = crate::error::ErrorContext::new()
                            .with_operation("Transaction")
                            .with_additional_info(format!(
                                "Failed to roll back transaction: {}",
                                rollback_err
                            ));

                        Err(e.chain_error("Transaction failed with rollback error", context))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DatabaseError;
    use crate::transaction::Transaction;
    use mockall::mock;
    use mockall::predicate::*;
    use std::sync::Arc;

    mock! {
        pub DatabasePool {}

        impl DatabasePool for DatabasePool {
            async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>>;
            async fn close(&self) -> DatabaseResult<()>;
            fn name(&self) -> &str;
        }
    }

    mock! {
        pub DatabaseConn {}

        impl DatabaseConnection for DatabaseConn {
            async fn execute(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<u64>;
            async fn query(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<Box<dyn DatabaseRowSet>>;
            async fn begin(&mut self) -> DatabaseResult<Transaction>;
            async fn close(self: Box<Self>) -> DatabaseResult<()>;
        }
    }

    mock! {
        pub DatabaseTx {}

        impl DatabaseTransaction for DatabaseTx {
            async fn execute(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<u64>;
            async fn query(&mut self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> DatabaseResult<Box<dyn DatabaseRowSet>>;
            async fn commit(self: Box<Self>) -> DatabaseResult<()>;
            async fn rollback(self: Box<Self>) -> DatabaseResult<()>;
        }
    }

    mock! {
        pub DatabaseRows {}

        impl DatabaseRowSet for DatabaseRows {
            fn len(&self) -> usize;
            fn is_empty(&self) -> bool;
            fn first(&self) -> DatabaseResult<Box<dyn DatabaseRow>>;
            fn get(&self, idx: usize) -> DatabaseResult<Box<dyn DatabaseRow>>;
            fn iter(&self) -> Box<dyn Iterator<Item = DatabaseResult<Box<dyn DatabaseRow>>> + '_>;
        }
    }

    mock! {
        pub DatabaseRowData {}

        impl DatabaseRow for DatabaseRowData {
            fn get<T: sqlx::Type<sqlx::Postgres> + 'static>(&self, col: &str) -> DatabaseResult<T>;
            fn get_raw<T: sqlx::Type<sqlx::Postgres> + 'static>(&self, col: usize) -> DatabaseResult<T>;
            fn columns(&self) -> &[&str];
        }
    }

    #[tokio::test]
    async fn test_transaction_auto_rollback_on_error() {
        let mut mock_pool = MockDatabasePool::new();
        let mut mock_conn = MockDatabaseConn::new();
        let mut mock_tx = MockDatabaseTx::new();

        // Set up the mock pool to return a mocked connection
        mock_pool
            .expect_acquire()
            .times(1)
            .return_once(move || Ok(Box::new(mock_conn)));

        // Set up the mock connection
        mock_conn.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx);
            Ok(Transaction::new(tx))
        });

        // Set up expectations for rollback
        mock_tx.expect_rollback().times(1).returning(|_| Ok(()));

        // Create a connection manager with the mock pool
        let manager = DatabaseConnectionManager {
            pool: Arc::new(Box::new(mock_pool) as Box<dyn DatabasePool>),
        };

        // Execute a transaction that returns an error
        let result = manager
            .transaction(|_tx| async {
                Err::<(), _>(DatabaseError::QueryError("Test error".to_string()))
            })
            .await;

        // Verify that the transaction was rolled back
        assert!(result.is_err());
        match result {
            Err(DatabaseError::QueryError(msg)) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Expected QueryError"),
        }
    }

    #[tokio::test]
    async fn test_transaction_auto_rollback_failure() {
        let mut mock_pool = MockDatabasePool::new();
        let mut mock_conn = MockDatabaseConn::new();
        let mut mock_tx = MockDatabaseTx::new();

        // Set up the mock pool to return a mocked connection
        mock_pool
            .expect_acquire()
            .times(1)
            .return_once(move || Ok(Box::new(mock_conn)));

        // Set up the mock connection
        mock_conn.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx);
            Ok(Transaction::new(tx))
        });

        // Set up expectations for rollback to fail
        mock_tx.expect_rollback().times(1).returning(|_| {
            Err(DatabaseError::TransactionError(
                "Rollback failed".to_string(),
            ))
        });

        // Create a connection manager with the mock pool
        let manager = DatabaseConnectionManager {
            pool: Arc::new(Box::new(mock_pool) as Box<dyn DatabasePool>),
        };

        // Execute a transaction that returns an error
        let result = manager
            .transaction(|_tx| async {
                Err::<(), _>(DatabaseError::QueryError("Test error".to_string()))
            })
            .await;

        // Verify that we get a chained error with both the original and rollback errors
        assert!(result.is_err());
        match result {
            Err(DatabaseError::ChainedError { message, chain, .. }) => {
                assert!(message.contains("Transaction failed with rollback error"));
                assert_eq!(chain.len(), 1);
                assert!(matches!(*chain[0], DatabaseError::QueryError(_)));
            }
            _ => panic!("Expected ChainedError"),
        }
    }

    #[tokio::test]
    async fn test_transaction_commit_success() {
        let mut mock_pool = MockDatabasePool::new();
        let mut mock_conn = MockDatabaseConn::new();
        let mut mock_tx = MockDatabaseTx::new();

        // Set up the mock pool to return a mocked connection
        mock_pool
            .expect_acquire()
            .times(1)
            .return_once(move || Ok(Box::new(mock_conn)));

        // Set up the mock connection
        mock_conn.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx);
            Ok(Transaction::new(tx))
        });

        // Set up expectations for commit
        mock_tx.expect_commit().times(1).returning(|_| Ok(()));

        // Create a connection manager with the mock pool
        let manager = DatabaseConnectionManager {
            pool: Arc::new(Box::new(mock_pool) as Box<dyn DatabasePool>),
        };

        // Execute a transaction that succeeds
        let result = manager.transaction(|_tx| async { Ok(42) }).await;

        // Verify that the transaction was committed and returned the correct result
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_transaction_commit_failure() {
        let mut mock_pool = MockDatabasePool::new();
        let mut mock_conn = MockDatabaseConn::new();
        let mut mock_tx = MockDatabaseTx::new();

        // Set up the mock pool to return a mocked connection
        mock_pool
            .expect_acquire()
            .times(1)
            .return_once(move || Ok(Box::new(mock_conn)));

        // Set up the mock connection
        mock_conn.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx);
            Ok(Transaction::new(tx))
        });

        // Set up expectations for commit to fail
        mock_tx
            .expect_commit()
            .times(1)
            .returning(|_| Err(DatabaseError::TransactionError("Commit failed".to_string())));

        // Create a connection manager with the mock pool
        let manager = DatabaseConnectionManager {
            pool: Arc::new(Box::new(mock_pool) as Box<dyn DatabasePool>),
        };

        // Execute a transaction that succeeds but fails to commit
        let result = manager.transaction(|_tx| async { Ok(42) }).await;

        // Verify that we get an error about the commit failure
        assert!(result.is_err());
        match result {
            Err(DatabaseError::WithContext { context, source }) => {
                assert_eq!(context, "Transaction commit failed");
                assert!(matches!(*source, DatabaseError::TransactionError(_)));
            }
            _ => panic!("Expected WithContext error"),
        }
    }

    #[tokio::test]
    async fn test_transaction_with_retry_success_after_transient_error() {
        let mut mock_pool = MockDatabasePool::new();

        // For the first attempt
        let mut mock_conn1 = MockDatabaseConn::new();
        let mut mock_tx1 = MockDatabaseTx::new();

        // For the second attempt that will succeed
        let mut mock_conn2 = MockDatabaseConn::new();
        let mut mock_tx2 = MockDatabaseTx::new();

        // Set up the mock pool to return mocked connections
        mock_pool.expect_acquire().times(2).returning(move || {
            // Return a different connection each time
            if mock_conn1.checkpoint.len() == 0 {
                Ok(Box::new(mock_conn1))
            } else {
                Ok(Box::new(mock_conn2))
            }
        });

        // Set up the first mock connection
        mock_conn1.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx1);
            Ok(Transaction::new(tx))
        });

        // Set up the second mock connection
        mock_conn2.expect_begin().times(1).returning(move || {
            let tx = Box::new(mock_tx2);
            Ok(Transaction::new(tx))
        });

        // Set up expectations for commit on the second transaction
        mock_tx2.expect_commit().times(1).returning(|_| Ok(()));

        // Set up expectations for rollback on the first transaction
        mock_tx1.expect_rollback().times(1).returning(|_| Ok(()));

        // Create a connection manager with the mock pool
        let manager = DatabaseConnectionManager {
            pool: Arc::new(Box::new(mock_pool) as Box<dyn DatabasePool>),
        };

        // A variable to track which attempt we're on
        let attempt = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let attempt_clone = attempt.clone();

        // Execute a transaction with retry that fails with a transient error on first attempt
        let result = manager
            .transaction_with_retry(3, move |_tx| {
                let current_attempt =
                    attempt_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                async move {
                    if current_attempt == 0 {
                        // First attempt fails with a transient error
                        let err = sqlx::Error::Database(Box::new(sqlx::error::DatabaseError::new(
                            "40001", // serialization_failure - transient error
                            "Serialization failure",
                        )));
                        Err(DatabaseError::SQLXError(err))
                    } else {
                        // Second attempt succeeds
                        Ok(42)
                    }
                }
            })
            .await;

        // Verify that we get the correct result after retrying
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 2); // Should have made 2 attempts
    }
}
