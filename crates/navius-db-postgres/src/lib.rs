/*!
Navius DB PostgreSQL - PostgreSQL implementation for the Navius database framework

This crate provides PostgreSQL connectivity for the Navius database framework using SQLx.
It implements the core database interfaces defined in navius-db.
*/

// Re-export sqlx for convenience
pub use sqlx;

// Internal modules
mod config;
mod connection;
mod error;
mod migration;
mod pool;
mod query;
mod repository;
mod transaction;

// Public exports
pub use config::PgDatabaseConfig;
pub use connection::PgConnectionManager;
pub use error::PgDatabaseError;
pub use migration::{MigrationInfo, MigrationOptions, MigrationSource, PgMigrationManager};
pub use pool::{PgConnection, PgPool, PgPoolOptions, PgRow, PgTransaction};
pub use query::{PgQuery, PgQueryExecutor};
pub use repository::PgRepository;
pub use transaction::PgTransaction as Transaction;

/// PostgreSQL provider for navius-db
pub struct PostgresProvider;

/// Implementation of the DatabaseProvider trait for PostgreSQL
impl navius_db::DatabaseProvider for PostgresProvider {
    fn name(&self) -> &'static str {
        "postgresql"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn supports_migrations(&self) -> bool {
        true
    }
}

/// Helper functions for common database operations
pub mod helpers {
    use crate::PgPool;
    use crate::migration::{MigrationOptions, MigrationSource, PgMigrationManager};
    use navius_db::error::DatabaseResult;

    /// Initialize a database with migrations
    ///
    /// This function will:
    /// 1. Check if the database exists, creating it if it doesn't
    /// 2. Connect to the database
    /// 3. Run migrations
    ///
    /// # Arguments
    ///
    /// * `connection_string` - The connection string to the database
    /// * `migrations_path` - The path to the migrations directory
    /// * `create_if_missing` - Whether to create the database if it doesn't exist
    ///
    /// # Returns
    ///
    /// * `Ok(pool)` - The database connection pool
    /// * `Err(e)` - An error if any operation fails
    pub async fn initialize_database(
        connection_string: &str,
        migrations_path: &str,
        create_if_missing: bool,
    ) -> DatabaseResult<PgPool> {
        // Check if database exists
        let exists = PgMigrationManager::database_exists(connection_string).await?;

        if !exists && create_if_missing {
            // Create database
            PgMigrationManager::create_database(connection_string).await?;
        } else if !exists {
            return Err(navius_db::error::DatabaseError::MigrationError(format!(
                "Database does not exist and create_if_missing is false"
            )));
        }

        // Connect to database
        let pool = match PgPool::connect(connection_string).await {
            Ok(pool) => pool,
            Err(e) => {
                return Err(navius_db::error::DatabaseError::ConnectionError(format!(
                    "Failed to connect to database: {}",
                    e
                )));
            }
        };

        // Run migrations
        let migration_manager = PgMigrationManager::new(pool.inner().clone());
        let options = MigrationOptions {
            source: MigrationSource::Directory(migrations_path.to_string()),
            ignore_missing: false,
            validate_only: false,
            lock_timeout: Some(30),
        };

        migration_manager.migrate(&options).await?;

        Ok(pool)
    }

    /// Create a connection manager with a database pool
    ///
    /// This function will:
    /// 1. Connect to the database
    /// 2. Run migrations if specified
    /// 3. Create a connection manager
    ///
    /// # Arguments
    ///
    /// * `connection_string` - The connection string to the database
    /// * `migrations_path` - The path to the migrations directory (if migrations should be run)
    /// * `run_migrations` - Whether to run migrations
    ///
    /// # Returns
    ///
    /// * `Ok(manager)` - The database connection manager
    /// * `Err(e)` - An error if any operation fails
    pub async fn create_connection_manager(
        connection_string: &str,
        migrations_path: Option<&str>,
        run_migrations: bool,
    ) -> DatabaseResult<crate::connection::PgConnectionManager> {
        // Parse connection string to extract DB info (this is simplified, real implementation would be more robust)
        // Format expected: postgres://username:password@host:port/database
        let connection_parts: Vec<&str> = connection_string.split("://").collect();
        if connection_parts.len() != 2 {
            return Err(navius_db::error::DatabaseError::ConfigurationError(
                "Invalid connection string format".to_string(),
            ));
        }

        let auth_and_db: Vec<&str> = connection_parts[1].split('@').collect();
        if auth_and_db.len() != 2 {
            return Err(navius_db::error::DatabaseError::ConfigurationError(
                "Invalid connection string format".to_string(),
            ));
        }

        let auth: Vec<&str> = auth_and_db[0].split(':').collect();
        if auth.len() != 2 {
            return Err(navius_db::error::DatabaseError::ConfigurationError(
                "Invalid connection string format".to_string(),
            ));
        }

        let host_port_db: Vec<&str> = auth_and_db[1].split('/').collect();
        if host_port_db.len() < 2 {
            return Err(navius_db::error::DatabaseError::ConfigurationError(
                "Invalid connection string format".to_string(),
            ));
        }

        let host_port: Vec<&str> = host_port_db[0].split(':').collect();
        if host_port.len() != 2 {
            return Err(navius_db::error::DatabaseError::ConfigurationError(
                "Invalid connection string format".to_string(),
            ));
        }

        let port: u16 = host_port[1].parse().map_err(|_| {
            navius_db::error::DatabaseError::ConfigurationError(
                "Invalid port in connection string".to_string(),
            )
        })?;

        // Create a config
        let config = crate::PgDatabaseConfig {
            host: host_port[0].to_string(),
            port,
            database: host_port_db[1].to_string(),
            username: auth[0].to_string(),
            password: auth[1].to_string(),
            max_connections: 10,
            min_idle: Some(2),
            connect_timeout_seconds: 30,
            idle_timeout_seconds: Some(600),
            max_lifetime_seconds: Some(1800),
            migrations_path: migrations_path.map(String::from),
            use_tls: false,
            application_name: "navius_app".to_string(),
        };

        // Override the actual URL to ensure we use the original, validated string
        let mut config_with_url = config.clone();

        // Set run_migrations flag
        if let Some(path) = migrations_path {
            if run_migrations {
                crate::connection::PgConnectionManager::with_migrations(&config, path).await
            } else {
                crate::connection::PgConnectionManager::new(&config).await
            }
        } else {
            crate::connection::PgConnectionManager::new(&config).await
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
