pub mod error_types;
pub mod logger;
pub mod middleware;
pub mod result_ext;

// Re-export common types and functions
pub use error_types::{AppError, ErrorResponse, ErrorSeverity, Result};
pub use logger::{LogInfo, LogLevel, log, log_error};
pub use middleware::RequestTrackingLayer;
pub use middleware::{RequestId, RequestIdExt, generate_request_id};
pub use result_ext::{ResultExt, StatusCodeExt};

// Re-export macro
pub use crate::log_at_level;
