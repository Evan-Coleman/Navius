//! PostgreSQL provider implementation.
//!
//! This module provides a PostgreSQL implementation of the DatabaseProvider interface
//! defined in the navius-db crate.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use navius_db::provider::{DatabaseProvider, ProviderOptions};
use navius_db::transaction::TransactionManager;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use tracing::{debug, info, warn};

use crate::error::PgError;
use crate::migration::{MigrationOptions, MigrationRunner};
use crate::pool::PgPoolConfig;
use crate::transaction::PgTransactionManager;

/// A PostgreSQL implementation of the DatabaseProvider trait
pub struct PostgresProvider {
    /// The PostgreSQL connection pool
    pool: PgPool,
    /// Provider options
    options: PostgresProviderOptions,
}

/// Options for configuring the PostgreSQL provider
#[derive(Debug, Clone)]
pub struct PostgresProviderOptions {
    /// PostgreSQL connection pool configuration
    pub pool_config: PgPoolConfig,
    /// Migration options
    pub migration_options: Option<MigrationOptions>,
    /// Whether to run migrations on startup
    pub run_migrations: bool,
    /// Whether to validate migrations on startup
    pub validate_migrations: bool,
}

impl Default for PostgresProviderOptions {
    fn default() -> Self {
        Self {
            pool_config: PgPoolConfig {
                url: "postgres://postgres:postgres@localhost/postgres".to_string(),
                max_connections: 10,
                min_connections: 1,
                max_lifetime: Some(std::time::Duration::from_secs(60 * 60)), // 1 hour
                idle_timeout: Some(std::time::Duration::from_secs(10 * 60)), // 10 minutes
                acquire_timeout: std::time::Duration::from_secs(30),         // 30 seconds
            },
            migration_options: None,
            run_migrations: false,
            validate_migrations: false,
        }
    }
}

impl From<ProviderOptions> for PostgresProviderOptions {
    fn from(options: ProviderOptions) -> Self {
        let mut pg_options = PostgresProviderOptions::default();

        // Extract database URL from provider options
        if let Some(url) = options.get("database_url") {
            pg_options.pool_config.url = url.to_string();
        }

        // Extract connection pool configuration
        if let Some(max_conn) = options
            .get("max_connections")
            .and_then(|v| v.parse::<u32>().ok())
        {
            pg_options.pool_config.max_connections = max_conn;
        }

        if let Some(min_conn) = options
            .get("min_connections")
            .and_then(|v| v.parse::<u32>().ok())
        {
            pg_options.pool_config.min_connections = min_conn;
        }

        // Extract migration configuration
        if let Some(migrations_dir) = options.get("migrations_dir") {
            let dir = PathBuf::from(migrations_dir);
            let table = options.get("migrations_table").map(|s| s.to_string());

            let validate = options
                .get("validate_migrations")
                .and_then(|v| v.parse::<bool>().ok())
                .unwrap_or(true);

            pg_options.migration_options = Some(MigrationOptions {
                migrations_dir: dir,
                migrations_table: table,
                validate_checksums: validate,
            });
        }

        // Extract migration execution flags
        if let Some(run) = options
            .get("run_migrations")
            .and_then(|v| v.parse::<bool>().ok())
        {
            pg_options.run_migrations = run;
        }

        if let Some(validate) = options
            .get("validate_migrations")
            .and_then(|v| v.parse::<bool>().ok())
        {
            pg_options.validate_migrations = validate;
        }

        pg_options
    }
}

impl PostgresProvider {
    /// Create a new PostgreSQL provider with the given options
    pub async fn new(options: PostgresProviderOptions) -> Result<Self, PgError> {
        // Parse connection options from URL using the standard parse method
        let connect_options = options
            .pool_config
            .url
            .parse::<PgConnectOptions>()
            .map_err(|e| PgError::Configuration(format!("Invalid connection URL: {}", e)))?;

        // Create the connection pool
        let pool = PgPoolOptions::new()
            .max_connections(options.pool_config.max_connections)
            .min_connections(options.pool_config.min_connections)
            .max_lifetime(options.pool_config.max_lifetime)
            .idle_timeout(options.pool_config.idle_timeout)
            .acquire_timeout(options.pool_config.acquire_timeout)
            .connect_with(connect_options)
            .await
            .map_err(|e| PgError::Connection(format!("Failed to create connection pool: {}", e)))?;

        let provider = Self { pool, options };

        // If configured, run migrations on startup
        if provider.options.run_migrations {
            provider.run_migrations().await?;
        }

        // If configured, validate migrations on startup
        if provider.options.validate_migrations {
            provider.validate_migrations().await?;
        }

        Ok(provider)
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
