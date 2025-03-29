//! HTTP client functionality for the Navius framework.
//!
//! This module provides HTTP client functionality using Reqwest.

use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::Duration;

/// HTTP client for making requests.
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: Option<String>,
    default_headers: HeaderMap,
}

impl HttpClient {
    /// Create a new HTTP client.
    pub fn new() -> Self {
        Self {
            inner: reqwest::Client::new(),
            base_url: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// Create a new HTTP client with a custom configuration.
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            inner: client,
            base_url: None,
            default_headers: HeaderMap::new(),
        }
    }

    /// Create a new HTTP client builder.
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// Set the base URL for the client.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the default headers for the client.
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.default_headers = headers;
        self
    }

    /// Add a default header to the client.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        let header_name = HeaderName::from_bytes(name.as_ref().as_bytes())
            .map_err(|e| Error::validation(format!("Invalid header name: {}", e)))?;

        let header_value = HeaderValue::from_str(value.as_ref())
            .map_err(|e| Error::validation(format!("Invalid header value: {}", e)))?;

        self.default_headers.insert(header_name, header_value);
        Ok(self)
    }

    /// Create a GET request.
    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.get(url), self.default_headers.clone())
    }

    /// Create a POST request.
    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.post(url), self.default_headers.clone())
    }

    /// Create a PUT request.
    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.put(url), self.default_headers.clone())
    }

    /// Create a DELETE request.
    pub fn delete(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.delete(url), self.default_headers.clone())
    }

    /// Create a PATCH request.
    pub fn patch(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.patch(url), self.default_headers.clone())
    }

    /// Create a HEAD request.
    pub fn head(&self, url: impl AsRef<str>) -> RequestBuilder {
        let url = self.resolve_url(url.as_ref());
        RequestBuilder::new(self.inner.head(url), self.default_headers.clone())
    }

    /// Resolve a URL against the base URL, if set.
    fn resolve_url(&self, url: &str) -> String {
        if let Some(base_url) = &self.base_url {
            if url.starts_with("http://") || url.starts_with("https://") {
                url.to_string()
            } else {
                // Simple join for now - could be improved
                let base = base_url.trim_end_matches('/');
                let path = url.trim_start_matches('/');
                format!("{}/{}", base, path)
            }
        } else {
            url.to_string()
        }
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating HTTP clients.
pub struct HttpClientBuilder {
    timeout: Option<Duration>,
    user_agent: Option<String>,
    headers: HeaderMap,
    base_url: Option<String>,
    follow_redirects: bool,
    max_redirects: Option<usize>,
}

impl HttpClientBuilder {
    /// Create a new HTTP client builder.
    pub fn new() -> Self {
        Self {
            timeout: None,
            user_agent: None,
            headers: HeaderMap::new(),
            base_url: None,
            follow_redirects: true,
            max_redirects: None,
        }
    }

    /// Set the timeout for the client.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the user agent for the client.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set the base URL for the client.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Set the default headers for the client.
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Add a default header to the client.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        let header_name = HeaderName::from_bytes(name.as_ref().as_bytes())
            .map_err(|e| Error::validation(format!("Invalid header name: {}", e)))?;

        let header_value = HeaderValue::from_str(value.as_ref())
            .map_err(|e| Error::validation(format!("Invalid header value: {}", e)))?;

        self.headers.insert(header_name, header_value);
        Ok(self)
    }

    /// Set whether to follow redirects.
    pub fn follow_redirects(mut self, follow: bool) -> Self {
        self.follow_redirects = follow;
        self
    }

    /// Set the maximum number of redirects to follow.
    pub fn max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = Some(max);
        self
    }

    /// Build the HTTP client.
    pub fn build(self) -> Result<HttpClient> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(timeout) = self.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        if let Some(user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        if let Some(max) = self.max_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(max));
        } else if !self.follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
        }

        let client = client_builder
            .build()
            .map_err(|e| Error::configuration(format!("Failed to build HTTP client: {}", e)))?;

        let mut http_client = HttpClient::with_client(client);
        http_client.default_headers = self.headers;

        if let Some(base_url) = self.base_url {
            http_client.base_url = Some(base_url);
        }

        Ok(http_client)
    }
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating HTTP requests.
pub struct RequestBuilder {
    inner: reqwest::RequestBuilder,
    default_headers: HeaderMap,
}

impl RequestBuilder {
    /// Create a new request builder.
    fn new(inner: reqwest::RequestBuilder, default_headers: HeaderMap) -> Self {
        let mut builder = Self {
            inner,
            default_headers: HeaderMap::new(),
        };

        // Apply default headers
        for (name, value) in default_headers.iter() {
            builder.inner = builder.inner.header(name.clone(), value.clone());
        }

        builder
    }

    /// Add a header to the request.
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Result<Self> {
        let header_name = HeaderName::from_bytes(name.as_ref().as_bytes())
            .map_err(|e| Error::validation(format!("Invalid header name: {}", e)))?;

        let header_value = HeaderValue::from_str(value.as_ref())
            .map_err(|e| Error::validation(format!("Invalid header value: {}", e)))?;

        self.inner = self.inner.header(header_name, header_value);
        Ok(self)
    }

    /// Add headers to the request.
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.inner = self.inner.headers(headers);
        self
    }

    /// Add a query parameter to the request.
    pub fn query<T: serde::Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.inner = self.inner.query(query);
        self
    }

    /// Set the request body as JSON.
    pub fn json<T: serde::Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.inner = self.inner.json(json);
        self
    }

    /// Set the request body as form data.
    pub fn form<T: serde::Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.inner = self.inner.form(form);
        self
    }

    /// Set the request body as a string.
    pub fn body<T: Into<reqwest::Body>>(mut self, body: T) -> Self {
        self.inner = self.inner.body(body);
        self
    }

    /// Set the timeout for the request.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    /// Send the request and get the response.
    pub async fn send(self) -> Result<reqwest::Response> {
        self.inner
            .send()
            .await
            .map_err(|e| Error::internal(format!("HTTP client error: {}", e)))
    }

    /// Send the request and parse the response as JSON.
    pub async fn get_json<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        let response = self.send().await?;

        if !response.status().is_success() {
            return Err(Error::http(
                response.status().as_u16(),
                format!("Request failed with status: {}", response.status()),
            ));
        }

        response
            .json()
            .await
            .map_err(|e| Error::response(format!("Failed to parse JSON response: {}", e)))
    }

    /// Send the request and return the response as text.
    pub async fn text(self) -> Result<String> {
        let response = self.send().await?;

        if !response.status().is_success() {
            return Err(Error::http(
                response.status().as_u16(),
                format!("Request failed with status: {}", response.status()),
            ));
        }

        response
            .text()
            .await
            .map_err(|e| Error::response(format!("Failed to parse text response: {}", e)))
    }

    /// Send the request and return the response as bytes.
    pub async fn bytes(self) -> Result<bytes::Bytes> {
        let response = self.send().await?;

        if !response.status().is_success() {
            return Err(Error::http(
                response.status().as_u16(),
                format!("Request failed with status: {}", response.status()),
            ));
        }

        response
            .bytes()
            .await
            .map_err(|e| Error::response(format!("Failed to parse bytes response: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Matcher, mock};

    #[tokio::test]
    async fn test_client_get() {
        let url = mockito::server_url();
        let _m = mock("GET", "/test")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"key": "value"}"#)
            .create();

        let client = HttpClient::new().with_base_url(&url);
        let response = client.get("/test").send().await.unwrap();

        assert!(response.status().is_success());
        assert_eq!(response.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn test_client_json() {
        let url = mockito::server_url();
        let _m = mock("GET", "/test-json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"key": "value"}"#)
            .create();

        let client = HttpClient::new().with_base_url(&url);
        let response: HashMap<String, String> = client.get("/test-json").get_json().await.unwrap();

        assert_eq!(response.get("key").unwrap(), "value");
    }

    #[tokio::test]
    async fn test_client_post() {
        let url = mockito::server_url();
        let _m = mock("POST", "/test-post")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id": 123}"#)
            .match_body(Matcher::Json(serde_json::json!({"name": "test"})))
            .create();

        let client = HttpClient::new().with_base_url(&url);
        let response = client
            .post("/test-post")
            .json(&serde_json::json!({"name": "test"}))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status().as_u16(), 201);
    }
}
