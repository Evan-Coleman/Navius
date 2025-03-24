use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

/// Generic repository trait that provides standard CRUD operations
/// for entity-related database interactions.
///
/// This trait is designed to be implemented by concrete repositories
/// that need to perform common operations on entities.
#[async_trait]
pub trait EntityRepository<T, E: Error + Send + Sync>: Send + Sync {
    /// Retrieve all entities
    async fn find_all(&self) -> Result<Vec<T>, E>;

    /// Find entity by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, E>;

    /// Create a new entity
    async fn create(&self, entity: T) -> Result<T, E>;

    /// Update an existing entity
    async fn update(&self, entity: T) -> Result<T, E>;

    /// Delete an entity by ID
    async fn delete(&self, id: Uuid) -> Result<bool, E>;
}
