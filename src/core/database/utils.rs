use crate::core::error::AppError;
use futures::future::BoxFuture;
use sqlx::{Error as SqlxError, Pool, Postgres, Transaction};
use std::future::Future;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

/// Execute a function within a database transaction
///
/// This function will:
/// 1. Begin a transaction
/// 2. Execute the provided function with the transaction
/// 3. If the function succeeds, commit the transaction
/// 4. If the function fails, rollback the transaction
///
/// # Arguments
/// * `pool` - The database connection pool
/// * `f` - The function to execute within the transaction
///
/// # Returns
/// The result of the function execution
pub async fn with_transaction<F, T, E>(pool: &Pool<Postgres>, f: F) -> Result<T, E>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Postgres>) -> BoxFuture<'a, Result<T, E>>,
    E: From<sqlx::Error>,
{
    let mut tx = pool.begin().await?;
    let result = f(&mut tx).await;
    match result {
        Ok(value) => {
            tx.commit().await?;
            Ok(value)
        }
        Err(e) => {
            tx.rollback().await?;
            Err(e)
        }
    }
}

/// Generate a new UUID
///
/// Utility function to generate a new UUID v4 for entity identifiers
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Helper to extract error message from database errors
///
/// Creates a consistent error message format for database operations
pub fn db_error_message(operation: &str, entity: &str, error: SqlxError) -> AppError {
    AppError::DatabaseError(format!("Failed to {} {} - {}", operation, entity, error))
}

/// Check if a value exists in a table
///
/// # Arguments
/// * `pool` - The database connection pool
/// * `table` - The table to check
/// * `column` - The column to check
/// * `value` - The value to check for
///
/// # Returns
/// `true` if the value exists, `false` otherwise
pub async fn exists<T>(
    pool: &Pool<Postgres>,
    table: &str,
    column: &str,
    value: T,
) -> Result<bool, sqlx::Error>
where
    T: sqlx::Type<Postgres> + sqlx::Encode<'static, Postgres> + Send + 'static,
{
    let query_str = format!(
        "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = $1)",
        table, column
    );

    // Convert to a static str
    let query_static = Box::leak(query_str.into_boxed_str());

    sqlx::query_scalar::<_, bool>(query_static)
        .bind(value)
        .fetch_one(pool)
        .await
}
