//! Database connection management
//!
//! This module handles PostgreSQL connection pooling and management

use std::fmt::Debug;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;

#[cfg(test)]
use crate::mockable::MockExtern;
use crate::repository::models::User;
use async_trait::async_trait;

use crate::core::config::app_config::DatabaseConfig;
use crate::core::error::error_types::AppError;

use super::error::DatabaseError;
use super::{PgPool, PgTransaction};
use sqlx::{Pool, Postgres, Transaction, postgres::PgPoolOptions};
use std::any::Any;

/// PostgreSQL transaction implementation
pub struct PgTransactionImpl {
    transaction: sqlx::Transaction<'static, sqlx::Postgres>,
}

#[async_trait]
impl PgTransaction for PgTransactionImpl {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        self.transaction
            .commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        self.transaction
            .rollback()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }
}

/// PostgreSQL connection implementation
pub struct PgDatabaseConnection {
    pool: Arc<Pool<Postgres>>,
    config: DatabaseConfig,
}

impl PgDatabaseConnection {
    /// Create a new PgDatabaseConnection with a pool
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool: Arc::new(pool),
            config: DatabaseConfig::default(),
        }
    }

    /// Connect to the database using configuration
    pub async fn connect(config: DatabaseConfig) -> Result<Self, AppError> {
        let pool = create_connection_pool(&config).await?;
        Ok(Self {
            pool: Arc::new(pool),
            config,
        })
    }
}

#[async_trait]
impl PgPool for PgDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map(|_| ())
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let transaction = self
            .pool
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(Box::new(PgTransactionImpl { transaction }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Mock database connection for testing
#[derive(Debug)]
pub struct MockDatabaseConnection {
    config: DatabaseConfig,
    users: Arc<RwLock<Vec<User>>>,
}

impl Clone for MockDatabaseConnection {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            users: self.users.clone(),
        }
    }
}

impl MockDatabaseConnection {
    /// Create a new mock connection
    pub fn new(config: DatabaseConfig) -> Self {
        Self {
            config,
            users: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a user to the mock database
    pub fn add_user(&self, user: User) {
        let mut users = self.users.write().unwrap();
        users.push(user);
    }

    /// Get all users from the mock database
    pub fn get_users(&self) -> Vec<User> {
        let users = self.users.read().unwrap();
        users.to_vec()
    }

    /// Find a user by ID
    pub fn find_user_by_id(&self, id: uuid::Uuid) -> Option<User> {
        let users = self.users.read().unwrap();
        users.iter().find(|u| u.id == id).cloned()
    }

    /// Find a user by username
    pub fn find_user_by_username(&self, username: &str) -> Option<User> {
        let users = self.users.read().unwrap();
        users.iter().find(|u| u.username == username).cloned()
    }

    /// Find a user by email
    pub fn find_user_by_email(&self, email: &str) -> Option<User> {
        let users = self.users.read().unwrap();
        users.iter().find(|u| u.email == email).cloned()
    }

    /// Save a user (create or update)
    pub fn save_user(&self, user: User) -> User {
        let mut users = self.users.write().unwrap();

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
        let mut users = self.users.write().unwrap();
        if let Some(idx) = users.iter().position(|u| u.id == id) {
            users.remove(idx);
            true
        } else {
            false
        }
    }

    /// Count users
    pub fn count_users(&self) -> usize {
        let users = self.users.read().unwrap();
        users.len()
    }
}

#[async_trait]
impl PgPool for MockDatabaseConnection {
    async fn ping(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        Ok(Box::new(MockTransaction {}))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Mock transaction for testing
#[derive(Debug, Clone)]
pub struct MockTransaction {}

#[async_trait]
impl PgTransaction for MockTransaction {
    async fn commit(self: Box<Self>) -> Result<(), AppError> {
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), AppError> {
        Ok(())
    }
}

/// Initialize the database connection
pub async fn init_database(config: DatabaseConfig) -> Result<Arc<dyn PgPool>, AppError> {
    if cfg!(test) {
        Ok(Arc::new(MockDatabaseConnection::new(config)))
    } else {
        let connection = PgDatabaseConnection::connect(config).await?;
        Ok(Arc::new(connection))
    }
}

/// Create a PostgreSQL connection pool from configuration
pub async fn create_pool(config: &DatabaseConfig) -> Result<Arc<dyn PgPool>, DatabaseError> {
    if !config.enabled {
        return Err(DatabaseError::NotEnabled);
    }

    // For real PostgreSQL connection
    if cfg!(not(test)) {
        return match PgDatabaseConnection::connect(config.clone()).await {
            Ok(conn) => Ok(Arc::new(conn)),
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to PostgreSQL, falling back to mock connection: {}",
                    e
                );
                let pool = sqlx::postgres::PgPool::connect(&config.url)
                    .await
                    .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
                Ok(Arc::new(MockDatabaseConnection::new(config.clone())))
            }
        };
    }

    // For tests, create a mock connection
    let pool = sqlx::postgres::PgPool::connect(&config.url)
        .await
        .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;
    Ok(Arc::new(MockDatabaseConnection::new(config.clone())))
}

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    /// Number of idle connections in the pool
    pub idle_connections: u32,

    /// Number of active connections in the pool
    pub active_connections: u32,

    /// Maximum pool size
    pub max_connections: u32,
}

/// Create a Postgres connection pool from a database configuration
pub async fn create_connection_pool(config: &DatabaseConfig) -> Result<Pool<Postgres>, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(Duration::from_secs(config.connect_timeout_seconds))
        .idle_timeout(config.idle_timeout_seconds.map(Duration::from_secs))
        .connect(&config.url)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Failed to connect: {}", e)))?;

    sqlx::migrate!("./src/app/database/migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::DatabaseError(format!("Migration failed: {}", e)))?;

    Ok(pool)
}

/// Implement PgPool trait for SQLx Pool<Postgres>
#[async_trait]
impl PgPool for Pool<Postgres> {
    async fn ping(&self) -> Result<(), AppError> {
        self.acquire()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to ping database: {}", e)))?;
        Ok(())
    }

    async fn begin(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        let tx = self
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(format!("Failed to begin transaction: {}", e)))?;

        Ok(Box::new(PgTransactionWrapper { tx }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Wrapper for SQLx Transaction<'static, Postgres> to implement PgTransaction
pub struct PgTransactionWrapper {
    tx: sqlx::Transaction<'static, Postgres>,
}

#[async_trait]
impl PgTransaction for PgTransactionWrapper {
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
