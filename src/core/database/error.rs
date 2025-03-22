//! Database error handling
//!
//! This module provides the error types for database operations

use std::fmt::{Debug, Display};

use crate::core::error::AppError;

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    /// Database not enabled in configuration
    #[error("Database not enabled in configuration")]
    NotEnabled,

    /// Database connection failed
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    /// Database query failed
    #[error("Database query failed: {0}")]
    QueryFailed(String),

    /// Database transaction failed
    #[error("Database transaction failed: {0}")]
    TransactionFailed(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// No rows returned by query
    #[error("No rows returned by query")]
    NoRows,

    /// Database migration failed
    #[error("Database migration failed: {0}")]
    MigrationFailed(String),

    /// Other database error
    #[error("Database error: {0}")]
    Other(String),
}

impl From<DatabaseError> for AppError {
    fn from(error: DatabaseError) -> Self {
        AppError::DatabaseError(error.to_string())
    }
}
