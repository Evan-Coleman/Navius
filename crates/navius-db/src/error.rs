use navius_core::error::Error as AppError;
use std::fmt::{Display, Formatter};
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

    /// Transaction already finished
    #[error("Transaction already committed or rolled back")]
    TransactionFinished,

    /// Savepoint errors
    #[error("Savepoint operation failed: {0}")]
    SavepointError(String),

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

    /// Error with context
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        source: Box<DatabaseError>,
    },

    /// Parameter binding error
    #[error("Parameter binding error: {0}")]
    ParameterError(String),

    /// Row access error
    #[error("Row access error: {0}")]
    RowAccessError(String),

    /// SQLx errors
    #[error(transparent)]
    SQLXError(#[from] sqlx::Error),

    /// IO errors
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

/// Error context information to enrich error messages
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The query that was being executed when the error occurred
    pub query: Option<String>,

    /// The entity type involved in the operation
    pub entity_type: Option<String>,

    /// The operation being performed
    pub operation: Option<String>,

    /// Additional context information
    pub additional_info: Option<String>,
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if let Some(op) = &self.operation {
            parts.push(format!("operation: {}", op));
        }

        if let Some(entity) = &self.entity_type {
            parts.push(format!("entity: {}", entity));
        }

        if let Some(info) = &self.additional_info {
            parts.push(info.clone());
        }

        if parts.is_empty() {
            write!(f, "Database error")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

impl DatabaseError {
    /// Add context to an error
    pub fn with_context<C: Into<String>>(self, context: C) -> Self {
        DatabaseError::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Add query context to an error
    pub fn with_query_context(self, query: &str, operation: &str) -> Self {
        let context = format!(
            "Error in {} query: {}",
            operation,
            truncate_query(query, 100)
        );
        self.with_context(context)
    }

    /// Create a new savepoint error
    pub fn savepoint_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::SavepointError(message.into())
    }

    /// Get the error code for this database error
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::ConnectionError(_) => "DB_CONNECTION_ERROR",
            Self::QueryError(_) => "DB_QUERY_ERROR",
            Self::TransactionError(_) => "DB_TRANSACTION_ERROR",
            Self::TransactionFinished => "DB_TRANSACTION_FINISHED",
            Self::SavepointError(_) => "DB_SAVEPOINT_ERROR",
            Self::PoolError(_) => "DB_POOL_ERROR",
            Self::MigrationError(_) => "DB_MIGRATION_ERROR",
            Self::ConfigurationError(_) => "DB_CONFIG_ERROR",
            Self::NotFoundError(_) => "DB_NOT_FOUND",
            Self::ValidationError(_) => "DB_VALIDATION_ERROR",
            Self::UnexpectedStateError(_) => "DB_UNEXPECTED_STATE",
            Self::WithContext { source, .. } => source.error_code(),
            Self::ParameterError(_) => "DB_PARAMETER_ERROR",
            Self::RowAccessError(_) => "DB_ROW_ACCESS_ERROR",
            Self::SQLXError(_) => "DB_SQLX_ERROR",
            Self::IOError(_) => "DB_IO_ERROR",
        }
    }

    /// Get the HTTP status code for this database error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFoundError(_) => 404,
            Self::ValidationError(_) => 400,
            Self::ConfigurationError(_) => 500,
            Self::ConnectionError(_) => 503,
            Self::WithContext { source, .. } => source.status_code(),
            Self::ParameterError(_) => 500,
            Self::RowAccessError(_) => 500,
            Self::SQLXError(_) => 500,
            Self::IOError(_) => 500,
            _ => 500,
        }
    }

    /// Unwrap the underlying error if this is a context error
    pub fn unwrap_context(&self) -> &DatabaseError {
        match self {
            Self::WithContext { source, .. } => source.unwrap_context(),
            _ => self,
        }
    }

    /// Create a new connection error
    pub fn connection_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::ConnectionError(message.into())
    }

    /// Create a new transaction error
    pub fn transaction_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::TransactionError(message.into())
    }

    /// Create a new query error
    pub fn query_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::QueryError(message.into())
    }

    /// Create a new parameter error
    pub fn parameter_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::ParameterError(message.into())
    }

    /// Create a new row access error
    pub fn row_access_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::RowAccessError(message.into())
    }

    /// Create a new pool error
    pub fn pool_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::PoolError(message.into())
    }

    /// Create a new configuration error
    pub fn configuration_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::ConfigurationError(message.into())
    }

    /// Create a new migration error
    pub fn migration_error<S: Into<String>>(message: S) -> Self {
        DatabaseError::MigrationError(message.into())
    }

    /// Check if the error is transient
    pub fn is_transient(&self) -> bool {
        match self {
            Self::SQLXError(e) => match e {
                sqlx::Error::Database(db_err) => {
                    // PostgreSQL error codes for transient errors:
                    // - 40001: serialization_failure
                    // - 40P01: deadlock_detected
                    // - 55P03: lock_not_available
                    // - 57P03: cannot_connect_now
                    // - 57P04: query_canceled
                    if let Some(code) = db_err.code() {
                        return match code.as_ref() {
                            "40001" | "40P01" | "55P03" | "57P03" | "57P04" => true,
                            _ => false,
                        };
                    }
                    false
                }
                sqlx::Error::Io(_) | sqlx::Error::PoolTimedOut => true,
                _ => false,
            },
            Self::ConnectionError(_) => true,
            _ => false,
        }
    }
}

/// Truncate a query string to a specified length for logging
fn truncate_query(query: &str, max_length: usize) -> String {
    if query.len() <= max_length {
        query.to_string()
    } else {
        format!("{}...", &query[..max_length])
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
            DatabaseError::WithContext { context, source } => {
                let app_error: AppError = (*source).into();
                app_error.with_context(context)
            }
            DatabaseError::ParameterError(msg) => AppError::validation(msg),
            DatabaseError::RowAccessError(msg) => AppError::validation(msg),
            DatabaseError::SQLXError(e) => {
                let msg = e.to_string();
                if e.is_transient() {
                    AppError::transient(msg)
                } else {
                    AppError::internal(msg)
                }
            }
            DatabaseError::IOError(e) => AppError::internal(e.to_string()),
            _ => AppError::internal(err.to_string()),
        }
    }
}

// Create a Result type alias for database operations
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_with_context() {
        let base_error = DatabaseError::ValidationError("Invalid value".to_string());
        let with_context = base_error.with_context("User registration failed");

        assert_eq!(with_context.error_code(), "DB_VALIDATION_ERROR");
        assert_eq!(with_context.status_code(), 400);
        assert!(
            with_context
                .to_string()
                .contains("User registration failed")
        );
        assert!(with_context.to_string().contains("Invalid value"));
    }

    #[test]
    fn test_error_with_query_context() {
        let base_error = DatabaseError::QueryError("Syntax error".to_string());
        let query = "SELECT * FROM users WHERE email = 'user@example.com' AND very_long_column_name = 'very_long_value_that_should_be_truncated'";
        let with_context = base_error.with_query_context(query, "select");

        assert_eq!(with_context.error_code(), "DB_QUERY_ERROR");
        assert!(with_context.to_string().contains("Error in select query"));
        assert!(with_context.to_string().contains("..."));
        assert!(
            !with_context
                .to_string()
                .contains("very_long_value_that_should_be_truncated")
        );
    }

    #[test]
    fn test_savepoint_error() {
        let error = DatabaseError::savepoint_error("Invalid savepoint name");

        assert_eq!(error.error_code(), "DB_SAVEPOINT_ERROR");
        assert_eq!(error.status_code(), 500);
        assert!(error.to_string().contains("Invalid savepoint name"));
    }

    #[test]
    fn test_unwrap_context() {
        let base_error = DatabaseError::ValidationError("Invalid value".to_string());
        let with_context = base_error.with_context("Context 1");
        let nested_context = with_context.with_context("Context 2");

        let unwrapped = nested_context.unwrap_context();
        match unwrapped {
            DatabaseError::ValidationError(msg) => assert_eq!(msg, "Invalid value"),
            _ => panic!("Unexpected error type after unwrapping context"),
        }
    }
}
