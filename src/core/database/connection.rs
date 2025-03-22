//! Database connection management
//!
//! This module handles PostgreSQL connection pooling and management

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
use crate::mockable::MockExtern;
use async_trait::async_trait;

use crate::core::config::app_config::DatabaseConfig;
use crate::core::error::AppError;

use super::error::DatabaseError;
use super::{PgPool, PgTransaction};

// For non-test environments, we need a simple DatabaseConnection impl
#[cfg(not(test))]
#[derive(Debug)]
pub struct SimpleDatabaseConnection {
    config: DatabaseConfig,
}

#[cfg(not(test))]
impl SimpleDatabaseConnection {
    pub fn new(config: &DatabaseConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

#[cfg(not(test))]
#[async_trait]
impl DatabaseConnection for SimpleDatabaseConnection {
    async fn ping(&self) -> Result<(), DatabaseError> {
        // Just return success in non-test environments
        Ok(())
    }

    async fn stats(&self) -> Result<ConnectionStats, DatabaseError> {
        Ok(ConnectionStats {
            idle_connections: 0,
            active_connections: 0,
            max_connections: self.config.max_connections,
        })
    }

    async fn begin_transaction(&self) -> Result<super::Transaction, DatabaseError> {
        Ok(super::Transaction::new())
    }
}

#[cfg(not(test))]
#[async_trait]
impl PgPool for SimpleDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx = MockTransaction::new();
        Ok(Box::new(tx))
    }
}

/// Legacy DatabaseConnection trait - will be deprecated
/// Create a PostgreSQL connection pool from configuration
pub async fn create_pool(
    config: &DatabaseConfig,
) -> Result<Box<dyn DatabaseConnection>, DatabaseError> {
    if !config.enabled {
        return Err(DatabaseError::NotEnabled);
    }

    // This is a placeholder until we add actual sqlx dependency
    // and implement the real connection pooling
    #[cfg(test)]
    return Ok(Box::new(MockDatabaseConnection::new(config)));

    #[cfg(not(test))]
    return Ok(Box::new(SimpleDatabaseConnection::new(config)));
}

/// Legacy DatabaseConnection trait - will be deprecated
#[async_trait]
pub trait DatabaseConnection: Send + Sync + Debug {
    /// Check if the connection is healthy
    async fn ping(&self) -> Result<(), DatabaseError>;

    /// Get connection statistics
    async fn stats(&self) -> Result<ConnectionStats, DatabaseError>;

    /// Begin a new transaction
    async fn begin_transaction(&self) -> Result<super::Transaction, DatabaseError>;
}

/// Connection statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of idle connections in the pool
    pub idle_connections: u32,

    /// Number of active connections from the pool
    pub active_connections: u32,

    /// Maximum pool size
    pub max_connections: u32,
}

/// Mock implementation of DatabaseConnection for testing
#[cfg(test)]
#[derive(Debug)]
pub struct MockDatabaseConnection {
    config: DatabaseConfig,
}

#[cfg(test)]
impl crate::mockable::MockExtern for MockDatabaseConnection {}

#[cfg(test)]
impl MockDatabaseConnection {
    /// Create a new MockDatabaseConnection
    pub fn new(config: &DatabaseConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

// Legacy trait implementation - will be deprecated
#[cfg(test)]
#[async_trait]
impl DatabaseConnection for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), DatabaseError> {
        // Simulate a ping with a small delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    async fn stats(&self) -> Result<ConnectionStats, DatabaseError> {
        Ok(ConnectionStats {
            idle_connections: 5,
            active_connections: 2,
            max_connections: self.config.max_connections,
        })
    }

    async fn begin_transaction(&self) -> Result<super::Transaction, DatabaseError> {
        // Create a mock transaction
        Ok(super::Transaction::new())
    }
}

// New trait implementation that will replace the old one
#[cfg(test)]
#[async_trait]
impl PgPool for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        // Simulate a ping with a small delay
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        // Create a mock transaction
        let tx = MockTransaction::new();
        Ok(Box::new(tx))
    }
}

/// Initialize the database connection
pub async fn init_database(config: &DatabaseConfig) -> Result<Arc<Box<dyn PgPool>>, AppError> {
    if !config.enabled {
        tracing::info!("Database is disabled in configuration");
        return Err(AppError::DatabaseError("Database is disabled".into()));
    }

    tracing::info!("Initializing database connection to {}", config.url);

    // In a real implementation, we would connect to the database here
    // For example, using sqlx:
    // let pool = sqlx::PgPool::connect(&config.url).await
    //    .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

    // For now, we'll use a mock connection
    #[cfg(test)]
    let conn = MockDatabaseConnection::new(config);
    #[cfg(not(test))]
    let conn = SimpleDatabaseConnection::new(config);

    // Return an Arc boxed as PgPool trait object
    Ok(Arc::new(Box::new(conn) as Box<dyn PgPool>))
}

/// Ping the database to check if the connection is still alive
pub async fn ping_database(pool: &dyn PgPool) -> Result<(), AppError> {
    pool.ping().await
}

/// Mock transaction for testing
pub struct MockTransaction {
    // In a real implementation, this would hold transaction state
}

impl MockTransaction {
    /// Create a new mock transaction
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl PgTransaction for MockTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        // This is a mock, so we'll just return success
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        // This is a mock, so we'll just return success
        Ok(())
    }
}
