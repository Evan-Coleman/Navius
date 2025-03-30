use navius_db::error::{DatabaseError, DatabaseResult};
use sqlx::migrate::{Migrate, MigrateDatabase, Migrator};
use sqlx::postgres::PgPool;
use std::path::Path;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

/// Migration source - where migrations are loaded from
#[derive(Debug, Clone)]
pub enum MigrationSource {
    /// Migrations from a directory
    Directory(String),

    /// Embedded migrations
    Embedded(&'static [&'static str]),
}

/// Migration options
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Source of migrations
    pub source: MigrationSource,

    /// Whether to ignore missing migrations directory
    pub ignore_missing: bool,

    /// Whether to validate migrations only (don't apply)
    pub validate_only: bool,

    /// Lock timeout in seconds
    pub lock_timeout: Option<u64>,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            source: MigrationSource::Directory("./migrations".to_string()),
            ignore_missing: false,
            validate_only: false,
            lock_timeout: Some(30),
        }
    }
}

/// Information about a database migration
#[derive(Debug, Clone)]
pub struct MigrationInfo {
    /// The version of the migration
    pub version: i64,

    /// The description of the migration
    pub description: String,

    /// The installed time as ISO-8601 formatted string
    pub installed_on: String,

    /// Whether the migration was successful
    pub success: bool,

    /// Hash of the migration content
    pub checksum: Option<String>,

    /// Execution time in milliseconds
    pub execution_time: i64,
}

/// Database migrations manager
pub struct PgMigrationManager {
    pool: PgPool,
}

impl PgMigrationManager {
    /// Create a new migration manager
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check if database exists
    #[instrument(skip(connection_string))]
    pub async fn database_exists(connection_string: &str) -> DatabaseResult<bool> {
        debug!("Checking if database exists");

        match sqlx::Postgres::database_exists(connection_string).await {
            Ok(exists) => Ok(exists),
            Err(err) => Err(DatabaseError::MigrationError(format!(
                "Failed to check if database exists: {}",
                err
            ))),
        }
    }

    /// Create database
    #[instrument(skip(connection_string))]
    pub async fn create_database(connection_string: &str) -> DatabaseResult<()> {
        debug!("Creating database");

        match sqlx::Postgres::create_database(connection_string).await {
            Ok(_) => Ok(()),
            Err(err) => Err(DatabaseError::MigrationError(format!(
                "Failed to create database: {}",
                err
            ))),
        }
    }

    /// Drop database
    #[instrument(skip(connection_string))]
    pub async fn drop_database(connection_string: &str) -> DatabaseResult<()> {
        debug!("Dropping database");

        match sqlx::Postgres::drop_database(connection_string).await {
            Ok(_) => Ok(()),
            Err(err) => Err(DatabaseError::MigrationError(format!(
                "Failed to drop database: {}",
                err
            ))),
        }
    }

    /// Load migrator from source
    async fn load_migrator(&self, source: &MigrationSource) -> DatabaseResult<Migrator> {
        match source {
            MigrationSource::Directory(path) => {
                let migrator = match Migrator::new(Path::new(path)).await {
                    Ok(m) => m,
                    Err(err) => {
                        return Err(DatabaseError::MigrationError(format!(
                            "Failed to load migrations from directory: {}",
                            err
                        )));
                    }
                };
                Ok(migrator)
            }
            MigrationSource::Embedded(_files) => {
                // TODO: Implement embedded migrations once SQLx supports it in a more convenient way
                Err(DatabaseError::MigrationError(
                    "Embedded migrations are not yet supported".to_string(),
                ))
            }
        }
    }

    /// Apply migrations
    #[instrument(skip(self, options))]
    pub async fn migrate(&self, options: &MigrationOptions) -> DatabaseResult<()> {
        info!("Starting database migration");
        let start = Instant::now();

        let migrator = match self.load_migrator(&options.source).await {
            Ok(m) => m,
            Err(err) => {
                if let MigrationSource::Directory(_) = &options.source {
                    if options.ignore_missing {
                        info!("Ignoring missing migrations directory");
                        return Ok(());
                    }
                }
                return Err(err);
            }
        };

        // Set lock timeout if specified
        if let Some(timeout) = options.lock_timeout {
            debug!("Setting lock timeout to {} seconds", timeout);
            let lock_timeout_query = format!("SET LOCAL lock_timeout = '{} seconds'", timeout);
            match sqlx::query(&lock_timeout_query).execute(&self.pool).await {
                Ok(_) => {}
                Err(err) => {
                    error!("Failed to set lock timeout: {}", err);
                    // Continue anyway, it's not critical
                }
            }
        }

        if options.validate_only {
            debug!("Validating migrations only");
            match self.validate_migrations(&migrator).await {
                Ok(_) => {
                    info!("Migration validation successful");
                    Ok(())
                }
                Err(err) => {
                    error!("Migration validation failed: {}", err);
                    Err(err)
                }
            }
        } else {
            debug!("Applying migrations");
            match migrator.run(&self.pool).await {
                Ok(_) => {
                    let elapsed = start.elapsed();
                    info!(
                        "Database migration completed successfully in {:.2?}",
                        elapsed
                    );
                    Ok(())
                }
                Err(err) => {
                    error!("Database migration failed: {}", err);
                    Err(DatabaseError::MigrationError(format!(
                        "Migration failed: {}",
                        err
                    )))
                }
            }
        }
    }

    /// Validate migrations
    async fn validate_migrations(&self, migrator: &Migrator) -> DatabaseResult<()> {
        // Get applied migrations from database
        let applied_migrations = self.get_applied_migrations().await?;

        // Get migrations from migrator
        let available_migrations = migrator.iter().collect::<Vec<_>>();

        // Check if there are migrations in the database that are not in the migrator
        for applied in &applied_migrations {
            let migration_version: i64 = applied.version;

            if !available_migrations.iter().any(|m| {
                m.version.to_string().parse::<i64>().unwrap_or_default() == migration_version
            }) {
                return Err(DatabaseError::MigrationError(format!(
                    "Migration with version {} is applied in the database but not found in migration files",
                    migration_version
                )));
            }
        }

        // Check if the order of migrations is consistent
        let mut last_version = -1;
        for m in migrator.iter() {
            let version = m.version.to_string().parse::<i64>().unwrap_or_default();
            if version <= last_version {
                return Err(DatabaseError::MigrationError(format!(
                    "Migration version {} is not in increasing order (last version was {})",
                    version, last_version
                )));
            }
            last_version = version;
        }

        Ok(())
    }

    /// Get applied migrations
    #[instrument(skip(self))]
    pub async fn get_applied_migrations(&self) -> DatabaseResult<Vec<MigrationInfo>> {
        debug!("Fetching applied migrations");

        let rows = match sqlx::query(
            r#"
            SELECT 
                version, 
                description, 
                installed_on::text, 
                success, 
                checksum,
                execution_time
            FROM _sqlx_migrations 
            ORDER BY version ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        {
            Ok(rows) => rows,
            Err(err) => {
                if let sqlx::Error::Database(ref db_err) = err {
                    if db_err.code().unwrap_or_default() == "42P01" {
                        // Table doesn't exist, which means no migrations have been applied
                        debug!("Migration table doesn't exist yet");
                        return Ok(vec![]);
                    }
                }
                return Err(DatabaseError::MigrationError(format!(
                    "Failed to fetch applied migrations: {}",
                    err
                )));
            }
        };

        let migrations = rows
            .iter()
            .map(|row| MigrationInfo {
                version: row.get::<i64, _>("version"),
                description: row.get::<String, _>("description"),
                installed_on: row.get::<String, _>("installed_on"),
                success: row.get::<bool, _>("success"),
                checksum: row.get::<Option<String>, _>("checksum"),
                execution_time: row.get::<i64, _>("execution_time"),
            })
            .collect();

        Ok(migrations)
    }

    /// Run a conditional migration that will only be applied if the condition is met
    #[instrument(skip(self, sql, condition))]
    pub async fn conditional_migration(
        &self,
        sql: &str,
        condition: impl FnOnce(&PgPool) -> futures::future::BoxFuture<'_, bool>,
    ) -> DatabaseResult<bool> {
        debug!("Running conditional migration");

        // Check condition
        let should_run = condition(&self.pool).await;

        if should_run {
            debug!("Condition met, running migration");
            match sqlx::query(sql).execute(&self.pool).await {
                Ok(_) => {
                    debug!("Conditional migration successful");
                    Ok(true)
                }
                Err(err) => Err(DatabaseError::MigrationError(format!(
                    "Failed to run conditional migration: {}",
                    err
                ))),
            }
        } else {
            debug!("Condition not met, skipping migration");
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::fs;

    // Helper to create a test database URL
    fn test_db_url() -> String {
        let test_db_name = format!("navius_test_{}", uuid::Uuid::new_v4().simple());
        format!(
            "postgres://postgres:postgres@localhost:5432/{}",
            test_db_name
        )
    }

    // Helper to create a test migrations directory
    async fn create_test_migrations() -> String {
        let temp_dir =
            env::temp_dir().join(format!("migrations_{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir_all(&temp_dir).await.unwrap();

        // Create a test migration
        let migration_dir = temp_dir.join("20220101000000_create_test_table");
        fs::create_dir_all(&migration_dir).await.unwrap();

        // Create up.sql
        fs::write(
            migration_dir.join("up.sql"),
            "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
        )
        .await
        .unwrap();

        // Create down.sql
        fs::write(migration_dir.join("down.sql"), "DROP TABLE test_table;")
            .await
            .unwrap();

        // Return the temp dir as a string
        temp_dir.to_str().unwrap().to_string()
    }

    #[tokio::test]
    #[ignore] // Requires a PostgreSQL database
    async fn test_database_operations() {
        let db_url = test_db_url();

        // Check database doesn't exist yet
        let exists = PgMigrationManager::database_exists(&db_url).await.unwrap();
        assert!(!exists);

        // Create database
        PgMigrationManager::create_database(&db_url).await.unwrap();

        // Check database exists now
        let exists = PgMigrationManager::database_exists(&db_url).await.unwrap();
        assert!(exists);

        // Drop database
        PgMigrationManager::drop_database(&db_url).await.unwrap();

        // Check database doesn't exist anymore
        let exists = PgMigrationManager::database_exists(&db_url).await.unwrap();
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore] // Requires a PostgreSQL database
    async fn test_apply_migrations() {
        let db_url = test_db_url();

        // Create database
        PgMigrationManager::create_database(&db_url).await.unwrap();

        // Connect
        let pool = PgPool::connect(&db_url).await.unwrap();
        let manager = PgMigrationManager::new(pool.clone());

        // Create migrations
        let migrations_dir = create_test_migrations().await;

        // Apply migrations
        let options = MigrationOptions {
            source: MigrationSource::Directory(migrations_dir.clone()),
            ignore_missing: false,
            validate_only: false,
            lock_timeout: Some(10),
        };

        manager.migrate(&options).await.unwrap();

        // Check migrations were applied
        let migrations = manager.get_applied_migrations().await.unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, 20220101000000);

        // Check table exists
        let result = sqlx::query("SELECT * FROM test_table").execute(&pool).await;
        assert!(result.is_ok());

        // Clean up
        PgMigrationManager::drop_database(&db_url).await.unwrap();
    }
}
