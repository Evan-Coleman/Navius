use serde::{Deserialize, Serialize};

/// Standard pagination parameters for list endpoints
#[derive(Debug, Deserialize, Clone)]
pub struct PaginationParams {
    /// Page number (1-based)
    pub page: Option<u32>,
    /// Number of items per page
    pub per_page: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
        }
    }
}

impl PaginationParams {
    /// Get the offset for database queries
    pub fn offset(&self) -> u64 {
        let page = self.page.unwrap_or(1).max(1) as u64;
        let per_page = self.per_page.unwrap_or(20).max(1) as u64;
        (page - 1) * per_page
    }

    /// Get the limit for database queries
    pub fn limit(&self) -> u64 {
        self.per_page.unwrap_or(20).max(1) as u64
    }
}

/// Standard sorting parameter
#[derive(Debug, Deserialize, Clone)]
pub struct SortParams {
    /// Field to sort by
    pub sort: Option<String>,
    /// Sort direction (asc or desc)
    pub order: Option<String>,
}

impl Default for SortParams {
    fn default() -> Self {
        Self {
            sort: None,
            order: None,
        }
    }
}

/// Standard pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Current page number
    pub page: u32,
    /// Number of items per page
    pub per_page: u32,
    /// Total number of items
    pub total_items: u64,
    /// Total number of pages
    pub total_pages: u32,
    /// Has more pages
    pub has_more: bool,
}

/// Paginated response for list endpoints
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Data items
    pub data: Vec<T>,
    /// Pagination metadata
    pub meta: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;

        Self {
            data,
            meta: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
                has_more: page < total_pages,
            },
        }
    }

    /// Create a new paginated response from pagination params
    pub fn from_params(data: Vec<T>, params: &PaginationParams, total_items: u64) -> Self {
        let page = params.page.unwrap_or(1).max(1);
        let per_page = params.per_page.unwrap_or(20).max(1);

        Self::new(data, page, per_page, total_items)
    }
}
