//! HTTP utility functions and extensions.

use crate::error::{Error, Result};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// Convert a string to a header value.
pub fn to_header_value<T: fmt::Display>(value: T) -> Result<HeaderValue> {
    HeaderValue::from_str(&value.to_string())
        .map_err(|e| Error::validation(format!("Invalid header value: {}", e)))
}

/// Convert a map of string key/values to a header map.
pub fn map_to_headers(map: &HashMap<String, String>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for (key, value) in map {
        let header_name = HeaderName::from_str(key)
            .map_err(|e| Error::validation(format!("Invalid header name '{}': {}", key, e)))?;

        let header_value = to_header_value(value)?;
        headers.insert(header_name, header_value);
    }
    Ok(headers)
}

/// Parse a URL from a string.
pub fn parse_url(url: &str) -> Result<reqwest::Url> {
    reqwest::Url::parse(url).map_err(|e| Error::validation(format!("Invalid URL '{}': {}", url, e)))
}

/// Join a base URL with a path.
pub fn join_url(base: &str, path: &str) -> Result<reqwest::Url> {
    let base_url = parse_url(base)?;
    base_url
        .join(path)
        .map_err(|e| Error::validation(format!("Invalid URL join '{}' + '{}': {}", base, path, e)))
}

/// Get the request ID from headers, or generate a new one if not present.
pub fn get_request_id(headers: &HeaderMap) -> String {
    headers
        .get(navius_core::constants::headers::REQUEST_ID)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| navius_core::util::random_id().to_string())
}

/// HTTP methods as an enum for easier manipulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
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

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
            Method::Patch => write!(f, "PATCH"),
            Method::Head => write!(f, "HEAD"),
            Method::Options => write!(f, "OPTIONS"),
        }
    }
}

impl From<Method> for reqwest::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => reqwest::Method::GET,
            Method::Post => reqwest::Method::POST,
            Method::Put => reqwest::Method::PUT,
            Method::Delete => reqwest::Method::DELETE,
            Method::Patch => reqwest::Method::PATCH,
            Method::Head => reqwest::Method::HEAD,
            Method::Options => reqwest::Method::OPTIONS,
        }
    }
}

impl FromStr for Method {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Method::Get),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "PATCH" => Ok(Method::Patch),
            "HEAD" => Ok(Method::Head),
            "OPTIONS" => Ok(Method::Options),
            _ => Err(Error::validation(format!("Invalid HTTP method: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_header_value() {
        let value = to_header_value("test-value").unwrap();
        assert_eq!(value, HeaderValue::from_static("test-value"));

        let value = to_header_value(42).unwrap();
        assert_eq!(value, HeaderValue::from_static("42"));
    }

    #[test]
    fn test_map_to_headers() {
        let mut map = HashMap::new();
        map.insert("content-type".to_string(), "application/json".to_string());
        map.insert("x-request-id".to_string(), "123".to_string());

        let headers = map_to_headers(&map).unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers.get("content-type").unwrap(),
            HeaderValue::from_static("application/json")
        );
        assert_eq!(
            headers.get("x-request-id").unwrap(),
            HeaderValue::from_static("123")
        );
    }

    #[test]
    fn test_parse_url() {
        let url = parse_url("https://example.com").unwrap();
        assert_eq!(url.as_str(), "https://example.com/");

        let result = parse_url("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_join_url() {
        let url = join_url("https://example.com", "/api/users").unwrap();
        assert_eq!(url.as_str(), "https://example.com/api/users");

        let url = join_url("https://example.com/", "api/users").unwrap();
        assert_eq!(url.as_str(), "https://example.com/api/users");
    }

    #[test]
    fn test_method_display() {
        assert_eq!(Method::Get.to_string(), "GET");
        assert_eq!(Method::Post.to_string(), "POST");
    }

    #[test]
    fn test_method_from_str() {
        assert_eq!(Method::from_str("GET").unwrap(), Method::Get);
        assert_eq!(Method::from_str("post").unwrap(), Method::Post);
        assert!(Method::from_str("INVALID").is_err());
    }
}
