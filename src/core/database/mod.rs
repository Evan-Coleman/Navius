//! Database module for PostgreSQL integration
//!
//! This module provides functionality for connecting to PostgreSQL databases,
//! managing connections, and executing type-safe queries.

pub mod connection;
pub mod error;
pub mod repositories;
pub mod repository;
pub mod transaction;
pub mod utils;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub use connection::MockDatabaseConnection;
pub use connection::{DatabaseConnection, init_database, ping_database};
pub use error::DatabaseError;
pub use repository::EntityRepository;
pub use transaction::Transaction;
pub use utils::{db_error_message, exists, generate_uuid, with_transaction};

use crate::core::error::AppError;
use async_trait::async_trait;

/// PostgreSQL Connection Pool trait
///
/// This trait defines the interface for a PostgreSQL connection pool.
#[async_trait]
pub trait PgPool: Send + Sync + 'static {
    /// Ping the database to check if the connection is still alive
    async fn ping(&self) -> Result<(), AppError>;

    /// Begin a new transaction
    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError>;

    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// PostgreSQL Transaction trait
///
/// This trait defines the interface for a PostgreSQL transaction.
#[async_trait]
pub trait PgTransaction: Send + Sync + 'static {
    /// Commit the transaction
    async fn commit(self: Box<Self>) -> Result<(), AppError>;

    /// Rollback the transaction
    async fn rollback(self: Box<Self>) -> Result<(), AppError>;
}
