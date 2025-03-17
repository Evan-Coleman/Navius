use app_config::{AppConfig, load_config};
use lazy_static::lazy_static;
use std::sync::Arc;

mod app_config;
pub use app_config::*;

lazy_static! {
    pub static ref CONFIG: Arc<AppConfig> =
        Arc::new(load_config().expect("Failed to load configuration"));
}
