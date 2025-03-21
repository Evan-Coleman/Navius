use reqwest::{Response, StatusCode};
use serde::Serialize;
use std::fmt::Debug;
use tracing::{debug, error, info, warn};

use crate::error::AppError;

/// Log the start of an API request
pub fn log_request_start(api_name: &str, url: &str) {
    info!("🔄 Fetching data from {} API: {}", api_name, url);
}

/// Log a successful API response with formatted data
pub fn log_response_simple<T>(api_name: &str, url: &str, data: &T)
where
    T: Serialize + Debug,
{
    // Convert to JSON value for easy field access
    let json_value = serde_json::to_value(data).unwrap_or(serde_json::Value::Null);

    // Extract fields for simple preview
    let mut preview = String::new();

    if let serde_json::Value::Object(map) = &json_value {
        // Get up to 3 fields for preview
        let fields: Vec<_> = map.iter().take(3).collect();

        for (i, (key, value)) in fields.iter().enumerate() {
            let display_value = match value {
                serde_json::Value::String(s) => {
                    // Truncate long strings
                    if s.len() > 40 {
                        format!("\"{}...\"", &s[..37])
                    } else {
                        format!("\"{}\"", s)
                    }
                }
                _ => value.to_string(),
            };

            preview.push_str(&format!("{}=\"{}\"", key, display_value));

            // Add separator unless it's the last field
            if i < fields.len() - 1 {
                preview.push_str(", ");
            }
        }
    } else {
        preview = format!("{:?}", json_value);
    }

    // Log success with basic fields
    info!("✅ Successfully fetched data from {}: {}", url, preview);

    // Full data at debug level
    debug!("📊 Complete response from {}: {:?}", api_name, data);
}

/// Log an API request error
pub fn log_request_error(api_name: &str, url: &str, error: &str) {
    error!(
        "❌ Failed to send request to {} API ({}): {}",
        api_name, url, error
    );
}

/// Log an API response error
pub fn log_response_error(api_name: &str, status: StatusCode) {
    error!("❌ {} API returned error status: {}", api_name, status);
}

/// Log cache operations
pub fn log_cache_hit(entity_type: &str, id: &str) {
    info!("🔄 Retrieved {} {} from cache", entity_type, id);
}

pub fn log_cache_miss(entity_type: &str, id: &str) {
    info!("❌ Cache miss for {} {}", entity_type, id);
}

pub fn log_cache_store(entity_type: &str, id: &str) {
    info!("💾 Stored {} {} in cache", entity_type, id);
}

/// Helper function to check API response status and return appropriate error
pub fn check_response_status(
    response: &Response,
    api_name: &str,
    entity_type: &str,
    id: impl ToString,
) -> Result<(), AppError> {
    let status = response.status();
    let _url = response.url().to_string();

    if status == StatusCode::NOT_FOUND {
        warn!(
            "❓ {} with ID {} not found in {} API",
            entity_type,
            id.to_string(),
            api_name
        );
        return Err(AppError::NotFound(format!(
            "{} with ID {} not found (HTTP {})",
            entity_type,
            id.to_string(),
            status.as_u16()
        )));
    }

    if !status.is_success() {
        error!("❌ {} API returned error status: {}", api_name, status);
        return Err(AppError::ExternalServiceError(format!(
            "{} API returned error status: HTTP {}",
            api_name,
            status.as_u16()
        )));
    }

    Ok(())
}

/// API call with logging
pub async fn api_call<T, F, Fut>(
    api_name: &str,
    url: &str,
    fetch_fn: F,
    entity_type: &str,
    id: impl ToString,
) -> Result<T, AppError>
where
    T: Serialize + Debug + for<'de> serde::Deserialize<'de>,
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
{
    // Log request start
    log_request_start(api_name, url);

    // Make request to external API
    let response = fetch_fn().await.map_err(|e| {
        log_request_error(api_name, url, &e.to_string());
        AppError::ExternalServiceError(format!("Failed to fetch data from {}: {}", url, e))
    })?;

    // Check response status
    check_response_status(&response, api_name, entity_type, id)?;

    // Parse response
    let data = response.json::<T>().await.map_err(|e| {
        error!("❌ Failed to parse {} API response: {}", api_name, e);
        AppError::ExternalServiceError(format!("Failed to parse {} API response: {}", api_name, e))
    })?;

    // Log success
    log_response_simple(api_name, url, &data);

    Ok(data)
}
