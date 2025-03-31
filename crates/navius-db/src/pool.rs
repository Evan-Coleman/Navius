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
                    async fn execute<'a>(
                        &mut self,
                        query: &str,
                        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
                    ) -> DatabaseResult<u64> {
                        self.conn.execute(query, params).await
                    }

                    async fn query<'a>(
                        &mut self,
                        query: &str,
                        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
                    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
                        self.conn.query(query, params).await
                    }

                    async fn commit(self: Box<Self>) -> DatabaseResult<()> {
                        Ok(())
                    }

                    async fn rollback(self: Box<Self>) -> DatabaseResult<()> {
                        Ok(())
                    }

                    async fn savepoint(&mut self, name: &str) -> DatabaseResult<()> {
                        Ok(())
                    }

                    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
                        Ok(())
                    }

                    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
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
    fn next(&mut self) -> Option<Box<dyn DatabaseRow>>;
}

/// A PostgreSQL row set
pub struct PgRowSet {
    rows: Vec<sqlx::postgres::PgRow>,
    current_index: usize,
}

impl PgRowSet {
    fn new(rows: Vec<sqlx::postgres::PgRow>) -> Self {
        Self {
            rows,
            current_index: 0,
        }
    }
}

impl DatabaseRowSet for PgRowSet {
    fn next(&mut self) -> Option<Box<dyn DatabaseRow>> {
        if self.current_index < self.rows.len() {
            let row = self.rows[self.current_index].clone();
            self.current_index += 1;
            Some(Box::new(PgRow::new(row)))
        } else {
            None
        }
    }
}

/// A PostgreSQL row
pub struct PgRow {
    row: sqlx::postgres::PgRow,
}

impl PgRow {
    fn new(row: sqlx::postgres::PgRow) -> Self {
        Self { row }
    }
}

impl DatabaseRow for PgRow {
    fn get_column_value(&self, column: &str) -> DatabaseResult<Option<serde_json::Value>> {
        match self.row.try_get::<Option<serde_json::Value>, _>(column) {
            Ok(value) => Ok(value),
            Err(e) => Err(DatabaseError::query_error(format!(
                "Failed to get column value: {}",
                e
            ))),
        }
    }

    fn get_column_names(&self) -> Vec<String> {
        self.row
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect()
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

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabaseTransaction for PgTransaction {
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let mut query_builder = sqlx::query(query);
        for param in params.iter() {
            query_builder = query_builder.bind(param);
        }
        let result = query_builder
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
        let mut query_builder = sqlx::query(query);
        for param in params.iter() {
            query_builder = query_builder.bind(param);
        }
        let rows = query_builder
            .fetch_all(&mut self.tx)
            .await
            .map_err(|e| DatabaseError::query_error(format!("Query execution error: {}", e)))?;

        Ok(Box::new(PgRowSet::new(rows)))
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
        sqlx::query(&query)
            .execute(&mut self.tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!("Failed to create savepoint: {}", e))
            })?;
        Ok(())
    }

    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;
        let query = format!("ROLLBACK TO SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!("Failed to rollback to savepoint: {}", e))
            })?;
        Ok(())
    }

    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;
        let query = format!("RELEASE SAVEPOINT {}", name);
        sqlx::query(&query)
            .execute(&mut self.tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!("Failed to release savepoint: {}", e))
            })?;
        Ok(())
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

/// Validates a savepoint name to prevent SQL injection
/// Savepoint names should only contain alphanumeric characters and underscores
fn is_valid_savepoint_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub struct DatabaseConnectionManager {
    pub(crate) pool: Arc<Box<dyn DatabasePool>>,
}

impl DatabaseConnectionManager {
    pub fn new(pool: Box<dyn DatabasePool>) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub async fn transaction<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        let conn = self.pool.acquire().await?;
        let db_tx = conn.begin().await?;
        let mut tx = Transaction::new(db_tx);

        match f(tx.clone()).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(err) => {
                // Try to roll back the transaction, but if that fails, chain the errors
                if let Err(rollback_err) = tx.rollback().await {
                    Err(DatabaseError::ChainedError {
                        message: "Transaction failed with rollback error".to_string(),
                        context: ErrorContext::new()
                            .with_operation("transaction_rollback")
                            .with_additional_info("Rollback failed after transaction error"),
                        chain: vec![Box::new(err), Box::new(rollback_err)],
                    })
                } else {
                    Err(err)
                }
            }
        }
    }

    pub async fn transaction_with_retry<F, Fut, T>(
        &self,
        max_attempts: usize,
        f: F,
    ) -> DatabaseResult<T>
    where
        F: Fn(Transaction<'_>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = DatabaseResult<T>> + Send,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;

            match self.transaction(|tx| f(tx)).await {
                Ok(result) => return Ok(result),
                Err(err) if err.is_transient() && attempts < max_attempts => {
                    // Log the error and retry
                    warn!(
                        "Transient error during transaction (attempt {}/{}): {}. Retrying...",
                        attempts, max_attempts, err
                    );
                    last_error = Some(err);
                    // Add exponential backoff if needed
                    tokio::time::sleep(std::time::Duration::from_millis(
                        50 * (2_u64.pow(attempts as u32)),
                    ))
                    .await;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DatabaseError::TransactionError(
                "Transaction failed after maximum retry attempts".to_string(),
            )
        }))
    }

    pub async fn transaction_nested<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        self.transaction(|mut tx| async move { tx.nested(|| f(tx.clone())).await })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DatabaseError;
    use crate::transaction::Transaction;
    use mockall::mock;
    use mockall::predicate::*;
    use navius_test::error::{TestResult, assert_eq, assert_true};
    use std::sync::Arc;

    mock! {
        pub DatabasePool {}

        impl DatabasePool for DatabasePool {
            async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>>;
            async fn close(&self) -> DatabaseResult<()>;
            async fn check_health(&self) -> DatabaseResult<()>;
        }
    }

    mock! {
        pub DatabaseConn {}

        impl DatabaseConnection for DatabaseConn {
            #[cfg(feature = "postgres")]
            async fn execute<'a>(&mut self, query: &str, params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)]) -> DatabaseResult<u64>;

            #[cfg(feature = "postgres")]
            async fn query<'a>(&mut self, query: &str, params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)]) -> DatabaseResult<Box<dyn DatabaseRowSet>>;

            async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>>;
        }
    }

    mock! {
        pub DatabaseTx {}

        impl DatabaseTransaction for DatabaseTx {
            async fn execute<'a>(&mut self, query: &str, params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)]) -> DatabaseResult<u64>;
            async fn query<'a>(&mut self, query: &str, params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)]) -> DatabaseResult<Box<dyn DatabaseRowSet>>;
            async fn commit(self: Box<Self>) -> DatabaseResult<()>;
            async fn rollback(self: Box<Self>) -> DatabaseResult<()>;
            async fn savepoint(&mut self, name: &str) -> DatabaseResult<()>;
            async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()>;
            async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()>;
        }
    }

    mock! {
        pub DatabaseRowSet {}

        impl DatabaseRowSet for DatabaseRowSet {
            fn next(&mut self) -> DatabaseResult<Option<PgRow>>;
        }
    }

    #[tokio::test]
    async fn test_transaction_auto_rollback_on_error() -> TestResult<()> {
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
            Ok(tx)
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
        assert_true(
            result.is_err(),
            "Transaction should return an error when the closure returns an error",
        )?;

        if let Err(DatabaseError::QueryError(msg)) = result {
            assert_eq(
                msg,
                "Test error",
                "Error message should match the original error",
            )?;
        } else {
            return Err("Expected QueryError".into());
        }

        Ok(())
    }

    // Add more tests as needed...
}
