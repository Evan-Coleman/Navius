use navius_db::error::{DatabaseError, DatabaseResult};
use std::fmt;

/// PostgreSQL-specific database error
#[derive(Debug)]
pub struct PgDatabaseError {
    /// The original SQLx error if available
    pub source: Option<sqlx::Error>,

    /// The PostgreSQL error code if available
    pub code: Option<String>,

    /// Error message
    pub message: String,
}

impl PgDatabaseError {
    /// Create a new error with a message
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            source: None,
            code: None,
            message: message.into(),
        }
    }

    /// Create a new error from an SQLx error
    pub fn from_sqlx(err: sqlx::Error) -> Self {
        let code = if let sqlx::Error::Database(ref db_err) = err {
            db_err.code().map(|c| c.to_string())
        } else {
            None
        };

        Self {
            source: Some(err.clone()),
            code,
            message: err.to_string(),
        }
    }

    /// Check if this is a duplicate key error
    pub fn is_duplicate_key(&self) -> bool {
        self.code.as_deref() == Some("23505")
    }

    /// Check if this is a foreign key violation
    pub fn is_foreign_key_violation(&self) -> bool {
        self.code.as_deref() == Some("23503")
    }

    /// Check if this is a not null violation
    pub fn is_not_null_violation(&self) -> bool {
        self.code.as_deref() == Some("23502")
    }

    /// Check if this is a transaction aborted error
    pub fn is_transaction_aborted(&self) -> bool {
        self.code.as_deref() == Some("25P02")
    }
}

impl fmt::Display for PgDatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "PostgreSQL error ({}): {}", code, self.message)
        } else {
            write!(f, "PostgreSQL error: {}", self.message)
        }
    }
}

impl std::error::Error for PgDatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}

impl From<PgDatabaseError> for DatabaseError {
    fn from(err: PgDatabaseError) -> Self {
        match err.code.as_deref() {
            Some("23505") => {
                DatabaseError::ValidationError(format!("Duplicate key violation: {}", err.message))
            }
            Some("23503") => {
                DatabaseError::ValidationError(format!("Foreign key violation: {}", err.message))
            }
            Some("23502") => {
                DatabaseError::ValidationError(format!("Not null violation: {}", err.message))
            }
            Some("25P02") => {
                DatabaseError::TransactionError(format!("Transaction aborted: {}", err.message))
            }
            Some("08006") => {
                DatabaseError::ConnectionError(format!("Connection failed: {}", err.message))
            }
            Some("08001") => {
                DatabaseError::ConnectionError(format!("Connection rejected: {}", err.message))
            }
            Some("08004") => {
                DatabaseError::ConnectionError(format!("Connection rejected: {}", err.message))
            }
            Some("57P01") => {
                DatabaseError::ConnectionError(format!("Server shutdown: {}", err.message))
            }
            Some("57P02") => {
                DatabaseError::ConnectionError(format!("Connection dropped: {}", err.message))
            }
            Some("57P03") => {
                DatabaseError::ConnectionError(format!("Database unavailable: {}", err.message))
            }
            Some(code) if code.starts_with("42") => {
                DatabaseError::QueryError(format!("Syntax error {}: {}", code, err.message))
            }
            _ => DatabaseError::QueryError(err.message),
        }
    }
}

// Helper functions for working with SQLx errors
pub(crate) fn map_sqlx_error(err: sqlx::Error) -> DatabaseError {
    PgDatabaseError::from_sqlx(err).into()
}

/// Helper to handle sqlx result
pub(crate) fn handle_sqlx_result<T>(result: Result<T, sqlx::Error>) -> DatabaseResult<T> {
    result.map_err(map_sqlx_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pg_error_display() {
        let error = PgDatabaseError {
            source: None,
            code: Some("23505".to_string()),
            message: "Duplicate key violation".to_string(),
        };

        assert_eq!(
            error.to_string(),
            "PostgreSQL error (23505): Duplicate key violation"
        );

        let error = PgDatabaseError {
            source: None,
            code: None,
            message: "Unknown error".to_string(),
        };

        assert_eq!(error.to_string(), "PostgreSQL error: Unknown error");
    }

    #[test]
    fn test_error_checks() {
        let error = PgDatabaseError {
            source: None,
            code: Some("23505".to_string()),
            message: "Duplicate key violation".to_string(),
        };

        assert!(error.is_duplicate_key());
        assert!(!error.is_foreign_key_violation());
        assert!(!error.is_not_null_violation());
        assert!(!error.is_transaction_aborted());

        let error = PgDatabaseError {
            source: None,
            code: Some("23503".to_string()),
            message: "Foreign key violation".to_string(),
        };

        assert!(!error.is_duplicate_key());
        assert!(error.is_foreign_key_violation());
    }

    #[test]
    fn test_conversion_to_database_error() {
        let pg_error = PgDatabaseError {
            source: None,
            code: Some("23505".to_string()),
            message: "Duplicate key violation".to_string(),
        };

        let db_error: DatabaseError = pg_error.into();

        match db_error {
            DatabaseError::ValidationError(msg) => {
                assert!(msg.contains("Duplicate key violation"));
            }
            _ => panic!("Expected ValidationError"),
        }

        let pg_error = PgDatabaseError {
            source: None,
            code: Some("25P02".to_string()),
            message: "Transaction aborted".to_string(),
        };

        let db_error: DatabaseError = pg_error.into();

        match db_error {
            DatabaseError::TransactionError(msg) => {
                assert!(msg.contains("Transaction aborted"));
            }
            _ => panic!("Expected TransactionError"),
        }
    }
}
