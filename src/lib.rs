// Module declarations with better organization
pub mod app {
    pub use crate::app::router::*;
    pub mod router;
}

pub mod cache {
    pub use crate::cache::cache_manager::*;
    pub mod cache_manager;
}

pub mod config {
    pub use crate::config::app_config::*;
    pub mod app_config;
}

pub mod error {
    pub use crate::error::error_types::*;
    pub mod error_types;
}

pub mod metrics {
    pub use crate::metrics::metrics_service::*;
    pub mod metrics_service;
}

pub mod handlers;
pub mod models;
pub mod petstore_api;
