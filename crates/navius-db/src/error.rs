use navius_core::error::Error as AppError;
use navius_core::error::ErrorCode;
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

    /// Error with detailed database context
    #[error("{db_operation} failed on {table_name}: {message}")]
    DetailedDatabaseError {
        /// The database operation that was being performed (e.g., "INSERT", "UPDATE")
        db_operation: String,
        /// The table or entity that was being operated on
        table_name: String,
        /// The error message
        message: String,
        /// The underlying error, if any
        source: Option<Box<DatabaseError>>,
    },

    /// Error with error chains
    #[error("{message}")]
    ChainedError {
        /// Primary error message
        message: String,
        /// Detailed error context
        context: ErrorContext,
        /// Error chain (previous errors that led to this one)
        chain: Vec<Box<DatabaseError>>,
    },
}

impl Clone for DatabaseError {
    fn clone(&self) -> Self {
        match self {
            Self::ConnectionError(s) => Self::ConnectionError(s.clone()),
            Self::QueryError(s) => Self::QueryError(s.clone()),
            Self::TransactionError(s) => Self::TransactionError(s.clone()),
            Self::TransactionFinished => Self::TransactionFinished,
            Self::SavepointError(s) => Self::SavepointError(s.clone()),
            Self::PoolError(s) => Self::PoolError(s.clone()),
            Self::MigrationError(s) => Self::MigrationError(s.clone()),
            Self::ConfigurationError(s) => Self::ConfigurationError(s.clone()),
            Self::NotFoundError(s) => Self::NotFoundError(s.clone()),
            Self::ValidationError(s) => Self::ValidationError(s.clone()),
            Self::UnexpectedStateError(s) => Self::UnexpectedStateError(s.clone()),
            Self::WithContext { context, source } => Self::WithContext {
                context: context.clone(),
                source: source.clone(),
            },
            Self::ParameterError(s) => Self::ParameterError(s.clone()),
            Self::RowAccessError(s) => Self::RowAccessError(s.clone()),
            Self::SQLXError(_) => Self::QueryError("Database error (clone of SQLXError)".into()),
            Self::IOError(_) => Self::QueryError("IO error (clone of IOError)".into()),
            Self::DetailedDatabaseError {
                db_operation,
                table_name,
                message,
                source,
            } => Self::DetailedDatabaseError {
                db_operation: db_operation.clone(),
                table_name: table_name.clone(),
                message: message.clone(),
                source: source.clone(),
            },
            Self::ChainedError {
                message,
                context,
                chain: ref_chain,
            } => {
                // Create a new chain
                let new_chain = ref_chain
                    .iter()
                    .map(|err| Box::new((**err).clone()))
                    .collect();

                Self::ChainedError {
                    message: message.clone(),
                    context: context.clone(),
                    chain: new_chain,
                }
            }
        }
    }
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

    /// Database-specific information
    pub db_specific: Option<DatabaseSpecificInfo>,
}

/// Database-specific error information
#[derive(Debug, Clone)]
pub struct DatabaseSpecificInfo {
    /// Database error code (e.g., PostgreSQL error code)
    pub error_code: Option<String>,

    /// Database constraint that was violated, if any
    pub constraint: Option<String>,

    /// Database schema where the error occurred
    pub schema: Option<String>,

    /// Database table where the error occurred
    pub table: Option<String>,

    /// Database column where the error occurred
    pub column: Option<String>,

    /// Database severity level (e.g., ERROR, FATAL, PANIC)
    pub severity: Option<String>,
}

impl Display for DatabaseSpecificInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if let Some(code) = &self.error_code {
            parts.push(format!("error_code: {}", code));
        }

        if let Some(constraint) = &self.constraint {
            parts.push(format!("constraint: {}", constraint));
        }

        if let Some(schema) = &self.schema {
            if let Some(table) = &self.table {
                if let Some(column) = &self.column {
                    parts.push(format!("location: {}.{}.{}", schema, table, column));
                } else {
                    parts.push(format!("location: {}.{}", schema, table));
                }
            } else {
                parts.push(format!("schema: {}", schema));
            }
        } else if let Some(table) = &self.table {
            if let Some(column) = &self.column {
                parts.push(format!("location: {}.{}", table, column));
            } else {
                parts.push(format!("table: {}", table));
            }
        } else if let Some(column) = &self.column {
            parts.push(format!("column: {}", column));
        }

        if let Some(severity) = &self.severity {
            parts.push(format!("severity: {}", severity));
        }

        if parts.is_empty() {
            write!(f, "No database-specific information")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
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

        if let Some(query) = &self.query {
            parts.push(format!("query: {}", truncate_query(query, 100)));
        }

        if let Some(info) = &self.additional_info {
            parts.push(info.clone());
        }

        if let Some(db_info) = &self.db_specific {
            parts.push(format!("db_info: [{}]", db_info));
        }

        if parts.is_empty() {
            write!(f, "Database error")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            query: None,
            entity_type: None,
            operation: None,
            additional_info: None,
            db_specific: None,
        }
    }

    /// Add query information to the context
    pub fn with_query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Add entity type information to the context
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }

    /// Add operation information to the context
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Add additional information to the context
    pub fn with_additional_info(mut self, info: impl Into<String>) -> Self {
        self.additional_info = Some(info.into());
        self
    }

    /// Add database-specific information to the context
    pub fn with_db_specific(mut self, db_info: DatabaseSpecificInfo) -> Self {
        self.db_specific = Some(db_info);
        self
    }

    /// Extract database-specific information from a sqlx error
    pub fn from_sqlx_error(error: &sqlx::Error) -> Self {
        let mut context = Self::new();

        if let sqlx::Error::Database(db_err) = error {
            let mut db_info = DatabaseSpecificInfo {
                error_code: db_err.code().map(|c| c.to_string()),
                constraint: None,
                schema: None,
                table: None,
                column: None,
                severity: None,
            };

            #[cfg(feature = "postgres")]
            if let Some(pg_err) = db_err.try_downcast_ref::<sqlx::postgres::PgDatabaseError>() {
                db_info.constraint = pg_err.constraint().map(|s| s.to_string());
                db_info.schema = pg_err.schema().map(|s| s.to_string());
                db_info.table = pg_err.table().map(|s| s.to_string());
                db_info.column = pg_err.column().map(|s| s.to_string());
                db_info.severity = Some(pg_err.severity().to_string());

                if let Some(hint) = pg_err.hint() {
                    context = context.with_additional_info(format!("hint: {}", hint));
                }

                if let Some(detail) = pg_err.detail() {
                    if context.additional_info.is_some() {
                        context.additional_info = context
                            .additional_info
                            .map(|info| format!("{}, detail: {}", info, detail));
                    } else {
                        context = context.with_additional_info(format!("detail: {}", detail));
                    }
                }
            }

            context = context.with_db_specific(db_info);
        }

        context
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

    /// Create a new error with detailed context information
    pub fn with_detailed_context(self, context: ErrorContext) -> Self {
        match self {
            // If already a chained error, add to the chain
            DatabaseError::ChainedError {
                message,
                context: existing_context,
                mut chain,
            } => {
                // Combine contexts if possible
                let combined_context = Self::combine_contexts(existing_context, context);
                // Add the original error to the chain
                chain.push(Box::new(self.clone()));

                DatabaseError::ChainedError {
                    message,
                    context: combined_context,
                    chain,
                }
            }
            // Otherwise create a new chained error
            _ => DatabaseError::ChainedError {
                message: self.to_string(),
                context,
                chain: vec![Box::new(self)],
            },
        }
    }

    /// Create a new error with detailed database context
    pub fn detailed_db_error<S: Into<String>, T: Into<String>>(
        operation: S,
        table: T,
        message: S,
        source: Option<DatabaseError>,
    ) -> Self {
        DatabaseError::DetailedDatabaseError {
            db_operation: operation.into(),
            table_name: table.into(),
            message: message.into(),
            source: source.map(Box::new),
        }
    }

    /// Chain a new error with this one
    pub fn chain_error<M: Into<String>>(self, message: M, context: ErrorContext) -> Self {
        match self {
            // If already a chained error, add to the chain
            DatabaseError::ChainedError { mut chain, .. } => {
                // Add the original error to the chain
                chain.push(Box::new(self.clone()));

                DatabaseError::ChainedError {
                    message: message.into(),
                    context,
                    chain,
                }
            }
            // Otherwise create a new chained error
            _ => DatabaseError::ChainedError {
                message: message.into(),
                context,
                chain: vec![Box::new(self)],
            },
        }
    }

    /// Combine two error contexts, preferring the values from the first context when both have values
    fn combine_contexts(ctx1: ErrorContext, ctx2: ErrorContext) -> ErrorContext {
        ErrorContext {
            query: ctx1.query.or(ctx2.query),
            entity_type: ctx1.entity_type.or(ctx2.entity_type),
            operation: ctx1.operation.or(ctx2.operation),
            additional_info: match (ctx1.additional_info, ctx2.additional_info) {
                (Some(info1), Some(info2)) => Some(format!("{}, {}", info1, info2)),
                (Some(info), None) => Some(info),
                (None, Some(info)) => Some(info),
                (None, None) => None,
            },
            db_specific: ctx1.db_specific.or(ctx2.db_specific),
        }
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
            Self::DetailedDatabaseError { source, .. } => source
                .as_ref()
                .map_or("DB_DETAILED_ERROR", |s| s.error_code()),
            Self::ChainedError { chain, .. } => {
                chain.first().map_or("DB_CHAINED_ERROR", |s| s.error_code())
            }
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
            Self::DetailedDatabaseError { source, .. } => {
                source.as_ref().map_or(500, |s| s.status_code())
            }
            Self::ChainedError { chain, .. } => chain.first().map_or(500, |s| s.status_code()),
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

    /// Get the detailed error chain for this error
    pub fn error_chain(&self) -> Vec<&DatabaseError> {
        match self {
            Self::ChainedError { chain, .. } => {
                let mut result = vec![self];
                for err in chain.iter() {
                    result.push(err);
                }
                result
            }
            Self::WithContext { source, .. } => {
                let mut result = vec![self];
                let unwrapped = source.unwrap_context();
                result.push(unwrapped);
                result
            }
            Self::DetailedDatabaseError { source, .. } => {
                let mut result = vec![self];
                if let Some(src) = source {
                    result.push(src);
                }
                result
            }
            _ => vec![self],
        }
    }

    /// Get the root cause of this error
    pub fn root_cause(&self) -> &DatabaseError {
        match self {
            Self::ChainedError { chain, .. } => chain.last().map_or(self, |e| e.root_cause()),
            Self::WithContext { source, .. } => source.root_cause(),
            Self::DetailedDatabaseError { source, .. } => {
                source.as_ref().map_or(self, |s| s.root_cause())
            }
            _ => self,
        }
    }

    /// Extract database-specific information from this error
    pub fn db_specific_info(&self) -> Option<DatabaseSpecificInfo> {
        match self {
            Self::SQLXError(sqlx_err) => {
                if let sqlx::Error::Database(db_err) = sqlx_err {
                    let mut info = DatabaseSpecificInfo {
                        error_code: db_err.code().map(|c| c.to_string()),
                        constraint: None,
                        schema: None,
                        table: None,
                        column: None,
                        severity: None,
                    };

                    #[cfg(feature = "postgres")]
                    if let Some(pg_err) =
                        db_err.try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
                    {
                        info.constraint = pg_err.constraint().map(|s| s.to_string());
                        info.schema = pg_err.schema().map(|s| s.to_string());
                        info.table = pg_err.table().map(|s| s.to_string());
                        info.column = pg_err.column().map(|s| s.to_string());
                        info.severity = Some(pg_err.severity().to_string());
                    }

                    Some(info)
                } else {
                    None
                }
            }
            Self::DetailedDatabaseError { table_name, .. } => Some(DatabaseSpecificInfo {
                error_code: None,
                constraint: None,
                schema: None,
                table: Some(table_name.clone()),
                column: None,
                severity: None,
            }),
            Self::ChainedError { context, .. } => context.db_specific.clone(),
            Self::WithContext { source, .. } => source.db_specific_info(),
            _ => None,
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
            Self::WithContext { source, .. } => source.is_transient(),
            Self::DetailedDatabaseError { source, .. } => {
                source.as_ref().map_or(false, |s| s.is_transient())
            }
            Self::ChainedError { chain, .. } => chain.iter().any(|err| err.is_transient()),
            _ => false,
        }
    }

    /// Create an error with detailed database context from a SQLx error
    pub fn from_sqlx_with_context(
        sqlx_err: sqlx::Error,
        operation: &str,
        entity: Option<&str>,
    ) -> Self {
        let context = ErrorContext::from_sqlx_error(&sqlx_err);

        let context = if let Some(entity_type) = entity {
            context.with_entity_type(entity_type)
        } else {
            context
        };

        let context = context.with_operation(operation);

        let db_error = DatabaseError::SQLXError(sqlx_err);
        db_error.with_detailed_context(context)
    }

    /// Unwraps a QueryError and returns the error message
    pub fn unwrap_query_error(&self) -> &str {
        match self {
            DatabaseError::QueryError(msg) => msg,
            _ => panic!("Expected QueryError, got {:?}", self),
        }
    }
}

/// Truncate a query string to a maximum length for display in error messages
fn truncate_query(query: &str, max_length: usize) -> String {
    if query.len() <= max_length {
        query.to_string()
    } else {
        format!("{}...", &query[0..max_length])
    }
}

impl From<DatabaseError> for AppError {
    fn from(err: DatabaseError) -> Self {
        match &err {
            DatabaseError::NotFoundError(msg) => AppError::not_found(msg),
            DatabaseError::ValidationError(msg) => AppError::validation(msg),
            DatabaseError::ConfigurationError(msg) => AppError::internal(msg),
            DatabaseError::ConnectionError(msg) => AppError::external(msg),
            _ => {
                let error_code = match err.status_code() {
                    400 => ErrorCode::Validation,
                    404 => ErrorCode::NotFound,
                    500 => ErrorCode::Internal,
                    _ => ErrorCode::Unknown,
                };

                AppError::new(error_code, err.to_string())
            }
        }
    }
}

/// Type alias for database results
pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_with_context() {
        let inner_error = DatabaseError::QueryError("SQL syntax error".to_string());
        let error = inner_error.with_context("Failed to execute query");

        match error {
            DatabaseError::WithContext { context, source } => {
                assert_eq!(context, "Failed to execute query");
                assert!(matches!(*source, DatabaseError::QueryError(_)));
            }
            _ => panic!("Expected WithContext error"),
        }
    }

    #[test]
    fn test_error_with_query_context() {
        let inner_error = DatabaseError::QueryError("column not found".to_string());
        let error =
            inner_error.with_query_context("SELECT * FROM users WHERE username = ?", "SELECT");

        match error {
            DatabaseError::WithContext { context, source } => {
                assert_eq!(
                    context,
                    "Error in SELECT query: SELECT * FROM users WHERE username = ?"
                );
                assert!(matches!(*source, DatabaseError::QueryError(_)));
            }
            _ => panic!("Expected WithContext error"),
        }
    }

    #[test]
    fn test_savepoint_error() {
        let error = DatabaseError::savepoint_error("Savepoint 'test' does not exist");

        match error {
            DatabaseError::SavepointError(msg) => {
                assert_eq!(msg, "Savepoint 'test' does not exist");
            }
            _ => panic!("Expected SavepointError"),
        }
    }

    #[test]
    fn test_unwrap_context() {
        let inner_error = DatabaseError::QueryError("inner error".to_string());
        let with_context = inner_error.with_context("outer context");
        let unwrapped = with_context.unwrap_context();

        assert!(matches!(unwrapped, DatabaseError::QueryError(_)));
    }

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new()
            .with_query("SELECT * FROM users")
            .with_entity_type("User")
            .with_operation("SELECT")
            .with_additional_info("User ID: 123");

        assert_eq!(context.query, Some("SELECT * FROM users".to_string()));
        assert_eq!(context.entity_type, Some("User".to_string()));
        assert_eq!(context.operation, Some("SELECT".to_string()));
        assert_eq!(context.additional_info, Some("User ID: 123".to_string()));
    }

    #[test]
    fn test_detailed_db_error() {
        let error = DatabaseError::detailed_db_error(
            "INSERT",
            "users",
            "Duplicate key value violates unique constraint",
            Some(DatabaseError::ValidationError(
                "Email already exists".to_string(),
            )),
        );

        match error {
            DatabaseError::DetailedDatabaseError {
                db_operation,
                table_name,
                message,
                source,
            } => {
                assert_eq!(db_operation, "INSERT");
                assert_eq!(table_name, "users");
                assert_eq!(message, "Duplicate key value violates unique constraint");
                assert!(
                    matches!(source, Some(box_err) if matches!(*box_err, DatabaseError::ValidationError(_)))
                );
            }
            _ => panic!("Expected DetailedDatabaseError"),
        }
    }

    #[test]
    fn test_error_chaining() {
        let inner_error = DatabaseError::QueryError("column not found".to_string());

        let context = ErrorContext::new()
            .with_operation("SELECT")
            .with_entity_type("User");

        let chained_error = inner_error.chain_error("Failed to retrieve user", context);

        match chained_error {
            DatabaseError::ChainedError {
                message,
                context,
                chain,
            } => {
                assert_eq!(message, "Failed to retrieve user");
                assert_eq!(context.operation, Some("SELECT".to_string()));
                assert_eq!(context.entity_type, Some("User".to_string()));
                assert_eq!(chain.len(), 1);
                assert!(matches!(*chain[0], DatabaseError::QueryError(_)));
            }
            _ => panic!("Expected ChainedError"),
        }
    }

    #[test]
    fn test_error_chain_extraction() {
        let inner_error = DatabaseError::QueryError("column not found".to_string());

        let context = ErrorContext::new()
            .with_operation("SELECT")
            .with_entity_type("User");

        let chained_error = inner_error.chain_error("Failed to retrieve user", context);

        let chain = chained_error.error_chain();
        assert_eq!(chain.len(), 2);
        assert!(matches!(chain[0], DatabaseError::ChainedError { .. }));
        assert!(matches!(chain[1], DatabaseError::QueryError(_)));
    }

    #[test]
    fn test_root_cause() {
        let inner_error = DatabaseError::QueryError("column not found".to_string());

        let context = ErrorContext::new()
            .with_operation("SELECT")
            .with_entity_type("User");

        let chained_error = inner_error.chain_error("Failed to retrieve user", context);

        let root = chained_error.root_cause();
        assert!(matches!(root, DatabaseError::QueryError(_)));
    }
}
