use axum::Json;
use axum::response::IntoResponse;
use serde::Serialize;

/// Standard API response envelope
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Response metadata
#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    /// Request ID for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// Create a new API response with just data
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    /// Create a new API response with data and request ID
    pub fn with_request_id(data: T, request_id: String) -> Self {
        Self {
            data,
            meta: Some(ResponseMeta {
                request_id: Some(request_id),
                additional: None,
            }),
        }
    }

    /// Add additional metadata to the response
    pub fn with_additional_meta(mut self, additional: serde_json::Value) -> Self {
        let meta = self.meta.get_or_insert(ResponseMeta {
            request_id: None,
            additional: None,
        });
        meta.additional = Some(additional);
        self
    }
}

/// Implementation of IntoResponse for ApiResponse
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

/// Convert a value into an ApiResponse
pub fn into_response<T>(data: T) -> ApiResponse<T> {
    ApiResponse::new(data)
}
