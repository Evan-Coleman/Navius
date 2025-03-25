//! # Service Examples
//!
//! This module provides examples of services using Spring Boot-like patterns.
//! These examples demonstrate caching, transactional behavior, and other
//! common service patterns.

pub mod cached_user_service;
pub use cached_user_service::CachedUserService;
