use crate::error::{DatabaseError, DatabaseResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Database configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL (connection string)
    pub url: String,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections to maintain in the pool
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_seconds: u64,

    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: u64,

    /// Maximum lifetime of a connection in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_seconds: u64,

    /// Whether to enable connection tracing
    #[serde(default)]
    pub trace: bool,

    /// Whether to run migrations on startup
    #[serde(default)]
    pub run_migrations: bool,

    /// Path to migrations directory
    #[serde(default = "default_migrations_path")]
    pub migrations_path: String,

    /// Whether to abort on migration failure
    #[serde(default = "default_true")]
    pub abort_on_migration_failure: bool,
}

impl DatabaseConfig {
    /// Create a new database configuration with default values
    pub fn new(url: String) -> Self {
        Self {
            url,
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_seconds: default_connect_timeout(),
            idle_timeout_seconds: default_idle_timeout(),
            max_lifetime_seconds: default_max_lifetime(),
            trace: false,
            run_migrations: false,
            migrations_path: default_migrations_path(),
            abort_on_migration_failure: default_true(),
        }
    }

    /// Validate the database configuration
    pub fn validate(&self) -> DatabaseResult<()> {
        if self.url.is_empty() {
            return Err(DatabaseError::ConfigurationError(
                "Database URL cannot be empty".to_string(),
            ));
        }

        if self.max_connections < self.min_connections {
            return Err(DatabaseError::ConfigurationError(
                "Max connections must be greater than or equal to min connections".to_string(),
            ));
        }

        if self.min_connections == 0 {
            return Err(DatabaseError::ConfigurationError(
                "Min connections must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the connection timeout as a Duration
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_seconds)
    }

    /// Get the idle timeout as a Duration
    pub fn idle_timeout(&self) -> Duration {
        Duration::from_secs(self.idle_timeout_seconds)
    }

    /// Get the max lifetime as a Duration
    pub fn max_lifetime(&self) -> Duration {
        Duration::from_secs(self.max_lifetime_seconds)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_seconds: default_connect_timeout(),
            idle_timeout_seconds: default_idle_timeout(),
            max_lifetime_seconds: default_max_lifetime(),
            trace: false,
            run_migrations: false,
            migrations_path: default_migrations_path(),
            abort_on_migration_failure: default_true(),
        }
    }
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    300
}

fn default_max_lifetime() -> u64 {
    1800
}

fn default_migrations_path() -> String {
    "migrations".to_string()
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connect_timeout_seconds, 30);
        assert_eq!(config.idle_timeout_seconds, 300);
        assert_eq!(config.max_lifetime_seconds, 1800);
        assert_eq!(config.trace, false);
        assert_eq!(config.run_migrations, false);
        assert_eq!(config.migrations_path, "migrations");
        assert_eq!(config.abort_on_migration_failure, true);
    }

    #[test]
    fn test_new_config() {
        let url = "postgres://user:pass@localhost:5432/testdb".to_string();
        let config = DatabaseConfig::new(url.clone());
        assert_eq!(config.url, url);
    }

    #[test]
    fn test_validation_empty_url() {
        let config = DatabaseConfig {
            url: "".to_string(),
            ..DatabaseConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_min_greater_than_max() {
        let config = DatabaseConfig {
            min_connections: 10,
            max_connections: 5,
            ..DatabaseConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_min_zero() {
        let config = DatabaseConfig {
            min_connections: 0,
            ..DatabaseConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_config() {
        let config = DatabaseConfig {
            url: "postgres://user:pass@localhost:5432/testdb".to_string(),
            ..DatabaseConfig::default()
        };
        assert!(config.validate().is_ok());
    }
}
