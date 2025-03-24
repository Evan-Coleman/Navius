pub mod api;
pub mod auth;
pub mod cache;
pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod metrics;
pub mod models;
pub mod reliability;
pub mod repository;
pub mod router;
pub mod services;
pub mod utils;

// Re-export key components for easier access
pub use auth::{EntraAuthLayer, EntraTokenClient};
pub use cache::{CacheRegistry, ResourceCache, get_resource_cache, init_cache_registry};
pub use config::app_config::{AppConfig, load_config};
pub use database::{DatabaseConnection, PgPool, Transaction, init_database};
pub use error::{AppError, Result};
pub use metrics::{init_metrics, metrics_handler, try_record_metrics};
pub use reliability::apply_reliability;
pub use router::CoreRouter;
pub use utils::api_resource::{
    ApiHandlerOptions, ApiResource, ApiResourceRegistry, create_api_handler,
};

pub use auth::*;
pub use cache::*;
pub use config::*;
pub use database::*;
pub use handlers::*;
pub use metrics::*;
pub use models::*;
pub use router::*;
pub use services::*;
pub use utils::*;
