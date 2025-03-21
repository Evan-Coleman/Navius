//! # Application Error Handling
//!
//! This module provides a user-friendly interface to the application's error handling system.
//! It serves as a wrapper around the core error handling system located in `src/core/error`.

// Re-export all error types and functions from core
pub use crate::core::error::error_types::{AppError, ErrorResponse, ErrorSeverity, Result};
pub use crate::core::error::logger::{LogInfo, LogLevel, log, log_error};
pub use crate::core::error::middleware::{
    RequestId, RequestIdExt, RequestTrackingLayer, generate_request_id,
};
pub use crate::core::error::result_ext::{ResultExt, StatusCodeExt};

// Re-export macro
pub use crate::log_at_level;

// User extensions to the error system can be added here:
pub mod error_types;
pub mod logger;
pub mod middleware;
pub mod result_ext;
