use crate::config::PgDatabaseConfig;
use crate::pool::{PgPool, PgPoolOptions};
use navius_db::error::{DatabaseError, DatabaseResult};
use navius_db::pool::DatabaseConnectionManager;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// PostgreSQL connection manager
pub struct PgConnectionManager {
    manager: DatabaseConnectionManager,
}

impl PgConnectionManager {
    /// Create a new connection manager with a PostgreSQL pool
    #[instrument(skip(config), fields(database_url = %format_db_url(&config.url)))]
    pub async fn new(config: &PgDatabaseConfig) -> DatabaseResult<Self> {
        info!("Creating PostgreSQL connection manager");
        debug!("Using pool options: {:?}", config.pool_options);

        // Create a new pool options from the config
        let pg_pool_options = PgPoolOptions::new()
            .max_connections(config.pool_options.max_connections)
            .min_connections(config.pool_options.min_connections)
            .acquire_timeout(config.pool_options.connect_timeout)
            .idle_timeout(config.pool_options.idle_timeout)
            .max_lifetime(config.pool_options.max_lifetime);

        // Connect to the database
        let pool = match pg_pool_options.connect(&config.url).await {
            Ok(pool) => pool,
            Err(e) => {
                error!("Failed to connect to PostgreSQL database: {}", e);
                return Err(DatabaseError::ConnectionError(format!(
                    "Failed to connect to PostgreSQL database: {}",
                    e
                )));
            }
        };

        debug!("Successfully connected to PostgreSQL database");

        // Create a PgPool wrapper for the SQLx pool
        let pg_pool = PgPool::new(pool);

        // Create a database connection manager with the pool
        let manager = DatabaseConnectionManager::new(Box::new(pg_pool));

        Ok(Self { manager })
    }

    /// Create a new connection manager with a PostgreSQL pool and run migrations
    #[instrument(skip(config), fields(database_url = %format_db_url(&config.url)))]
    pub async fn with_migrations(
        config: &PgDatabaseConfig,
        migrations_path: &str,
    ) -> DatabaseResult<Self> {
        info!("Creating PostgreSQL connection manager with migrations");

        // Create the connection manager first
        let manager = Self::new(config).await?;

        // Run migrations if configured to do so
        if config.run_migrations {
            use crate::migration::{MigrationOptions, MigrationSource, PgMigrationManager};

            info!("Running migrations from path: {}", migrations_path);

            // Extract pool reference from manager
            let pool = if let Some(pg_pool) = manager.get_pool().downcast_ref::<PgPool>() {
                pg_pool.inner().clone()
            } else {
                return Err(DatabaseError::MigrationError(
                    "Failed to get PostgreSQL pool for migrations".to_string(),
                ));
            };

            // Create migration manager and run migrations
            let migration_manager = PgMigrationManager::new(pool);
            let options = MigrationOptions {
                source: MigrationSource::Directory(migrations_path.to_string()),
                ignore_missing: false,
                validate_only: false,
                lock_timeout: Some(30),
            };

            migration_manager.migrate(&options).await?;
            info!("Migrations completed successfully");
        }

        Ok(manager)
    }

    /// Get a reference to the inner connection manager
    pub fn inner(&self) -> &DatabaseConnectionManager {
        &self.manager
    }

    /// Get the pool reference
    pub fn get_pool(&self) -> &Arc<Box<dyn navius_db::pool::DatabasePool>> {
        self.manager.get_pool()
    }

    /// Begin a transaction with the provided function
    pub async fn transaction<F, Fut, T>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(navius_db::transaction::Transaction<'_>) -> Fut,
        Fut: std::future::Future<Output = DatabaseResult<T>>,
    {
        self.manager.transaction(f).await
    }

    /// Begin a transaction with retry logic
    pub async fn transaction_with_retry<F, Fut, T>(
        &self,
        max_attempts: usize,
        f: F,
    ) -> DatabaseResult<T>
    where
        F: Fn(navius_db::transaction::Transaction<'_>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = DatabaseResult<T>> + Send,
    {
        self.manager.transaction_with_retry(max_attempts, f).await
    }
}

/// Format a database URL for logging, hiding credentials
fn format_db_url(url: &str) -> String {
    if let Some(prefix_end) = url.find("://") {
        let prefix = &url[0..prefix_end + 3];

        if let Some(credentials_end) = url[prefix_end + 3..].find('@') {
            let credentials_part = &url[prefix_end + 3..prefix_end + 3 + credentials_end];

            if let Some(colon_pos) = credentials_part.find(':') {
                let username = &credentials_part[0..colon_pos];
                // Hide the password part
                return format!(
                    "{}{}:*****@{}",
                    prefix,
                    username,
                    &url[prefix_end + 3 + credentials_end + 1..]
                );
            }
        }
    }

    // If we can't parse it properly, just hide most of it
    if url.len() > 10 {
        format!("{}...{}", &url[0..5], &url[url.len() - 5..])
    } else {
        "[hidden]".to_string()
    }
}
