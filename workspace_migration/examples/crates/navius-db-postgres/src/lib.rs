//! PostgreSQL implementation of the DatabaseProvider interface.
//!
//! This crate provides a PostgreSQL implementation of the DatabaseProvider interface
//! defined in the navius-db crate. It uses SQLx to interact with PostgreSQL.

// Public modules
pub mod error;
pub mod migration;
pub mod pool;
pub mod provider;
pub mod query;
pub mod transaction;

// Re-exports for convenience
pub use error::PgError;
pub use migration::{
    Migration, MigrationError, MigrationOptions, MigrationRunner, MigrationVersion,
    MigrationVersionError,
};
pub use pool::{PgPool, PgPoolConfig, PgPoolOptions};
pub use provider::PostgresProvider;
pub use query::PgQuery;
pub use transaction::PgTransaction;
