use async_trait::async_trait;
use navius_db::error::{DatabaseError, DatabaseResult};
use navius_db::repository::{Entity, Repository};
use sqlx::postgres::PgRow;
use std::marker::PhantomData;
use tracing::{debug, instrument};

/// PostgreSQL repository implementation for entities
pub struct PgRepository<T: Entity> {
    /// The pool for database connections
    pool: sqlx::PgPool,
    /// The table name for this entity
    table_name: String,
    /// Phantom data for the entity type
    _phantom: PhantomData<T>,
}

impl<T: Entity> PgRepository<T> {
    /// Create a new PostgreSQL repository
    pub fn new(pool: sqlx::PgPool, table_name: impl Into<String>) -> Self {
        Self {
            pool,
            table_name: table_name.into(),
            _phantom: PhantomData,
        }
    }

    /// Get the table name for this repository
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[async_trait]
impl<T> Repository<T> for PgRepository<T>
where
    T: Entity + for<'a> sqlx::FromRow<'a, PgRow> + Send + Sync + 'static,
{
    /// Find an entity by its ID
    #[instrument(skip(self), fields(entity_type = std::any::type_name::<T>(), table_name = %self.table_name))]
    async fn find_by_id(&self, id: T::Id) -> DatabaseResult<Option<T>> {
        debug!("Finding entity by ID");

        let query = format!("SELECT * FROM {} WHERE id = $1", self.table_name);

        // Convert the ID to a parameter
        let id_value = serde_json::to_value(&id)
            .map_err(|e| DatabaseError::DataError(format!("Failed to serialize ID: {}", e)))?;

        // Execute the query
        let result = sqlx::query_as::<_, T>(&query)
            .bind(id_value)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query failed: {}", e)))?;

        Ok(result)
    }

    /// Find all entities
    #[instrument(skip(self), fields(entity_type = std::any::type_name::<T>(), table_name = %self.table_name))]
    async fn find_all(&self) -> DatabaseResult<Vec<T>> {
        debug!("Finding all entities");

        let query = format!("SELECT * FROM {}", self.table_name);

        // Execute the query
        let results = sqlx::query_as::<_, T>(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query failed: {}", e)))?;

        Ok(results)
    }

    /// Create a new entity
    #[instrument(skip(self, entity), fields(entity_type = std::any::type_name::<T>(), table_name = %self.table_name))]
    async fn create(&self, entity: &T) -> DatabaseResult<T> {
        debug!("Creating entity");

        // Serialize to a value that can be extracted into column/value pairs
        let entity_value = serde_json::to_value(entity)
            .map_err(|e| DatabaseError::DataError(format!("Failed to serialize entity: {}", e)))?;

        // Extract as an object
        let entity_map = match entity_value {
            serde_json::Value::Object(map) => map,
            _ => {
                return Err(DatabaseError::DataError(
                    "Entity did not serialize to an object".to_string(),
                ));
            }
        };

        // Build column and placeholder lists
        let mut columns = Vec::new();
        let mut placeholders = Vec::new();
        let mut values = Vec::new();
        let mut i = 1;

        for (key, value) in entity_map {
            // Skip 'id' column for inserts if it's null
            if key == "id" && value.is_null() {
                continue;
            }

            columns.push(key);
            placeholders.push(format!("${}", i));
            values.push(value);
            i += 1;
        }

        // Build the query
        let columns_str = columns.join(", ");
        let placeholders_str = placeholders.join(", ");
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            self.table_name, columns_str, placeholders_str
        );

        // Start building the query
        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // Add each parameter
        for value in values {
            query_builder = query_builder.bind(value);
        }

        // Execute the query
        let result = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Insert failed: {}", e)))?;

        Ok(result)
    }

    /// Update an existing entity
    #[instrument(skip(self, entity), fields(entity_type = std::any::type_name::<T>(), table_name = %self.table_name))]
    async fn update(&self, entity: &T) -> DatabaseResult<T> {
        debug!("Updating entity");

        // Serialize to a value that can be extracted into column/value pairs
        let entity_value = serde_json::to_value(entity)
            .map_err(|e| DatabaseError::DataError(format!("Failed to serialize entity: {}", e)))?;

        // Extract as an object
        let entity_map = match entity_value {
            serde_json::Value::Object(map) => map,
            _ => {
                return Err(DatabaseError::DataError(
                    "Entity did not serialize to an object".to_string(),
                ));
            }
        };

        // Extract ID
        let id = match entity_map.get("id") {
            Some(id) => id.clone(),
            None => {
                return Err(DatabaseError::ValidationError(
                    "Entity ID is required for update".to_string(),
                ));
            }
        };

        // Build SET clause
        let mut set_clauses = Vec::new();
        let mut values = Vec::new();
        let mut i = 1;

        for (key, value) in entity_map {
            // Skip ID for the SET clause
            if key == "id" {
                continue;
            }

            set_clauses.push(format!("{} = ${}", key, i));
            values.push(value);
            i += 1;
        }

        // Add ID as the last parameter
        values.push(id);

        // Build the query
        let set_clauses_str = set_clauses.join(", ");
        let query = format!(
            "UPDATE {} SET {} WHERE id = ${} RETURNING *",
            self.table_name, set_clauses_str, i
        );

        // Start building the query
        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // Add each parameter
        for value in values {
            query_builder = query_builder.bind(value);
        }

        // Execute the query
        let result = query_builder
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Update failed: {}", e)))?;

        Ok(result)
    }

    /// Delete an entity by its ID
    #[instrument(skip(self), fields(entity_type = std::any::type_name::<T>(), table_name = %self.table_name))]
    async fn delete_by_id(&self, id: T::Id) -> DatabaseResult<bool> {
        debug!("Deleting entity by ID");

        let query = format!("DELETE FROM {} WHERE id = $1", self.table_name);

        // Convert the ID to a parameter
        let id_value = serde_json::to_value(&id)
            .map_err(|e| DatabaseError::DataError(format!("Failed to serialize ID: {}", e)))?;

        // Execute the query
        let result = sqlx::query(&query)
            .bind(id_value)
            .execute(&self.pool)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Delete failed: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use sqlx::FromRow;

    #[derive(Debug, Serialize, Deserialize, FromRow, PartialEq)]
    struct TestEntity {
        id: i32,
        name: String,
        value: Option<i32>,
    }

    impl Entity for TestEntity {
        type Id = i32;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn set_id(&mut self, id: Self::Id) {
            self.id = id;
        }
    }

    // Note: These are mock tests that don't connect to a real database
    // Complete integration tests would be needed with a test database
    #[test]
    fn test_table_name() {
        let pool = sqlx::PgPool::builder().build_unchecked();
        let repo = PgRepository::<TestEntity>::new(pool, "test_entities");
        assert_eq!(repo.table_name(), "test_entities");
    }
}
