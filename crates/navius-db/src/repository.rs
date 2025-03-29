use crate::connection::DatabaseConnectionManager;
use crate::error::{DatabaseError, DatabaseResult};
use crate::transaction::Transaction;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::marker::PhantomData;
use tracing::{debug, instrument};
use uuid::Uuid;

/// Entity trait for database entities
pub trait Entity: Serialize + DeserializeOwned + Clone + Debug + Send + Sync + 'static {
    /// Get the entity ID
    fn id(&self) -> Uuid;

    /// Set the entity ID
    fn set_id(&mut self, id: Uuid);

    /// Get the table name for this entity
    fn table_name() -> &'static str;
}

/// Repository trait for database repositories
#[async_trait]
pub trait Repository<T: Entity>: Send + Sync + 'static {
    /// Find an entity by ID
    async fn find_by_id(&self, id: Uuid) -> DatabaseResult<T>;

    /// Find all entities
    async fn find_all(&self) -> DatabaseResult<Vec<T>>;

    /// Save an entity
    async fn save(&self, entity: &T) -> DatabaseResult<T>;

    /// Delete an entity
    async fn delete(&self, id: Uuid) -> DatabaseResult<()>;

    /// Find entities by a custom query
    #[cfg(feature = "postgres")]
    async fn find_by_query(
        &self,
        query: &str,
        params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Vec<T>>;

    /// Count all entities
    async fn count(&self) -> DatabaseResult<i64>;
}

/// Base repository implementation
pub struct BaseRepository<T: Entity> {
    /// Database connection manager
    db: DatabaseConnectionManager,
    /// Phantom type for entity
    _phantom: PhantomData<T>,
}

impl<T: Entity> BaseRepository<T> {
    /// Create a new base repository
    pub fn new(db: DatabaseConnectionManager) -> Self {
        Self {
            db,
            _phantom: PhantomData,
        }
    }

    /// Get the database connection manager
    pub fn db(&self) -> &DatabaseConnectionManager {
        &self.db
    }

    /// Execute a function within a transaction
    pub async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: for<'c> FnOnce(Transaction<'c>) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
        E: From<DatabaseError> + Send + 'static,
    {
        self.db.transaction(f).await
    }
}

/// Base SQL repository implementation
#[cfg(feature = "postgres")]
pub struct SqlRepository<T: Entity> {
    base: BaseRepository<T>,
}

#[cfg(feature = "postgres")]
impl<T: Entity> SqlRepository<T> {
    /// Create a new SQL repository
    pub fn new(db: DatabaseConnectionManager) -> Self {
        Self {
            base: BaseRepository::new(db),
        }
    }
}

#[cfg(feature = "postgres")]
#[async_trait]
impl<T: Entity + for<'de> serde::Deserialize<'de>> Repository<T> for SqlRepository<T> {
    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> DatabaseResult<T> {
        debug!("Finding entity by ID: {}", id);

        let _query = format!("SELECT * FROM {} WHERE id = $1", T::table_name());
        let _conn = self.base.db().get_connection().await?;

        // This is just a placeholder implementation
        // In a real implementation, we would execute the query and convert the result
        // to the entity type using serde

        Err(DatabaseError::NotFoundError(format!(
            "Entity not found: {} with ID {}",
            T::table_name(),
            id
        )))
    }

    #[instrument(skip(self))]
    async fn find_all(&self) -> DatabaseResult<Vec<T>> {
        debug!("Finding all entities");

        let _query = format!("SELECT * FROM {}", T::table_name());
        let _conn = self.base.db().get_connection().await?;

        // This is just a placeholder implementation

        Ok(Vec::new())
    }

    #[instrument(skip(self, entity))]
    async fn save(&self, entity: &T) -> DatabaseResult<T> {
        let id = entity.id();
        debug!("Saving entity with ID: {:?}", id);

        let mut entity_clone = entity.clone();

        // If ID is nil, generate a new one
        if id == Uuid::nil() {
            let new_id = Uuid::new_v4();
            entity_clone.set_id(new_id);
            debug!("Generated new ID for entity: {}", new_id);
        }

        // This is just a placeholder implementation

        Ok(entity_clone)
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> DatabaseResult<()> {
        debug!("Deleting entity with ID: {}", id);

        let _query = format!("DELETE FROM {} WHERE id = $1", T::table_name());
        let _conn = self.base.db().get_connection().await?;

        // This is just a placeholder implementation

        Ok(())
    }

    #[instrument(skip(self, _query, _params))]
    async fn find_by_query(
        &self,
        _query: &str,
        _params: &[&(dyn sqlx::Encode<'_, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Vec<T>> {
        debug!("Finding entities by custom query");

        let _conn = self.base.db().get_connection().await?;

        // This is just a placeholder implementation

        Ok(Vec::new())
    }

    #[instrument(skip(self))]
    async fn count(&self) -> DatabaseResult<i64> {
        debug!("Counting entities");

        let _query = format!("SELECT COUNT(*) FROM {}", T::table_name());
        let _conn = self.base.db().get_connection().await?;

        // This is just a placeholder implementation

        Ok(0)
    }
}
