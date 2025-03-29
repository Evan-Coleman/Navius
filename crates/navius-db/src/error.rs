use navius_core::error::Error as AppError;
use thiserror::Error;

/// Database errors that can occur during database operations
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Connection errors
    #[error("Failed to connect to database: {0}")]
    ConnectionError(String),

    /// Query execution errors
    #[error("Query execution failed: {0}")]
    QueryError(String),

    /// Transaction errors
    #[error("Transaction failed: {0}")]
    TransactionError(String),

    /// Pool errors
    #[error("Database pool error: {0}")]
    PoolError(String),

    /// Migration errors
    #[error("Database migration failed: {0}")]
    MigrationError(String),

    /// Configuration errors
    #[error("Invalid database configuration: {0}")]
    ConfigurationError(String),

    /// Entity not found
    #[error("Entity not found: {0}")]
    NotFoundError(String),

    /// Validation errors
    #[error("Data validation failed: {0}")]
    ValidationError(String),

    /// Unexpected database state
    #[error("Unexpected database state: {0}")]
    UnexpectedStateError(String),
}

impl DatabaseError {
    /// Get the error code for this database error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ConnectionError(_) => "DB_CONNECTION_ERROR",
            Self::QueryError(_) => "DB_QUERY_ERROR",
            Self::TransactionError(_) => "DB_TRANSACTION_ERROR",
            Self::PoolError(_) => "DB_POOL_ERROR",
            Self::MigrationError(_) => "DB_MIGRATION_ERROR",
            Self::ConfigurationError(_) => "DB_CONFIG_ERROR",
            Self::NotFoundError(_) => "DB_NOT_FOUND",
            Self::ValidationError(_) => "DB_VALIDATION_ERROR",
            Self::UnexpectedStateError(_) => "DB_UNEXPECTED_STATE",
        }
    }

    /// Get the HTTP status code for this database error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFoundError(_) => 404,
            Self::ValidationError(_) => 400,
            Self::ConfigurationError(_) => 500,
            Self::ConnectionError(_) => 503,
            _ => 500,
        }
    }
}

// Implement conversion from DatabaseError to AppError
impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::NotFoundError(msg) => AppError::validation(msg),
            DatabaseError::ValidationError(msg) => AppError::validation(msg),
            DatabaseError::ConnectionError(msg) => AppError::internal(msg),
            DatabaseError::ConfigurationError(msg) => AppError::configuration(msg),
            _ => AppError::internal(err.to_string()),
        }
    }
}

// Implement conversion from SQLx errors to DatabaseError
#[cfg(feature = "postgres")]
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                DatabaseError::NotFoundError("Record not found".to_string())
            }
            sqlx::Error::Database(db_err) => {
                // Postgres-specific error handling
                if let Some(code) = db_err.code() {
                    match code.as_ref() {
                        // Common Postgres error codes
                        "23505" => DatabaseError::ValidationError(format!(
                            "Duplicate key violation: {}",
                            db_err
                        )),
                        "23503" => DatabaseError::ValidationError(format!(
                            "Foreign key violation: {}",
                            db_err
                        )),
                        "23502" => DatabaseError::ValidationError(format!(
                            "Not null violation: {}",
                            db_err
                        )),
                        "22P02" => DatabaseError::ValidationError(format!(
                            "Invalid input syntax: {}",
                            db_err
                        )),
                        _ => DatabaseError::QueryError(format!("Database error: {}", db_err)),
                    }
                } else {
                    DatabaseError::QueryError(format!("Database error: {}", db_err))
                }
            }
            sqlx::Error::Io(io_err) => {
                DatabaseError::ConnectionError(format!("IO error: {}", io_err))
            }
            sqlx::Error::Tls(tls_err) => {
                DatabaseError::ConnectionError(format!("TLS error: {}", tls_err))
            }
            sqlx::Error::Protocol(msg) => {
                DatabaseError::QueryError(format!("Protocol error: {}", msg))
            }
            sqlx::Error::PoolTimedOut => {
                DatabaseError::PoolError("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                DatabaseError::PoolError("Connection pool closed".to_string())
            }
            sqlx::Error::WorkerCrashed => {
                DatabaseError::PoolError("Database worker crashed".to_string())
            }
            _ => DatabaseError::UnexpectedStateError(format!("Unexpected database error: {}", err)),
        }
    }
}

// Create a Result type alias for database operations
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;
