use crypto_hash::{Algorithm, Hasher, hex_digest};
use regex::Regex;
use sqlx::PgPool;
use std::fmt::Display;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, error, info, warn};

use crate::error::PgError;
use crate::migration::version::{MigrationVersion, MigrationVersionError, VersionManager};

/// Error that can occur during migrations
#[derive(Error, Debug)]
pub enum MigrationError {
    /// Error with migration version tracking
    #[error("Version error: {0}")]
    VersionError(#[from] MigrationVersionError),

    /// Error executing SQL
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),

    /// IO error reading migration files
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Error parsing migration filename
    #[error("Invalid migration filename: {0}")]
    InvalidFilename(String),

    /// No migrations found in directory
    #[error("No migrations found in directory: {0}")]
    NoMigrationsFound(String),

    /// Migration file is empty
    #[error("Migration file is empty: {0}")]
    EmptyMigrationFile(String),
}

impl From<MigrationError> for PgError {
    fn from(err: MigrationError) -> Self {
        PgError::Migration(err.to_string())
    }
}

/// A database migration that can be applied to the database
#[derive(Debug, Clone)]
pub struct Migration {
    /// Version identifier in format "YYYYMMDDhhmmss"
    pub version: String,

    /// Description of the migration
    pub description: String,

    /// Path to the migration file
    pub path: PathBuf,

    /// Content of the migration SQL
    pub content: String,

    /// Checksum of the content
    pub checksum: String,
}

impl Migration {
    /// Create a new migration from a file
    pub fn from_file(path: &Path) -> Result<Self, MigrationError> {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| MigrationError::InvalidFilename(path.display().to_string()))?;

        // Parse version and description from filename
        // Expected format: V20250329110000__Create_users_table.sql
        let re = Regex::new(r"^V(\d+)__(.+)\.sql$").unwrap();
        let captures = re
            .captures(filename)
            .ok_or_else(|| MigrationError::InvalidFilename(filename.to_string()))?;

        let version = captures
            .get(1)
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| MigrationError::InvalidFilename(filename.to_string()))?;

        let description = captures
            .get(2)
            .map(|m| m.as_str().replace('_', " "))
            .ok_or_else(|| MigrationError::InvalidFilename(filename.to_string()))?;

        // Read file content
        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Err(MigrationError::EmptyMigrationFile(filename.to_string()));
        }

        // Calculate checksum
        let checksum = calculate_checksum(&content);

        Ok(Self {
            version,
            description,
            path: path.to_path_buf(),
            content,
            checksum,
        })
    }
}

impl Display for Migration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migration(version={}, description={})",
            self.version, self.description
        )
    }
}

/// Calculate a SHA-256 checksum of the content
fn calculate_checksum(content: &str) -> String {
    let mut hasher = Hasher::new(Algorithm::SHA256);
    hasher.update(content.as_bytes());
    hex_digest(hasher)
}

/// Options for running migrations
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Directory containing migration files
    pub migrations_dir: PathBuf,

    /// Name of the migrations table
    pub migrations_table: Option<String>,

    /// Whether to validate checksums
    pub validate_checksums: bool,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            migrations_dir: PathBuf::from("migrations"),
            migrations_table: None,
            validate_checksums: true,
        }
    }
}

/// A runner for database migrations
pub struct MigrationRunner {
    pool: PgPool,
    version_manager: VersionManager,
    options: MigrationOptions,
}

impl MigrationRunner {
    /// Create a new migration runner
    pub fn new(pool: PgPool, options: MigrationOptions) -> Self {
        let version_manager = VersionManager::new(pool.clone(), options.migrations_table.clone());

        Self {
            pool,
            version_manager,
            options,
        }
    }

    /// Initialize the migration system
    pub async fn initialize(&self) -> Result<(), MigrationError> {
        self.version_manager.initialize().await?;
        Ok(())
    }

    /// Find all migrations in the migrations directory
    pub async fn find_migrations(&self) -> Result<Vec<Migration>, MigrationError> {
        let dir = &self.options.migrations_dir;

        if !dir.exists() {
            return Err(MigrationError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Migrations directory not found: {}", dir.display()),
            )));
        }

        let mut migrations = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
                // Check if filename matches the expected pattern
                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if filename.starts_with('V') && filename.contains("__") {
                        let migration = Migration::from_file(&path)?;
                        migrations.push(migration);
                    }
                }
            }
        }

        if migrations.is_empty() {
            return Err(MigrationError::NoMigrationsFound(dir.display().to_string()));
        }

        // Sort migrations by version
        migrations.sort_by(|a, b| a.version.cmp(&b.version));

        Ok(migrations)
    }

    /// Get a list of pending migrations
    pub async fn pending_migrations(&self) -> Result<Vec<Migration>, MigrationError> {
        let all_migrations = self.find_migrations().await?;
        let applied_migrations = self.version_manager.get_applied_migrations().await?;

        let applied_versions: Vec<String> = applied_migrations
            .iter()
            .map(|m| m.version.clone())
            .collect();

        let pending = all_migrations
            .into_iter()
            .filter(|m| !applied_versions.contains(&m.version))
            .collect();

        Ok(pending)
    }

    /// Execute a single migration
    async fn execute_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        info!("Applying migration: {}", migration);

        // Verify checksum if the migration was already applied
        if self
            .version_manager
            .version_exists(&migration.version)
            .await?
            && self.options.validate_checksums
        {
            let is_valid = self
                .version_manager
                .validate_checksum(&migration.version, &migration.checksum)
                .await?;

            if !is_valid {
                error!("Checksum mismatch for migration: {}", migration);
                return Err(MigrationError::VersionError(
                    MigrationVersionError::ChecksumMismatch {
                        version: migration.version.clone(),
                        expected: "unknown".to_string(), // We don't have this info
                        actual: migration.checksum.clone(),
                    },
                ));
            }
        }

        // Create a transaction
        let mut tx = self.pool.begin().await?;

        // Execute the migration
        let start = Instant::now();
        let result = sqlx::query(&migration.content).execute(&mut *tx).await;

        match result {
            Ok(result) => {
                let duration = start.elapsed();
                info!(
                    "Successfully applied migration {} in {:.2}s (rows affected: {})",
                    migration.version,
                    duration.as_secs_f64(),
                    result.rows_affected()
                );

                // Record successful migration
                self.version_manager
                    .record_version(
                        &migration.version,
                        &migration.description,
                        &migration.checksum,
                        true,
                    )
                    .await?;

                // Commit the transaction
                tx.commit().await?;
                Ok(())
            }
            Err(err) => {
                error!("Failed to apply migration {}: {}", migration.version, err);

                // Record failed migration
                self.version_manager
                    .record_version(
                        &migration.version,
                        &migration.description,
                        &migration.checksum,
                        false,
                    )
                    .await?;

                // Rollback the transaction
                let _ = tx.rollback().await;
                Err(MigrationError::SqlError(err))
            }
        }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<usize, MigrationError> {
        self.initialize().await?;

        let pending = self.pending_migrations().await?;

        if pending.is_empty() {
            info!("No pending migrations found");
            return Ok(0);
        }

        info!("Found {} pending migrations", pending.len());

        let mut count = 0;
        for migration in pending {
            self.execute_migration(&migration).await?;
            count += 1;
        }

        info!("Successfully applied {} migrations", count);
        Ok(count)
    }

    /// Validate all migrations
    pub async fn validate(&self) -> Result<bool, MigrationError> {
        self.initialize().await?;

        let all_migrations = self.find_migrations().await?;
        let applied_migrations = self.version_manager.get_applied_migrations().await?;

        // Check for applied migrations with no corresponding file
        let all_versions: Vec<String> = all_migrations.iter().map(|m| m.version.clone()).collect();

        for applied in &applied_migrations {
            if !all_versions.contains(&applied.version) {
                warn!(
                    "Migration {} is applied but not found in filesystem",
                    applied.version
                );
            }
        }

        // Check for checksum mismatches
        if self.options.validate_checksums {
            for migration in &all_migrations {
                if self
                    .version_manager
                    .version_exists(&migration.version)
                    .await?
                {
                    let is_valid = self
                        .version_manager
                        .validate_checksum(&migration.version, &migration.checksum)
                        .await?;

                    if !is_valid {
                        error!("Checksum mismatch for migration: {}", migration.version);
                        return Ok(false);
                    }
                }
            }
        }

        // Check for version order issues
        let mut last_version = None;

        for migration in &applied_migrations {
            if let Some(last) = &last_version {
                if migration.version <= *last {
                    error!(
                        "Migration versions out of order: {} after {}",
                        migration.version, last
                    );
                    return Ok(false);
                }
            }

            last_version = Some(migration.version.clone());
        }

        debug!("Migration validation successful");
        Ok(true)
    }

    /// Get information about all migrations
    pub async fn info(&self) -> Result<Vec<MigrationStatus>, MigrationError> {
        self.initialize().await?;

        let all_migrations = self.find_migrations().await?;
        let applied_migrations = self.version_manager.get_applied_migrations().await?;

        let applied_map: std::collections::HashMap<String, MigrationVersion> = applied_migrations
            .into_iter()
            .map(|m| (m.version.clone(), m))
            .collect();

        let mut statuses = Vec::new();

        for migration in all_migrations {
            let status = if let Some(applied) = applied_map.get(&migration.version) {
                MigrationStatus {
                    migration,
                    state: MigrationState::Applied {
                        applied_at: applied.applied_at,
                        success: applied.success,
                    },
                }
            } else {
                MigrationStatus {
                    migration,
                    state: MigrationState::Pending,
                }
            };

            statuses.push(status);
        }

        // Add missing applied migrations
        for (version, applied) in applied_map {
            if !statuses.iter().any(|s| s.migration.version == version) {
                statuses.push(MigrationStatus {
                    migration: Migration {
                        version: applied.version.clone(),
                        description: applied.description,
                        path: PathBuf::new(), // Missing file
                        content: String::new(),
                        checksum: applied.checksum,
                    },
                    state: MigrationState::Missing {
                        applied_at: applied.applied_at,
                        success: applied.success,
                    },
                });
            }
        }

        // Sort by version
        statuses.sort_by(|a, b| a.migration.version.cmp(&b.migration.version));

        Ok(statuses)
    }
}

/// Represents the state of a migration
#[derive(Debug, Clone)]
pub enum MigrationState {
    /// Migration has been applied
    Applied {
        /// When the migration was applied
        applied_at: chrono::DateTime<chrono::Utc>,
        /// Whether the migration was successful
        success: bool,
    },

    /// Migration is pending
    Pending,

    /// Migration is in database but missing from filesystem
    Missing {
        /// When the migration was applied
        applied_at: chrono::DateTime<chrono::Utc>,
        /// Whether the migration was successful
        success: bool,
    },
}

/// Status of a migration
#[derive(Debug, Clone)]
pub struct MigrationStatus {
    /// The migration
    pub migration: Migration,

    /// State of the migration
    pub state: MigrationState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use tempfile::TempDir;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;
    use tokio::test;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    async fn create_migration_file(
        dir: &Path,
        version: &str,
        description: &str,
        content: &str,
    ) -> PathBuf {
        let filename = format!("V{}__{}.sql", version, description.replace(' ', "_"));
        let path = dir.join(filename);

        let mut file = File::create(&path).await.expect("Failed to create file");
        file.write_all(content.as_bytes())
            .await
            .expect("Failed to write file");
        file.flush().await.expect("Failed to flush file");

        path
    }

    #[test]
    async fn test_migration_runner() {
        // Create a temp directory for migrations
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let migrations_dir = temp_dir.path();

        // Create test migrations
        create_migration_file(
            migrations_dir,
            "20250329000001",
            "Create test table",
            "CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
        )
        .await;

        create_migration_file(
            migrations_dir,
            "20250329000002",
            "Add columns",
            "ALTER TABLE test_table ADD COLUMN created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW();",
        ).await;

        // Create a unique table name for this test
        let test_table_name = format!("test_migrations_{}", chrono::Utc::now().timestamp());

        // Setup the pool and runner
        let pool = setup_test_pool().await;
        let options = MigrationOptions {
            migrations_dir: migrations_dir.to_path_buf(),
            migrations_table: Some(test_table_name.clone()),
            validate_checksums: true,
        };

        let runner = MigrationRunner::new(pool.clone(), options);

        // Initialize and clean up from previous test runs
        runner.initialize().await.expect("Failed to initialize");

        // Run migrations
        let count = runner
            .run_migrations()
            .await
            .expect("Failed to run migrations");
        assert_eq!(count, 2, "Expected 2 migrations to be applied");

        // Check that migrations table exists and has 2 records
        let migrations = runner
            .version_manager
            .get_applied_migrations()
            .await
            .expect("Failed to get applied migrations");
        assert_eq!(migrations.len(), 2, "Expected 2 applied migrations");

        // Verify the test table was created
        let table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT FROM information_schema.tables 
                WHERE table_schema = 'public' 
                AND table_name = 'test_table'
            )",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to check if test table exists");

        assert!(table_exists, "Test table should exist");

        // Verify no pending migrations
        let pending = runner
            .pending_migrations()
            .await
            .expect("Failed to get pending migrations");
        assert!(pending.is_empty(), "There should be no pending migrations");

        // Validate migrations
        let valid = runner
            .validate()
            .await
            .expect("Failed to validate migrations");
        assert!(valid, "Migrations should be valid");

        // Clean up
        sqlx::query("DROP TABLE IF EXISTS test_table")
            .execute(&pool)
            .await
            .expect("Failed to drop test table");

        sqlx::query(&format!("DROP TABLE IF EXISTS {}", test_table_name))
            .execute(&pool)
            .await
            .expect("Failed to drop migrations table");
    }
}
