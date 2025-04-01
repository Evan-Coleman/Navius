//! PostgreSQL provider implementation.
//!
//! This module provides a PostgreSQL implementation of the DatabaseProvider interface
//! defined in the navius-db crate.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use navius_db::provider::{DatabaseProvider, ProviderOptions};
use navius_db::transaction::TransactionManager;
use sqlx::Postgres;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use tracing::{debug, info, warn};

use crate::error::PgError;
use crate::migration::{MigrationOptions, MigrationRunner};
use crate::pool::PgPoolConfig;
use crate::transaction::PgTransactionManager;

use navius_db::connection::DatabaseConnection;
use navius_db::error::DatabaseError;
use navius_db::pool::ConnectionPool;
use navius_db::transaction::DatabaseTransaction;

use crate::connection::PostgresConnection;
use crate::error::PostgresError;
use crate::transaction::PostgresTransaction;

/// Options for configuring the Postgres database provider
#[derive(Debug, Clone)]
pub struct PostgresProviderOptions {
    /// Database URL
    pub url: String,

    /// Maximum number of connections in the pool
    pub max_connections: u32,

    /// Connection timeout in seconds
    pub connect_timeout_secs: u64,

    /// Idle timeout in seconds
    pub idle_timeout_secs: u64,

    /// Maximum lifetime of a connection in seconds
    pub max_lifetime_secs: u64,
}

impl Default for PostgresProviderOptions {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/postgres".to_string(),
            max_connections: 10,
            connect_timeout_secs: 30,
            idle_timeout_secs: 300,
            max_lifetime_secs: 1800,
        }
    }
}

/// A provider for PostgreSQL databases
pub struct PostgresProvider {
    /// Connection pool
    pool: PgPool,

    /// Provider options
    options: PostgresProviderOptions,
}

impl PostgresProvider {
    /// Create a new Postgres provider with the given options
    pub async fn new(options: PostgresProviderOptions) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .connect_timeout(Duration::from_secs(options.connect_timeout_secs))
            .idle_timeout(Some(Duration::from_secs(options.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(options.max_lifetime_secs)))
            .connect(&options.url)
            .await
            .map_err(|e| PostgresError::from(e))?;

        Ok(Self { pool, options })
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<usize, PgError> {
        if let Some(migration_options) = &self.options.migration_options {
            info!("Running database migrations");
            let runner = MigrationRunner::new(self.pool.clone(), migration_options.clone());

            let count = runner
                .run_migrations()
                .await
                .map_err(|e| PgError::Migration(format!("Failed to run migrations: {}", e)))?;

            info!("Applied {} migrations", count);
            Ok(count)
        } else {
            warn!("No migration options configured, skipping migrations");
            Ok(0)
        }
    }

    /// Validate database migrations
    pub async fn validate_migrations(&self) -> Result<bool, PgError> {
        if let Some(migration_options) = &self.options.migration_options {
            info!("Validating database migrations");
            let runner = MigrationRunner::new(self.pool.clone(), migration_options.clone());

            let valid = runner
                .validate()
                .await
                .map_err(|e| PgError::Migration(format!("Failed to validate migrations: {}", e)))?;

            if valid {
                info!("Migration validation successful");
            } else {
                warn!("Migration validation failed");
            }
            Ok(valid)
        } else {
            debug!("No migration options configured, skipping validation");
            Ok(true)
        }
    }

    /// Get information about migrations
    pub async fn migration_info(&self) -> Result<Vec<crate::migration::MigrationStatus>, PgError> {
        if let Some(migration_options) = &self.options.migration_options {
            let runner = MigrationRunner::new(self.pool.clone(), migration_options.clone());

            let status = runner
                .info()
                .await
                .map_err(|e| PgError::Migration(format!("Failed to get migration info: {}", e)))?;

            Ok(status)
        } else {
            warn!("No migration options configured, unable to get migration info");
            Ok(Vec::new())
        }
    }

    /// Get the underlying connection pool
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    /// Create a transaction manager
    pub fn transaction_manager(&self) -> Arc<dyn TransactionManager> {
        Arc::new(PgTransactionManager::new(self.pool.clone()))
    }
}

#[async_trait]
impl ConnectionPool for PostgresProvider {
    type Connection = PostgresConnection;
    type Transaction = PostgresTransaction;

    async fn acquire(&self) -> Result<Arc<Self::Connection>, DatabaseError> {
        let conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(Arc::new(PostgresConnection::new(conn)))
    }

    async fn begin_transaction(&self) -> Result<Arc<Self::Transaction>, DatabaseError> {
        let conn = self
            .pool
            .begin()
            .await
            .map_err(|e| PostgresError::from(e))?;
        Ok(Arc::new(PostgresTransaction::new(conn)))
    }

    async fn close(&self) -> Result<(), DatabaseError> {
        self.pool.close().await;
        Ok(())
    }
}

#[async_trait]
impl DatabaseProvider for PostgresProvider {
    async fn connect(options: ProviderOptions) -> Result<Self, navius_db::error::DatabaseError> {
        let pg_options = PostgresProviderOptions::from(options);

        let provider = Self::new(pg_options).await.map_err(|e| e.into())?;

        Ok(provider)
    }

    fn name(&self) -> &str {
        "postgresql"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn health_check(&self) -> Result<bool, navius_db::error::DatabaseError> {
        // Simple health check - try to execute a simple query
        match sqlx::query("SELECT 1").execute(&self.pool).await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }

    // Additional methods required by the DatabaseProvider trait will be implemented
    // in future PRs
}
