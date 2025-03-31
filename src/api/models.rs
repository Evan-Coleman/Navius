use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMetadata>,
}

/// Response metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Pagination information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,
}

/// Pagination information
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub page_size: u32,
    /// Total number of items
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u32,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error information
    pub error: ErrorInfo,
}

/// Error information
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request ID for tracing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Number of items per page
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

/// Sort direction
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "asc"),
            SortDirection::Desc => write!(f, "desc"),
        }
    }
}

/// Default page number
fn default_page() -> u32 {
    1
}

/// Default page size
fn default_page_size() -> u32 {
    20
}

/// Helper functions for pagination
pub mod pagination {
    use super::*;

    /// Calculate pagination values
    pub fn calculate_pagination(page: u32, page_size: u32, total_items: u64) -> PaginationInfo {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;

        PaginationInfo {
            page,
            page_size,
            total_items,
            total_pages,
        }
    }

    /// Calculate offset and limit for database queries
    pub fn calculate_offset_limit(page: u32, page_size: u32) -> (u64, u64) {
        let offset = ((page - 1) * page_size) as u64;
        let limit = page_size as u64;

        (offset, limit)
    }
}
