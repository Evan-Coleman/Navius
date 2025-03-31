use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mockall::predicate::*;
use mockall::*;
use serde_json::Value;

use crate::error::{TestError, TestResult};
use crate::mock::MockRegistry;

/// Error type for HTTP operations
#[derive(Debug, thiserror::Error)]
pub enum MockHttpError {
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// HTTP error
    #[error("HTTP error: code={code}, message={message}")]
    HttpError {
        /// HTTP status code
        code: u16,
        /// Error message
        message: String,
    },

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    /// Invalid URL
    #[error("Invalid URL: {0}")]
    InvalidUrlError(String),

    /// Other errors
    #[error("Other error: {0}")]
    OtherError(String),
}

/// HTTP request method
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET method
    Get,
    /// POST method
    Post,
    /// PUT method
    Put,
    /// DELETE method
    Delete,
    /// PATCH method
    Patch,
    /// HEAD method
    Head,
    /// OPTIONS method
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// HTTP request body
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestBody {
    /// Empty body
    Empty,
    /// Text body
    Text(String),
    /// JSON body
    Json(String),
    /// Binary body
    Binary(Vec<u8>),
    /// Form data
    Form(HashMap<String, String>),
}

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: ResponseBody,
}

/// HTTP response body
#[derive(Debug, Clone)]
pub enum ResponseBody {
    /// Empty body
    Empty,
    /// Text body
    Text(String),
    /// JSON body
    Json(Value),
    /// Binary body
    Binary(Vec<u8>),
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status: u16) -> Self {
        Self {
            status,
            headers: HashMap::new(),
            body: ResponseBody::Empty,
        }
    }

    /// Set a header on the response
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set the body as text
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.body = ResponseBody::Text(text.into());
        self
    }

    /// Set the body as JSON
    pub fn with_json(mut self, json: Value) -> Self {
        self.body = ResponseBody::Json(json);
        self
    }

    /// Set the body as binary
    pub fn with_binary(mut self, data: impl Into<Vec<u8>>) -> Self {
        self.body = ResponseBody::Binary(data.into());
        self
    }

    /// Create a success (200) response
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Create a created (201) response
    pub fn created() -> Self {
        Self::new(201)
    }

    /// Create a no content (204) response
    pub fn no_content() -> Self {
        Self::new(204)
    }

    /// Create a bad request (400) response
    pub fn bad_request() -> Self {
        Self::new(400)
    }

    /// Create an unauthorized (401) response
    pub fn unauthorized() -> Self {
        Self::new(401)
    }

    /// Create a forbidden (403) response
    pub fn forbidden() -> Self {
        Self::new(403)
    }

    /// Create a not found (404) response
    pub fn not_found() -> Self {
        Self::new(404)
    }

    /// Create an internal server error (500) response
    pub fn internal_server_error() -> Self {
        Self::new(500)
    }

    /// Extract the body as text
    pub fn as_text(&self) -> TestResult<String> {
        match &self.body {
            ResponseBody::Text(text) => Ok(text.clone()),
            ResponseBody::Json(json) => Ok(json.to_string()),
            ResponseBody::Binary(data) => String::from_utf8(data.clone()).map_err(|e| {
                TestError::ConversionError(format!("Failed to convert binary data to text: {}", e))
            }),
            ResponseBody::Empty => Ok("".to_string()),
        }
    }

    /// Extract the body as JSON
    pub fn as_json(&self) -> TestResult<Value> {
        match &self.body {
            ResponseBody::Json(json) => Ok(json.clone()),
            ResponseBody::Text(text) => serde_json::from_str(text).map_err(|e| {
                TestError::ConversionError(format!("Failed to parse text as JSON: {}", e))
            }),
            ResponseBody::Binary(data) => {
                let text = String::from_utf8(data.clone()).map_err(|e| {
                    TestError::ConversionError(format!(
                        "Failed to convert binary data to text: {}",
                        e
                    ))
                })?;
                serde_json::from_str(&text).map_err(|e| {
                    TestError::ConversionError(format!("Failed to parse text as JSON: {}", e))
                })
            }
            ResponseBody::Empty => Ok(Value::Null),
        }
    }

    /// Extract the body as binary
    pub fn as_binary(&self) -> TestResult<Vec<u8>> {
        match &self.body {
            ResponseBody::Binary(data) => Ok(data.clone()),
            ResponseBody::Text(text) => Ok(text.as_bytes().to_vec()),
            ResponseBody::Json(json) => Ok(json.to_string().as_bytes().to_vec()),
            ResponseBody::Empty => Ok(Vec::new()),
        }
    }
}

/// HTTP client for making HTTP requests
#[automock]
pub trait HttpClient: Send + Sync {
    /// Send an HTTP request
    fn request(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
    ) -> Result<HttpResponse, MockHttpError>;

    /// Send a GET request
    fn get(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, MockHttpError>;

    /// Send a POST request
    fn post(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
    ) -> Result<HttpResponse, MockHttpError>;

    /// Send a PUT request
    fn put(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
    ) -> Result<HttpResponse, MockHttpError>;

    /// Send a DELETE request
    fn delete(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> Result<HttpResponse, MockHttpError>;

    /// Send a PATCH request
    fn patch(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
    ) -> Result<HttpResponse, MockHttpError>;
}

impl MockHttpClient {
    /// Create a new mock HTTP client
    pub fn new() -> Self {
        let mock = Self::default();
        mock
    }

    /// Register the mock with the registry
    pub fn register(self, registry: &MockRegistry) -> TestResult<Arc<Self>> {
        let arc_self = Arc::new(self);
        registry.register::<dyn HttpClient, Self>(arc_self.clone())?;
        Ok(arc_self)
    }

    /// Expect a request with the specified method, URL, and headers
    pub fn expect_request(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let method_clone = method.clone();
        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.clone();

        self_mut
            .expect_request()
            .with(predicate::function(move |m: &HttpMethod| {
                *m == method_clone
            }))
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .with(predicate::function(move |b: &Option<RequestBody>| {
                match (&body_clone, b) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                }
            }))
            .return_once(move |_, _, _, _| response);
    }

    /// Expect a GET request
    pub fn expect_get(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let url_clone = url.to_string();
        let headers_clone = headers.clone();

        self_mut
            .expect_get()
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .return_once(move |_, _| response);
    }

    /// Expect a POST request
    pub fn expect_post(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.clone();

        self_mut
            .expect_post()
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .with(predicate::function(move |b: &Option<RequestBody>| {
                match (&body_clone, b) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                }
            }))
            .return_once(move |_, _, _| response);
    }

    /// Expect a PUT request
    pub fn expect_put(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.clone();

        self_mut
            .expect_put()
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .with(predicate::function(move |b: &Option<RequestBody>| {
                match (&body_clone, b) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                }
            }))
            .return_once(move |_, _, _| response);
    }

    /// Expect a DELETE request
    pub fn expect_delete(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let url_clone = url.to_string();
        let headers_clone = headers.clone();

        self_mut
            .expect_delete()
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .return_once(move |_, _| response);
    }

    /// Expect a PATCH request
    pub fn expect_patch(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<RequestBody>,
        response: Result<HttpResponse, MockHttpError>,
    ) {
        let self_mut = unsafe { &mut *(self as *const Self as *mut Self) };

        let url_clone = url.to_string();
        let headers_clone = headers.clone();
        let body_clone = body.clone();

        self_mut
            .expect_patch()
            .with(predicate::function(move |u: &str| u == url_clone))
            .with(predicate::function(
                move |h: &Option<HashMap<String, String>>| match (&headers_clone, h) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                },
            ))
            .with(predicate::function(move |b: &Option<RequestBody>| {
                match (&body_clone, b) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => expected == actual,
                    _ => false,
                }
            }))
            .return_once(move |_, _, _| response);
    }
}

/// Trait for accessing a mock HTTP client in tests
pub trait HasMockHttpClient {
    /// Get the mock HTTP client
    fn http_client(&self) -> Arc<MockHttpClient>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mock_http_client() {
        let registry = MockRegistry::new();
        let http_client = MockHttpClient::new().register(&registry).unwrap();

        // Set up expectations
        http_client.expect_get(
            "https://api.example.com/users/1",
            None,
            Ok(HttpResponse::ok().with_json(json!({
                "id": 1,
                "name": "Test User"
            }))),
        );

        // Use the mock
        let response = http_client
            .get("https://api.example.com/users/1", None)
            .unwrap();
        assert_eq!(response.status, 200);

        let body = response.as_json().unwrap();
        assert_eq!(body["id"], 1);
        assert_eq!(body["name"], "Test User");

        // Verify all expectations have been met
        registry.verify().unwrap();
    }

    #[test]
    fn test_http_response_helpers() {
        // Test text conversion
        let text_response = HttpResponse::ok().with_text("Hello, world!");
        assert_eq!(text_response.as_text().unwrap(), "Hello, world!");

        // Test JSON conversion
        let json_response = HttpResponse::ok().with_json(json!({"message": "Success"}));
        assert_eq!(json_response.as_json().unwrap()["message"], "Success");

        // Test binary conversion
        let binary_response = HttpResponse::ok().with_binary(vec![1, 2, 3, 4]);
        assert_eq!(binary_response.as_binary().unwrap(), vec![1, 2, 3, 4]);

        // Test status code helpers
        assert_eq!(HttpResponse::ok().status, 200);
        assert_eq!(HttpResponse::created().status, 201);
        assert_eq!(HttpResponse::no_content().status, 204);
        assert_eq!(HttpResponse::bad_request().status, 400);
        assert_eq!(HttpResponse::unauthorized().status, 401);
        assert_eq!(HttpResponse::forbidden().status, 403);
        assert_eq!(HttpResponse::not_found().status, 404);
        assert_eq!(HttpResponse::internal_server_error().status, 500);
    }
}
