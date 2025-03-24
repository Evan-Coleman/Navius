//! Logger module for Navius application

use tracing::{
    debug as tracing_debug, error as tracing_error, info as tracing_info, warn as tracing_warn,
};

// Re-export tracing macros as our own macros
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Initializes the logger with default configuration
pub fn init() {
    // This is a placeholder implementation
    // In a real application, we would initialize tracing or log crates
    // with appropriate configuration
}

/// Log a request
pub fn log_request(method: &str, path: &str, status: u16, duration: u128) {
    tracing_info!("{} {} {} {}ms", method, path, status, duration);
}
