pub mod app_config;
pub mod constants;
#[cfg(test)]
mod tests;

pub use app_config::AppConfig;
pub use app_config::load_config;

use lazy_static::lazy_static;
use std::sync::Arc;

lazy_static! {
    pub static ref CONFIG: Arc<AppConfig> =
        Arc::new(load_config().expect("Failed to load configuration"));
}
