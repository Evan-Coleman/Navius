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
        let migration_manager = PgMigrationManager::new(pool.clone());
        let options = MigrationOptions {
            source: MigrationSource::Directory(migrations_path.to_string()),
            ignore_missing: false,
            validate_only: false,
            lock_timeout: Some(30),
        };

        migration_manager.migrate(&options).await?;

        Ok(pool)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
