use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Standard API response structure for all endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// The actual data being returned (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    /// Response metadata
    pub metadata: ResponseMetadata,

    /// HATEOAS links for navigation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,
}

/// Metadata for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// ISO 8601 formatted timestamp
    pub timestamp: DateTime<Utc>,

    /// Unique request ID for tracing
    pub request_id: Uuid,

    /// Optional pagination information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationInfo>,

    /// HTTP status code
    pub status: u16,

    /// Success or error message
    pub message: String,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current page number (1-based)
    pub page: u32,

    /// Number of items per page
    pub per_page: u32,

    /// Total number of items across all pages
    pub total_items: u64,

    /// Total number of pages
    pub total_pages: u32,
}

/// HATEOAS links for navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Links {
    /// Link to the current resource
    pub self_link: String,

    /// Link to the first page (if paginated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first: Option<String>,

    /// Link to the previous page (if paginated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,

    /// Link to the next page (if paginated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,

    /// Link to the last page (if paginated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last: Option<String>,
}

impl<T> ApiResponse<T> {
    /// Create a new success response with data
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            metadata: ResponseMetadata {
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
                pagination: None,
                status: StatusCode::OK.as_u16(),
                message: "Success".to_string(),
            },
            links: None,
        }
    }

    /// Create a new success response with no data
    pub fn success_no_content() -> ApiResponse<()> {
        ApiResponse {
            data: None,
            metadata: ResponseMetadata {
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
                pagination: None,
                status: StatusCode::NO_CONTENT.as_u16(),
                message: "Success".to_string(),
            },
            links: None,
        }
    }

    /// Create a new error response
    pub fn error(status: StatusCode, message: String) -> ApiResponse<()> {
        ApiResponse {
            data: None,
            metadata: ResponseMetadata {
                timestamp: Utc::now(),
                request_id: Uuid::new_v4(),
                pagination: None,
                status: status.as_u16(),
                message,
            },
            links: None,
        }
    }

    /// Add pagination information to the response
    pub fn with_pagination(mut self, page: u32, per_page: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (per_page as f64)).ceil() as u32;

        self.metadata.pagination = Some(PaginationInfo {
            page,
            per_page,
            total_items,
            total_pages,
        });

        self
    }

    /// Add HATEOAS links to the response
    pub fn with_links(mut self, self_link: String) -> Self {
        self.links = Some(Links {
            self_link,
            first: None,
            prev: None,
            next: None,
            last: None,
        });

        self
    }

    /// Add pagination links
    pub fn with_pagination_links(mut self, base_url: &str, page: u32, total_pages: u32) -> Self {
        if self.links.is_none() {
            self.links = Some(Links {
                self_link: format!(
                    "{}?page={}&per_page={}",
                    base_url,
                    page,
                    self.metadata.pagination.as_ref().map_or(20, |p| p.per_page)
                ),
                first: None,
                prev: None,
                next: None,
                last: None,
            });
        }

        let per_page = self.metadata.pagination.as_ref().map_or(20, |p| p.per_page);
        let links = self.links.as_mut().unwrap();

        // Add first page link
        links.first = Some(format!("{}?page=1&per_page={}", base_url, per_page));

        // Add last page link
        links.last = Some(format!(
            "{}?page={}&per_page={}",
            base_url, total_pages, per_page
        ));

        // Add previous page link if not on first page
        if page > 1 {
            links.prev = Some(format!(
                "{}?page={}&per_page={}",
                base_url,
                page - 1,
                per_page
            ));
        }

        // Add next page link if not on last page
        if page < total_pages {
            links.next = Some(format!(
                "{}?page={}&per_page={}",
                base_url,
                page + 1,
                per_page
            ));
        }

        self
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.metadata.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = ApiResponse::success("test data");

        assert!(response.data.is_some());
        assert_eq!(response.data.unwrap(), "test data");
        assert_eq!(response.metadata.status, 200);
        assert_eq!(response.metadata.message, "Success");
    }

    #[test]
    fn test_error_response() {
        let response = ApiResponse::error(StatusCode::BAD_REQUEST, "Invalid input".to_string());

        assert!(response.data.is_none());
        assert_eq!(response.metadata.status, 400);
        assert_eq!(response.metadata.message, "Invalid input");
    }

    #[test]
    fn test_pagination() {
        let response = ApiResponse::success(vec![1, 2, 3]).with_pagination(2, 10, 25);

        assert!(response.metadata.pagination.is_some());
        let pagination = response.metadata.pagination.unwrap();

        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.total_items, 25);
        assert_eq!(pagination.total_pages, 3);
    }

    #[test]
    fn test_links() {
        let response = ApiResponse::success(vec![1, 2, 3])
            .with_pagination(2, 10, 25)
            .with_pagination_links("/api/items", 2, 3);

        assert!(response.links.is_some());
        let links = response.links.unwrap();

        assert_eq!(links.self_link, "/api/items?page=2&per_page=10");
        assert_eq!(
            links.first,
            Some("/api/items?page=1&per_page=10".to_string())
        );
        assert_eq!(
            links.prev,
            Some("/api/items?page=1&per_page=10".to_string())
        );
        assert_eq!(
            links.next,
            Some("/api/items?page=3&per_page=10".to_string())
        );
        assert_eq!(
            links.last,
            Some("/api/items?page=3&per_page=10".to_string())
        );
    }

    #[test]
    fn test_into_response() {
        let response = ApiResponse::success("test data").into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let response = ApiResponse::error(StatusCode::BAD_REQUEST, "Invalid input".to_string())
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
