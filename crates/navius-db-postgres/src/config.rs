use navius_db::config::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for PostgreSQL database connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PgDatabaseConfig {
    /// Host name or IP address of the PostgreSQL server
    pub host: String,

    /// Port of the PostgreSQL server
    pub port: u16,

    /// Name of the database to connect to
    pub database: String,

    /// Username for authentication
    pub username: String,

    /// Password for authentication
    pub password: String,

    /// Maximum number of connections in the pool
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of idle connections to maintain
    #[serde(default = "default_min_idle")]
    pub min_idle: Option<u32>,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_seconds: u64,

    /// Idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_seconds: Option<u64>,

    /// Max lifetime of a connection in seconds
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime_seconds: Option<u64>,

    /// Path to migrations directory
    #[serde(default)]
    pub migrations_path: Option<String>,

    /// Whether to use TLS for database connections
    #[serde(default)]
    pub use_tls: bool,

    /// Application name to identify the connection in pg_stat_activity
    #[serde(default = "default_app_name")]
    pub application_name: String,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_idle() -> Option<u32> {
    Some(2)
}

fn default_connect_timeout() -> u64 {
    10
}

fn default_idle_timeout() -> Option<u64> {
    Some(300)
}

fn default_max_lifetime() -> Option<u64> {
    Some(1800)
}

fn default_app_name() -> String {
    "navius_app".to_string()
}

impl PgDatabaseConfig {
    /// Create a new configuration with default values
    pub fn new(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        Self {
            host: host.to_string(),
            port,
            database: database.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            max_connections: default_max_connections(),
            min_idle: default_min_idle(),
            connect_timeout_seconds: default_connect_timeout(),
            idle_timeout_seconds: default_idle_timeout(),
            max_lifetime_seconds: default_max_lifetime(),
            migrations_path: None,
            use_tls: false,
            application_name: default_app_name(),
        }
    }

    /// Get the database connection string for this configuration
    pub fn connection_string(&self) -> String {
        let mut conn_str = format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        );

        // Add additional parameters
        conn_str.push_str(&format!("?application_name={}", self.application_name));

        if !self.use_tls {
            conn_str.push_str("&sslmode=disable");
        }

        conn_str
    }

    /// Create pool options from this configuration
    pub fn create_pool_options(&self) -> sqlx::postgres::PgPoolOptions {
        let mut options =
            sqlx::postgres::PgPoolOptions::new().max_connections(self.max_connections);

        if let Some(min_idle) = self.min_idle {
            options = options.min_connections(min_idle);
        }

        options = options.acquire_timeout(Duration::from_secs(self.connect_timeout_seconds));

        if let Some(idle_timeout) = self.idle_timeout_seconds {
            options = options.idle_timeout(Some(Duration::from_secs(idle_timeout)));
        }

        if let Some(max_lifetime) = self.max_lifetime_seconds {
            options = options.max_lifetime(Some(Duration::from_secs(max_lifetime)));
        }

        options
    }
}

impl From<PgDatabaseConfig> for DatabaseConfig {
    fn from(config: PgDatabaseConfig) -> Self {
        DatabaseConfig {
            provider: "postgresql".to_string(),
            connection_string: config.connection_string(),
            max_connections: config.max_connections,
            min_idle: config.min_idle,
            connect_timeout_seconds: config.connect_timeout_seconds,
            idle_timeout_seconds: config.idle_timeout_seconds,
            max_lifetime_seconds: config.max_lifetime_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_string() {
        let config =
            PgDatabaseConfig::new("localhost", 5432, "test_db", "test_user", "test_password");

        let conn_str = config.connection_string();
        assert!(conn_str.contains("postgres://test_user:test_password@localhost:5432/test_db"));
        assert!(conn_str.contains("application_name=navius_app"));
        assert!(conn_str.contains("sslmode=disable"));
    }

    #[test]
    fn test_create_pool_options() {
        let config = PgDatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "test_db".to_string(),
            username: "test_user".to_string(),
            password: "test_password".to_string(),
            max_connections: 20,
            min_idle: Some(5),
            connect_timeout_seconds: 15,
            idle_timeout_seconds: Some(400),
            max_lifetime_seconds: Some(2000),
            migrations_path: None,
            use_tls: false,
            application_name: "test_app".to_string(),
        };

        let options = config.to_pool_options();
        // Note: PgPoolOptions doesn't expose its internal fields, so we can't directly test them.
        // This is more of a smoke test to ensure the method doesn't panic.
        assert!(options.max_connections >= 20);
    }
}
