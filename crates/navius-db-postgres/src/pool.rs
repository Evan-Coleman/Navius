//! PostgreSQL connection pool implementation.
//!
//! This module will be implemented in a future PR.

// Re-export SQLx pool types for now
pub use sqlx::postgres::{PgPool, PgPoolOptions};

/// Configuration for the PostgreSQL connection pool
#[derive(Debug, Clone)]
pub struct PgPoolConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Minimum number of connections
    pub min_connections: u32,
    /// Maximum lifetime of a connection
    pub max_lifetime: Option<std::time::Duration>,
    /// Connection idle timeout
    pub idle_timeout: Option<std::time::Duration>,
    /// Connection acquisition timeout
    pub acquire_timeout: std::time::Duration,
}
