//! # Services module
//!
//! This module provides services that implement business logic.
//! Services use repositories to interact with data and implement business rules.

use crate::core::error::AppError;
use reqwest::Client;
use std::any::Any;
use std::sync::Arc;

pub mod error;
pub mod health;
pub mod metrics;

pub use error::ServiceError;

/// Type alias for service results
pub type ServiceResult<T> = Result<T, ServiceError>;
