use navius_db::error::DatabaseError;
use thiserror::Error;

/// PostgreSQL-specific error types
#[derive(Error, Debug)]
pub enum PgError {
    /// SQL error
    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Pool error
    #[error("Pool error: {0}")]
    Pool(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Query error
    #[error("Query error: {0}")]
    Query(String),

    /// Parameter binding error
    #[error("Parameter binding error: {0}")]
    ParameterBinding(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<PgError> for DatabaseError {
    fn from(err: PgError) -> Self {
        match err {
            PgError::Sql(e) => DatabaseError::ExecutionError(e.to_string()),
            PgError::Connection(msg) => DatabaseError::ConnectionError(msg),
            PgError::Pool(msg) => DatabaseError::ConnectionError(msg),
            PgError::Configuration(msg) => DatabaseError::ConfigError(msg),
            PgError::Migration(msg) => DatabaseError::MigrationError(msg),
            PgError::Query(msg) => DatabaseError::QueryError(msg),
            PgError::ParameterBinding(msg) => DatabaseError::ParameterError(msg),
            PgError::Transaction(msg) => DatabaseError::TransactionError(msg),
            PgError::TypeConversion(msg) => DatabaseError::ConversionError(msg),
            PgError::Other(msg) => DatabaseError::UnknownError(msg),
        }
    }
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(e) => {
                DatabaseError::ExecutionError(format!("Database error: {}", e))
            }
            sqlx::Error::Io(e) => DatabaseError::ConnectionError(format!("IO error: {}", e)),
            sqlx::Error::RowNotFound => DatabaseError::NotFoundError("Row not found".to_string()),
            sqlx::Error::ColumnNotFound(col) => {
                DatabaseError::QueryError(format!("Column not found: {}", col))
            }
            sqlx::Error::PoolTimedOut => {
                DatabaseError::ConnectionError("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                DatabaseError::ConnectionError("Connection pool closed".to_string())
            }
            sqlx::Error::WorkerCrashed => {
                DatabaseError::ConnectionError("Database worker crashed".to_string())
            }
            _ => DatabaseError::UnknownError(format!("Unknown database error: {}", err)),
        }
    }
}
