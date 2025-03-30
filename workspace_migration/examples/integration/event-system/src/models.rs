//! Event System Models
//!
//! This file contains the domain models used in the Event System integration example.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event record stored in the database
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Order item representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub price: f64,
}

/// Alert level enum for system alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Error,
    Critical,
}

/// Event metadata common to all domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

/// Inventory status response
#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryStatus {
    pub product_id: String,
    pub quantity: i32,
    pub last_updated: DateTime<Utc>,
    pub status: StockStatus,
}

/// Stock status indicator
#[derive(Debug, Serialize, Deserialize)]
pub enum StockStatus {
    InStock,
    LowStock,
    OutOfStock,
}

impl StockStatus {
    pub fn from_quantity(qty: i32) -> Self {
        match qty {
            0 => StockStatus::OutOfStock,
            1..=10 => StockStatus::LowStock,
            _ => StockStatus::InStock,
        }
    }
}

/// API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: None,
            data: Some(data),
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: Some(message.to_string()),
            data: None,
            timestamp: Utc::now(),
        }
    }
}

/// Event metrics summary
#[derive(Debug, Serialize)]
pub struct EventMetricsSummary {
    pub event_counts: std::collections::HashMap<String, i32>,
    pub total_events: i32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}
