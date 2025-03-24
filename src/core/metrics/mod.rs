pub mod metrics_handler;
mod metrics_service;

// Re-export key components for easier access but avoid ambiguity
pub use metrics_handler::{
    MetricsHandle, export_metrics, increment_counter, record_histogram, update_gauge,
};

pub use metrics_service::{init_metrics, try_record_metrics};
