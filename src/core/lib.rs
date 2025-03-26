//! # Core module
//!
//! This module provides the core functionality for the application.
//! It includes common utilities, abstractions, and the foundation for the application.

// Module declarations using Rust 2018 style
pub mod api;
pub mod auth; // Now points to auth.rs instead of auth/mod.rs
pub mod auth_providers;
pub mod cache; // Now points to cache.rs instead of cache/mod.rs
pub mod config; // Now points to config.rs instead of config/mod.rs
pub mod core_logger;
pub mod core_middleware;
pub mod error;
pub mod features; // Feature selection and customization system
pub mod handlers;
pub mod macros;
pub mod metrics;
pub mod models;
pub mod reliability; // Now points to reliability.rs instead of reliability/mod.rs
pub mod router;
pub mod services;
pub mod utils; // If needed

// Existing re-exports remain unchanged...
