use crate::config::DatabaseConfig;
use crate::error::{DatabaseError, DatabaseResult};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

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

/// Database transaction trait
#[async_trait]
pub trait DatabaseTransaction: DatabaseConnection {
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> DatabaseResult<()>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> DatabaseResult<()>;
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

#[cfg(feature = "postgres")]
#[async_trait]
impl DatabaseConnection for PgConnection {
    async fn execute(
        &mut self,
        _query: &str,
        _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        // This is a placeholder for now - actual implementation would use sqlx
        Ok(0)
    }

    async fn query(
        &mut self,
        _query: &str,
        _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        // This is a placeholder for now - actual implementation would use sqlx
        Err(DatabaseError::QueryError("Not implemented yet".to_string()))
    }

    async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>> {
        // This is a placeholder for now - actual implementation would use sqlx
        Err(DatabaseError::TransactionError(
            "Not implemented yet".to_string(),
        ))
    }
}
