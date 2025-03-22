//! Repository module for database interactions
//!
//! This module provides interfaces and implementations for interacting with the database.
//! It follows the repository pattern where each entity type has its own repository.

pub mod models;
pub mod user;

#[cfg(test)]
mod tests;

pub use models::User;
pub use user::UserRepository;

use async_trait::async_trait;
use std::sync::Arc;

use crate::core::database::{PgPool, PgTransaction};
use crate::core::error::AppError;

/// Repository trait that defines common operations for all repositories
#[async_trait]
pub trait Repository<T, ID>: Send + Sync
where
    T: Send + Sync + 'static,
    ID: Send + Sync + 'static,
{
    /// Find an entity by its ID
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, AppError>;

    /// Save an entity (create or update)
    async fn save(&self, entity: T) -> Result<T, AppError>;

    /// Delete an entity by its ID
    async fn delete(&self, id: ID) -> Result<bool, AppError>;

    /// Find all entities
    async fn find_all(&self) -> Result<Vec<T>, AppError>;
}

/// Base repository that provides common functionality for all repositories
pub struct BaseRepository {
    db_pool: Arc<Box<dyn PgPool>>,
}

impl BaseRepository {
    /// Create a new repository with a database pool
    pub fn new(db_pool: Arc<Box<dyn PgPool>>) -> Self {
        Self { db_pool }
    }

    /// Get a reference to the database pool
    pub fn db_pool(&self) -> &dyn PgPool {
        &**self.db_pool
    }

    /// Begin a transaction
    pub async fn begin_transaction(&self) -> Result<Box<dyn PgTransaction>, AppError> {
        self.db_pool.begin().await
    }
}
