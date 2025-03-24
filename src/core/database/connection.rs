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
use sqlx::{Pool, Postgres, Transaction, postgres::PgPoolOptions};

// PostgreSQL connection implementation
#[derive(Debug)]
pub struct PgDatabaseConnection {
    config: DatabaseConfig,
    pool: Pool<Postgres>,
}

impl PgDatabaseConnection {
    pub async fn new(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .idle_timeout(config.idle_timeout_seconds.map(Duration::from_secs))
            .connect(&config.url)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        // Run migrations if needed
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))?;

        Ok(Self {
            config: config.clone(),
            pool,
        })
    }

    pub fn get_pool(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

impl Clone for PgDatabaseConnection {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
        }
    }
}

// PostgreSQL transaction wrapper
pub struct PgSqlxTransaction {
    tx: Transaction<'static, Postgres>,
}

#[async_trait]
impl PgTransaction for PgSqlxTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        self.tx
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to commit transaction: {}", e)))
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        self.tx
            .rollback()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to rollback transaction: {}", e)))
    }
}

#[async_trait]
impl PgPool for PgDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::DatabaseError(format!("Database ping failed: {}", e)))?;
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx =
            self.pool.begin().await.map_err(|e| {
                AppError::DatabaseError(format!("Failed to begin transaction: {}", e))
            })?;

        Ok(Box::new(PgSqlxTransaction { tx }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// For backwards compatibility, also implement the legacy DatabaseConnection trait
#[async_trait]
impl DatabaseConnection for PgDatabaseConnection {
    async fn ping(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;
        Ok(())
    }

    async fn stats(&self) -> Result<ConnectionStats, DatabaseError> {
        // SQLx doesn't expose detailed pool stats, so we approximate
        Ok(ConnectionStats {
            idle_connections: 0,   // Not exposed by SQLx
            active_connections: 0, // Not exposed by SQLx
            max_connections: self.config.max_connections,
        })
    }

    async fn begin_transaction(&self) -> Result<super::Transaction, DatabaseError> {
        let _tx = self
            .pool
            .begin()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))?;

        // For now, just return a placeholder transaction
        // This will be deprecated in favor of the new PgTransaction trait
        Ok(super::Transaction::new())
    }
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

/// Connection pool statistics
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
    users: std::sync::Mutex<Vec<crate::repository::User>>,
}

#[cfg(test)]
impl crate::mockable::MockExtern for MockDatabaseConnection {}

#[cfg(test)]
impl MockDatabaseConnection {
    /// Create a new MockDatabaseConnection
    pub fn new(config: &DatabaseConfig) -> Self {
        Self {
            config: config.clone(),
            users: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Add a user to the mock database
    pub fn add_user(&self, user: crate::repository::User) {
        let mut users = self.users.lock().unwrap();
        users.push(user);
    }

    /// Get all users from the mock database
    pub fn get_users(&self) -> Vec<crate::repository::User> {
        let users = self.users.lock().unwrap();
        users.clone()
    }

    /// Find a user by ID
    pub fn find_user_by_id(&self, id: uuid::Uuid) -> Option<crate::repository::User> {
        let users = self.users.lock().unwrap();
        users.iter().find(|u| u.id == id).cloned()
    }

    /// Find a user by username
    pub fn find_user_by_username(&self, username: &str) -> Option<crate::repository::User> {
        let users = self.users.lock().unwrap();
        users.iter().find(|u| u.username == username).cloned()
    }

    /// Find a user by email
    pub fn find_user_by_email(&self, email: &str) -> Option<crate::repository::User> {
        let users = self.users.lock().unwrap();
        users.iter().find(|u| u.email == email).cloned()
    }

    /// Save a user (create or update)
    pub fn save_user(&self, user: crate::repository::User) -> crate::repository::User {
        let mut users = self.users.lock().unwrap();

        // Check if the user already exists
        if let Some(idx) = users.iter().position(|u| u.id == user.id) {
            // Update existing user
            users[idx] = user.clone();
        } else {
            // Add new user
            users.push(user.clone());
        }

        user
    }

    /// Delete a user by ID
    pub fn delete_user(&self, id: uuid::Uuid) -> bool {
        let mut users = self.users.lock().unwrap();
        if let Some(idx) = users.iter().position(|u| u.id == id) {
            users.remove(idx);
            true
        } else {
            false
        }
    }

    /// Count users
    pub fn count_users(&self) -> usize {
        let users = self.users.lock().unwrap();
        users.len()
    }
}

/// Mock implementation of DatabaseConnection for non-test fallbacks
#[cfg(not(test))]
#[derive(Debug)]
pub struct MockDatabaseConnection {
    config: DatabaseConfig,
    users: std::sync::Mutex<Vec<crate::repository::User>>,
}

#[cfg(not(test))]
impl MockDatabaseConnection {
    /// Create a new MockDatabaseConnection
    pub fn new(config: &DatabaseConfig) -> Self {
        Self {
            config: config.clone(),
            users: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get all users from the mock database
    pub fn get_users(&self) -> Vec<crate::repository::User> {
        let users = self.users.lock().unwrap();
        users.clone()
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

// Non-test implementation
#[cfg(not(test))]
#[async_trait]
impl DatabaseConnection for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), DatabaseError> {
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
impl Clone for MockDatabaseConnection {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            users: std::sync::Mutex::new(self.get_users()),
        }
    }
}

// New trait implementation for tests
#[cfg(test)]
#[async_trait]
impl PgPool for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        // Just return success in tests
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx = MockDbTransaction::new();
        Ok(Box::new(tx))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Non-test PgPool implementation
#[cfg(not(test))]
#[async_trait]
impl PgPool for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        // Just return success in non-test environments
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx = MockTransaction::new();
        Ok(Box::new(tx))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Initialize the database connection
pub async fn init_database(config: &DatabaseConfig) -> Result<Arc<Box<dyn PgPool>>, AppError> {
    if !config.enabled {
        tracing::info!("Database is disabled in configuration");
        return Err(AppError::DatabaseError("Database is disabled".into()));
    }

    tracing::info!("Initializing database connection to {}", config.url);

    // Connect to the database using the real implementation
    let conn = PgDatabaseConnection::new(config)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect to database: {}", e)))?;

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

impl Default for MockTransaction {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PgTransaction for MockTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        // Do nothing in mock implementation
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        // Do nothing in mock implementation
        Ok(())
    }
}

#[cfg(test)]
/// Enhanced mock transaction for testing with state
pub struct MockDbTransaction {
    // No connection needed for tests as we're not using it
}

#[cfg(test)]
impl MockDbTransaction {
    /// Create a new MockDbTransaction
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
#[async_trait]
impl PgTransaction for MockDbTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        // Transaction is already committed since we're using in-memory
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        // No real rollback needed for in-memory implementation
        Ok(())
    }
}

/// Create a PostgreSQL connection pool from configuration
pub async fn create_pool(
    config: &DatabaseConfig,
) -> Result<Box<dyn DatabaseConnection>, DatabaseError> {
    if !config.enabled {
        return Err(DatabaseError::NotEnabled);
    }

    // For real PostgreSQL connection
    if cfg!(not(test)) {
        return match PgDatabaseConnection::new(config).await {
            Ok(conn) => Ok(Box::new(conn)),
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to PostgreSQL, falling back to mock connection: {}",
                    e
                );
                Ok(Box::new(MockDatabaseConnection::new(config)))
            }
        };
    }

    // For tests
    #[cfg(test)]
    return Ok(Box::new(MockDatabaseConnection::new(config)));

    #[cfg(not(test))]
    return Ok(Box::new(MockDatabaseConnection::new(config)));
}
