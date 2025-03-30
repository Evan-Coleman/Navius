use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;

use crate::error::PgError;

/// Represents a database migration error
#[derive(Error, Debug)]
pub enum MigrationVersionError {
    /// Error executing SQL
    #[error("SQL error: {0}")]
    SqlError(#[from] sqlx::Error),

    /// Version tracking table not found
    #[error("Migration version table not found")]
    VersionTableNotFound,

    /// Migration version already exists
    #[error("Migration version already exists: {0}")]
    VersionAlreadyExists(String),

    /// Migration version not found
    #[error("Migration version not found: {0}")]
    VersionNotFound(String),

    /// Migration versions out of order
    #[error("Migration versions out of order. Expected: {expected}, found: {found}")]
    VersionsOutOfOrder { expected: String, found: String },

    /// Migration checksum mismatch
    #[error(
        "Migration checksum mismatch for version {version}. Expected: {expected}, found: {actual}"
    )]
    ChecksumMismatch {
        version: String,
        expected: String,
        actual: String,
    },
}

impl From<MigrationVersionError> for PgError {
    fn from(err: MigrationVersionError) -> Self {
        PgError::Migration(err.to_string())
    }
}

/// A database migration record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationVersion {
    /// Unique version identifier (e.g., "20250329110000")
    pub version: String,

    /// Description of the migration
    pub description: String,

    /// Checksum of the migration script
    pub checksum: String,

    /// When the migration was applied
    pub applied_at: DateTime<Utc>,

    /// Success status
    pub success: bool,
}

/// Manages migration versions in the database
pub struct VersionManager {
    pool: PgPool,
    table_name: String,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(pool: PgPool, table_name: Option<String>) -> Self {
        Self {
            pool,
            table_name: table_name.unwrap_or_else(|| "schema_migrations".to_string()),
        }
    }

    /// Initialize the version tracking table
    pub async fn initialize(&self) -> Result<(), MigrationVersionError> {
        sqlx::query(&format!(
            r#"
            CREATE TABLE IF NOT EXISTS {} (
                version VARCHAR(50) PRIMARY KEY,
                description TEXT NOT NULL,
                checksum VARCHAR(64) NOT NULL,
                applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                success BOOLEAN NOT NULL
            )
            "#,
            self.table_name
        ))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if the version tracking table exists
    pub async fn table_exists(&self) -> Result<bool, MigrationVersionError> {
        let result = sqlx::query_scalar::<_, i64>(&format!(
            r#"
            SELECT COUNT(*) FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = '{}'
            "#,
            self.table_name
        ))
        .fetch_one(&self.pool)
        .await?;

        Ok(result > 0)
    }

    /// Record a new migration version
    pub async fn record_version(
        &self,
        version: &str,
        description: &str,
        checksum: &str,
        success: bool,
    ) -> Result<(), MigrationVersionError> {
        // Check if version already exists
        if self.version_exists(version).await? {
            return Err(MigrationVersionError::VersionAlreadyExists(
                version.to_string(),
            ));
        }

        sqlx::query(&format!(
            r#"
            INSERT INTO {} (version, description, checksum, applied_at, success)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            self.table_name
        ))
        .bind(version)
        .bind(description)
        .bind(checksum)
        .bind(Utc::now())
        .bind(success)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Check if a version exists
    pub async fn version_exists(&self, version: &str) -> Result<bool, MigrationVersionError> {
        if !self.table_exists().await? {
            return Ok(false);
        }

        let result = sqlx::query_scalar::<_, i64>(&format!(
            r#"
            SELECT COUNT(*) FROM {} WHERE version = $1
            "#,
            self.table_name
        ))
        .bind(version)
        .fetch_one(&self.pool)
        .await?;

        Ok(result > 0)
    }

    /// Get all applied migrations
    pub async fn get_applied_migrations(
        &self,
    ) -> Result<Vec<MigrationVersion>, MigrationVersionError> {
        if !self.table_exists().await? {
            return Ok(Vec::new());
        }

        let migrations = sqlx::query_as!(
            MigrationVersion,
            r#"
            SELECT 
                version as "version!", 
                description as "description!", 
                checksum as "checksum!", 
                applied_at as "applied_at!", 
                success as "success!"
            FROM schema_migrations
            ORDER BY version ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(migrations)
    }

    /// Get the latest applied migration
    pub async fn get_latest_migration(
        &self,
    ) -> Result<Option<MigrationVersion>, MigrationVersionError> {
        if !self.table_exists().await? {
            return Ok(None);
        }

        let migration = sqlx::query_as!(
            MigrationVersion,
            r#"
            SELECT 
                version as "version!", 
                description as "description!", 
                checksum as "checksum!", 
                applied_at as "applied_at!", 
                success as "success!"
            FROM schema_migrations
            ORDER BY version DESC
            LIMIT 1
            "#
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(migration)
    }

    /// Validate a migration checksum
    pub async fn validate_checksum(
        &self,
        version: &str,
        checksum: &str,
    ) -> Result<bool, MigrationVersionError> {
        if !self.table_exists().await? {
            return Err(MigrationVersionError::VersionTableNotFound);
        }

        let stored_checksum = sqlx::query_scalar::<_, String>(&format!(
            r#"
            SELECT checksum FROM {} WHERE version = $1
            "#,
            self.table_name
        ))
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        match stored_checksum {
            Some(stored) => Ok(stored == checksum),
            None => Err(MigrationVersionError::VersionNotFound(version.to_string())),
        }
    }

    /// Clean up migration records (for testing)
    #[cfg(test)]
    pub async fn clean(&self) -> Result<(), MigrationVersionError> {
        if self.table_exists().await? {
            sqlx::query(&format!(r#"DROP TABLE IF EXISTS {}"#, self.table_name))
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn setup_test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to Postgres")
    }

    #[tokio::test]
    async fn test_version_manager() {
        let pool = setup_test_pool().await;
        let test_table = format!("test_migrations_{}", Utc::now().timestamp());

        let manager = VersionManager::new(pool, Some(test_table.clone()));

        // Clean up from previous test runs
        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", test_table))
            .execute(&manager.pool)
            .await;

        // Initialize
        manager
            .initialize()
            .await
            .expect("Failed to initialize version table");

        // Table should exist
        let exists = manager
            .table_exists()
            .await
            .expect("Failed to check table existence");
        assert!(exists);

        // Record a version
        manager
            .record_version("20250329000001", "Initial migration", "checksum1", true)
            .await
            .expect("Failed to record version");

        // Version should exist
        let exists = manager
            .version_exists("20250329000001")
            .await
            .expect("Failed to check version");
        assert!(exists);

        // Get applied migrations
        let migrations = manager
            .get_applied_migrations()
            .await
            .expect("Failed to get migrations");
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].version, "20250329000001");

        // Get latest migration
        let latest = manager
            .get_latest_migration()
            .await
            .expect("Failed to get latest migration");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "20250329000001");

        // Validate checksum
        let valid = manager
            .validate_checksum("20250329000001", "checksum1")
            .await
            .expect("Failed to validate checksum");
        assert!(valid);

        let invalid = manager
            .validate_checksum("20250329000001", "wrong")
            .await
            .expect("Failed to validate checksum");
        assert!(!invalid);

        // Clean up
        let _ = sqlx::query(&format!("DROP TABLE IF EXISTS {}", test_table))
            .execute(&manager.pool)
            .await;
    }
}
