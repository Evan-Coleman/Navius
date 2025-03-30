use async_trait::async_trait;
use navius_db::error::{DatabaseError, DatabaseResult};
use navius_db::pool::{DatabaseConnection, DatabasePool, DatabaseRowSet, DatabaseTransaction};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions as SqlxPgPoolOptions, PgRow as SqlxPgRow};
use std::fmt::{self, Debug};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// Re-export for convenience
pub use sqlx::postgres::PgPoolOptions as SqlxPgPoolOptions;

/// PostgreSQL connection pool
#[derive(Debug, Clone)]
pub struct PgPool {
    pool: Arc<sqlx::PgPool>,
}

impl PgPool {
    /// Create a new PostgreSQL connection pool
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    /// Create a new connection pool with the given options
    #[instrument(skip(url), fields(database_url = %redact_url(url)))]
    pub async fn connect(url: &str) -> DatabaseResult<Self> {
        info!("Connecting to PostgreSQL database");

        let pool = sqlx::PgPool::connect(url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(format!("Failed to connect: {}", e)))?;

        debug!("Successfully connected to PostgreSQL database");
        Ok(Self::new(pool))
    }

    /// Create a new connection pool with options
    #[instrument(skip(options, url), fields(database_url = %redact_url(url)))]
    pub async fn connect_with_options(options: &PgPoolOptions, url: &str) -> DatabaseResult<Self> {
        info!("Connecting to PostgreSQL database with custom options");
        debug!("Pool options: {:?}", options);

        let sqlx_options = SqlxPgPoolOptions::new()
            .max_connections(options.max_connections)
            .min_connections(options.min_connections)
            .acquire_timeout(options.connect_timeout)
            .idle_timeout(options.idle_timeout)
            .max_lifetime(options.max_lifetime);

        let pool = sqlx_options
            .connect(url)
            .await
            .map_err(|e| DatabaseError::ConnectionError(format!("Failed to connect: {}", e)))?;

        debug!("Successfully connected to PostgreSQL database");
        Ok(Self::new(pool))
    }

    /// Get the inner SQLx pool
    pub fn inner(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

/// Implementation of the DatabasePool trait for PostgreSQL connection pool
#[async_trait]
impl DatabasePool for PgPool {
    #[instrument(skip(self))]
    async fn acquire(&self) -> DatabaseResult<Box<dyn DatabaseConnection>> {
        debug!("Acquiring connection from PostgreSQL pool");

        let conn = self.pool.acquire().await.map_err(|e| {
            DatabaseError::ConnectionError(format!("Failed to acquire connection: {}", e))
        })?;

        Ok(Box::new(PgConnection::new(conn)))
    }

    #[instrument(skip(self))]
    async fn close(&self) -> DatabaseResult<()> {
        debug!("Closing PostgreSQL connection pool");
        self.pool.close().await;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn check_health(&self) -> DatabaseResult<()> {
        debug!("Checking PostgreSQL connection pool health");

        sqlx::query("SELECT 1")
            .execute(&*self.pool)
            .await
            .map_err(|e| DatabaseError::ConnectionError(format!("Health check failed: {}", e)))?;

        debug!("PostgreSQL connection pool is healthy");
        Ok(())
    }
}

/// PostgreSQL connection options wrapper
#[derive(Debug, Clone)]
pub struct PgPoolOptions {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Idle timeout before a connection is closed
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
}

impl PgPoolOptions {
    /// Create a new options object with default values
    pub fn new() -> Self {
        Self {
            max_connections: 10,
            min_connections: 0,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        }
    }

    /// Set the maximum number of connections
    pub fn max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Set the minimum number of connections
    pub fn min_connections(mut self, min_connections: u32) -> Self {
        self.min_connections = min_connections;
        self
    }

    /// Set the connection timeout
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the idle timeout
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set the maximum lifetime
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.max_lifetime = lifetime;
        self
    }
}

impl Default for PgPoolOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// PostgreSQL connection wrapper
pub struct PgConnection {
    conn: sqlx::pool::PoolConnection<sqlx::Postgres>,
}

impl PgConnection {
    /// Create a new connection wrapper
    fn new(conn: sqlx::pool::PoolConnection<sqlx::Postgres>) -> Self {
        Self { conn }
    }
}

impl fmt::Debug for PgConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgConnection").finish()
    }
}

/// Implementation of the DatabaseConnection trait for PostgreSQL
#[async_trait]
impl DatabaseConnection for PgConnection {
    #[instrument(skip(self, query, params), level = "debug")]
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let result = sqlx::query_with(query, params)
            .execute(&mut self.conn)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(result.rows_affected())
    }

    #[instrument(skip(self, query, params), level = "debug")]
    async fn query<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        let rows = sqlx::query_with(query, params)
            .fetch_all(&mut self.conn)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(Box::new(PgRowSet { rows, pos: 0 }))
    }

    #[instrument(skip(self), level = "debug")]
    async fn begin(&mut self) -> DatabaseResult<Box<dyn DatabaseTransaction>> {
        let tx = self.conn.begin().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to begin transaction: {}", e))
        })?;

        // Convert to static lifetime
        let tx = unsafe { std::mem::transmute(tx) };

        Ok(Box::new(crate::transaction::PgTransaction::new(tx)))
    }
}

/// PostgreSQL result set wrapper
struct PgRowSet {
    rows: Vec<SqlxPgRow>,
    pos: usize,
}

impl Debug for PgRowSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgRowSet")
            .field("row_count", &self.rows.len())
            .field("position", &self.pos)
            .finish()
    }
}

impl DatabaseRowSet for PgRowSet {
    fn next(&mut self) -> DatabaseResult<Option<crate::transaction::PgRow>> {
        if self.pos < self.rows.len() {
            let row = self.rows[self.pos].clone();
            self.pos += 1;
            Ok(Some(crate::transaction::PgRow::new(row)))
        } else {
            Ok(None)
        }
    }
}

// Row type re-export for convenience
pub use crate::transaction::PgRow;

// Transaction type re-export for convenience
pub use crate::transaction::PgTransaction;

/// Redact sensitive information from database URLs for logging
fn redact_url(url: &str) -> String {
    if let Some(auth_end) = url.find('@') {
        if let Some(protocol_end) = url.find("://") {
            if protocol_end < auth_end {
                let protocol = &url[0..protocol_end + 3];
                let after_auth = &url[auth_end..];
                return format!("{}***:***{}", protocol, after_auth);
            }
        }
    }

    // If we can't parse properly, return a heavily redacted version
    if url.len() > 10 {
        format!("{}...{}", &url[0..5], &url[url.len() - 5..])
    } else {
        "[redacted]".to_string()
    }
}
