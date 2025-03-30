use async_trait::async_trait;
use navius_db::error::{DatabaseError, DatabaseResult};
use navius_db::pool::{DatabaseRowSet, DatabaseTransaction};
use sqlx::postgres::PgRow;
use tracing::{debug, error, instrument};

/// PostgreSQL transaction implementation that wraps SQLx transaction
pub struct PgTransaction {
    tx: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl PgTransaction {
    /// Create a new PostgreSQL transaction
    pub fn new(tx: sqlx::Transaction<'static, sqlx::Postgres>) -> Self {
        Self { tx: Some(tx) }
    }

    /// Validate that the savepoint name is valid for PostgreSQL
    fn validate_savepoint_name(&self, name: &str) -> DatabaseResult<()> {
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(DatabaseError::savepoint_error(format!(
                "Invalid savepoint name: {}. Only alphanumeric characters and underscores are allowed.",
                name
            )));
        }
        Ok(())
    }

    /// Get the underlying transaction, returning an error if it's been consumed
    fn get_tx(&mut self) -> DatabaseResult<&mut sqlx::Transaction<'static, sqlx::Postgres>> {
        if let Some(tx) = self.tx.as_mut() {
            Ok(tx)
        } else {
            Err(DatabaseError::TransactionFinished)
        }
    }
}

/// Implementation of DatabaseTransaction trait
#[async_trait]
impl DatabaseTransaction for PgTransaction {
    #[instrument(skip(self, query, params), level = "debug")]
    async fn execute<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<u64> {
        let tx = self.get_tx()?;

        let query_result = sqlx::query_with(query, params)
            .execute(&mut **tx)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(query_result.rows_affected())
    }

    #[instrument(skip(self, query, params), level = "debug")]
    async fn query<'a>(
        &mut self,
        query: &str,
        params: &[&'a (dyn sqlx::Encode<'a, sqlx::Postgres> + Sync)],
    ) -> DatabaseResult<Box<dyn DatabaseRowSet>> {
        let tx = self.get_tx()?;

        let rows = sqlx::query_with(query, params)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| DatabaseError::QueryError(format!("Query execution failed: {}", e)))?;

        Ok(Box::new(PgRowSet::new(rows)))
    }

    #[instrument(skip(self), level = "debug")]
    async fn commit(mut self: Box<Self>) -> DatabaseResult<()> {
        let tx = self.tx.take().ok_or(DatabaseError::TransactionFinished)?;

        tx.commit().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to commit transaction: {}", e))
        })
    }

    #[instrument(skip(self), level = "debug")]
    async fn rollback(mut self: Box<Self>) -> DatabaseResult<()> {
        let tx = self.tx.take().ok_or(DatabaseError::TransactionFinished)?;

        tx.rollback().await.map_err(|e| {
            DatabaseError::TransactionError(format!("Failed to rollback transaction: {}", e))
        })
    }

    #[instrument(skip(self), level = "debug")]
    async fn savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let tx = self.get_tx()?;

        // Create a savepoint in PostgreSQL
        let savepoint_query = format!("SAVEPOINT {}", name);
        sqlx::query(&savepoint_query)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!(
                    "Failed to create savepoint {}: {}",
                    name, e
                ))
            })?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn rollback_to_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let tx = self.get_tx()?;

        // Roll back to a savepoint in PostgreSQL
        let rollback_query = format!("ROLLBACK TO SAVEPOINT {}", name);
        sqlx::query(&rollback_query)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!(
                    "Failed to rollback to savepoint {}: {}",
                    name, e
                ))
            })?;

        Ok(())
    }

    #[instrument(skip(self), level = "debug")]
    async fn release_savepoint(&mut self, name: &str) -> DatabaseResult<()> {
        self.validate_savepoint_name(name)?;

        let tx = self.get_tx()?;

        // Release a savepoint in PostgreSQL
        let release_query = format!("RELEASE SAVEPOINT {}", name);
        sqlx::query(&release_query)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                DatabaseError::savepoint_error(format!(
                    "Failed to release savepoint {}: {}",
                    name, e
                ))
            })?;

        Ok(())
    }
}

/// Implementation of a row set for PostgreSQL results
struct PgRowSet {
    rows: Vec<sqlx::postgres::PgRow>,
    pos: usize,
}

impl PgRowSet {
    /// Create a new row set from a collection of rows
    fn new(rows: Vec<sqlx::postgres::PgRow>) -> Self {
        Self { rows, pos: 0 }
    }
}

impl DatabaseRowSet for PgRowSet {
    fn next(&mut self) -> DatabaseResult<Option<PgRow>> {
        if self.pos < self.rows.len() {
            let row = PgRow::new(self.rows[self.pos].clone());
            self.pos += 1;
            Ok(Some(row))
        } else {
            Ok(None)
        }
    }
}

impl std::fmt::Debug for PgRowSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgRowSet")
            .field("row_count", &self.rows.len())
            .field("position", &self.pos)
            .finish()
    }
}

/// PostgreSQL row wrapper
pub struct PgRow {
    inner: sqlx::postgres::PgRow,
}

impl PgRow {
    /// Create a new row wrapper
    pub fn new(row: sqlx::postgres::PgRow) -> Self {
        Self { inner: row }
    }

    /// Get a value by column name
    pub fn get<T>(&self, name: &str) -> DatabaseResult<T>
    where
        T: for<'a> sqlx::decode::Decode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        self.inner
            .try_get(name)
            .map_err(|e| DatabaseError::DataError(format!("Failed to get column {}: {}", name, e)))
    }

    /// Get a value by column index
    pub fn get_by_index<T>(&self, index: usize) -> DatabaseResult<T>
    where
        T: for<'a> sqlx::decode::Decode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
    {
        self.inner.try_get(index).map_err(|e| {
            DatabaseError::DataError(format!("Failed to get column at index {}: {}", index, e))
        })
    }
}

impl std::fmt::Debug for PgRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgRow").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock tests to be expanded with more complete integration tests
    #[test]
    fn test_validate_savepoint_name() {
        let tx = PgTransaction { tx: None };

        // Valid names
        assert!(tx.validate_savepoint_name("valid").is_ok());
        assert!(tx.validate_savepoint_name("valid_name_123").is_ok());
        assert!(tx.validate_savepoint_name("_valid").is_ok());

        // Invalid names
        assert!(tx.validate_savepoint_name("invalid-name").is_err());
        assert!(tx.validate_savepoint_name("invalid.name").is_err());
        assert!(tx.validate_savepoint_name("invalid;name").is_err());
    }
}
