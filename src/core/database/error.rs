//! Database error handling
//!
//! This module provides the error types for database operations

use std::fmt::{Debug, Display};

use crate::core::error::AppError;

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    /// Database connection is not enabled in configuration
    #[error("Database is not enabled in configuration")]
    NotEnabled,

    /// Connection error
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    /// Query error
    #[error("Database query error: {0}")]
    QueryError(String),

    /// Transaction error
    #[error("Database transaction error: {0}")]
    TransactionError(String),

    /// Migration error
    #[error("Database migration error: {0}")]
    MigrationError(String),

    /// Row not found
    #[error("Row not found")]
    RowNotFound,
}

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}
