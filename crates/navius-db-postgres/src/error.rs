use navius_db::error::DatabaseError;
use sqlx::postgres::PgDatabaseError;
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

/// A wrapper around sqlx::Error for Postgres
#[derive(Debug)]
pub struct PostgresError(sqlx::Error);

impl From<sqlx::Error> for PostgresError {
    fn from(err: sqlx::Error) -> Self {
        Self(err)
    }
}

impl From<PostgresError> for DatabaseError {
    fn from(err: PostgresError) -> Self {
        match err.0 {
            sqlx::Error::Database(ref e) => {
                if let Some(pg_err) = e.try_downcast_ref::<PgDatabaseError>() {
                    match pg_err.code() {
                        "23505" => DatabaseError::UniqueViolation {
                            table: pg_err.table().unwrap_or("unknown").to_string(),
                            column: pg_err.column().unwrap_or("unknown").to_string(),
                            message: pg_err.message().to_string(),
                        },
                        "23503" => DatabaseError::ForeignKeyViolation {
                            table: pg_err.table().unwrap_or("unknown").to_string(),
                            column: pg_err.column().unwrap_or("unknown").to_string(),
                            message: pg_err.message().to_string(),
                        },
                        "23502" => DatabaseError::NotNullViolation {
                            table: pg_err.table().unwrap_or("unknown").to_string(),
                            column: pg_err.column().unwrap_or("unknown").to_string(),
                            message: pg_err.message().to_string(),
                        },
                        _ => DatabaseError::Other(pg_err.message().to_string()),
                    }
                } else {
                    DatabaseError::Other(e.to_string())
                }
            }
            sqlx::Error::RowNotFound => DatabaseError::NotFound("Row not found".to_string()),
            sqlx::Error::PoolTimedOut => {
                DatabaseError::ConnectionTimeout("Pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => DatabaseError::ConnectionError("Pool closed".to_string()),
            sqlx::Error::WorkerCrashed => {
                DatabaseError::ConnectionError("Worker crashed".to_string())
            }
            _ => DatabaseError::Other(err.0.to_string()),
        }
    }
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
