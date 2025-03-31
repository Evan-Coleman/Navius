use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::error::{PluginHealth, PluginResult};

/// Base trait for all plugin capabilities
pub trait Capability: Any + Send + Sync + Debug {
    /// Get the name of the capability
    fn name(&self) -> &'static str;

    /// Get the capability as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get the capability as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get the capability type ID
    fn capability_type(&self) -> &'static str;

    /// Get the capability ID
    fn id(&self) -> &str;

    /// Clone the capability as a boxed dyn Capability
    fn clone_capability(&self) -> Box<dyn Capability>;
}

/// Convenience macro for implementing the Capability trait
#[macro_export]
macro_rules! impl_capability {
    ($type:ty, $capability_type:expr) => {
        impl Capability for $type {
            fn capability_type(&self) -> &'static str {
                $capability_type
            }

            fn id(&self) -> &str {
                $capability_type
            }

            fn clone_capability(&self) -> Box<dyn Capability> {
                Box::new(self.clone())
            }
        }
    };
}

/// Helper function to downcast a capability to a specific type
pub fn downcast_capability<T: Capability + 'static>(capability: &dyn Capability) -> Option<&T> {
    capability.as_any().downcast_ref::<T>()
}

/// Helper function to downcast a capability to a specific type with mutability
pub fn downcast_capability_mut<T: Capability + 'static>(
    capability: &mut dyn Capability,
) -> Option<&mut T> {
    capability.as_any_mut().downcast_mut::<T>()
}

/// Logging capability for plugins
#[async_trait]
pub trait LoggingCapability: Capability + Send + Sync {
    /// Log a debug message
    fn debug(&self, message: &str);

    /// Log an info message
    fn info(&self, message: &str);

    /// Log a warning message
    fn warn(&self, message: &str);

    /// Log an error message
    fn error(&self, message: &str);

    /// Log a structured event
    fn log_event(&self, event_type: &str, data: HashMap<String, Value>);
}

/// Health check capability for plugins
#[async_trait]
pub trait HealthCheckCapability: Capability + Send + Sync {
    /// Check health of the capability
    async fn check_health(&self) -> PluginHealth;

    /// Get detailed health information
    async fn health_details(&self) -> HashMap<String, Value>;
}

/// Configuration capability for plugins
#[async_trait]
pub trait ConfigurationCapability: Capability + Send + Sync {
    /// Get available configuration keys
    fn get_config_keys(&self) -> Vec<String>;

    /// Get a configuration value as a string
    fn get_config_string(&self, key: &str) -> Option<String>;

    /// Get a configuration value as a number
    fn get_config_number(&self, key: &str) -> Option<f64>;

    /// Get a configuration value as a boolean
    fn get_config_bool(&self, key: &str) -> Option<bool>;

    /// Get a configuration value as a JSON value
    fn get_config_value(&self, key: &str) -> Option<Value>;

    /// Set a configuration value
    fn set_config<T: Serialize>(&mut self, key: &str, value: T) -> PluginResult<()>;
}

/// Storage capability for plugins
#[async_trait]
pub trait StorageCapability: Capability + Send + Sync {
    /// Store a value with the given key
    async fn store<T: Serialize + Send + Sync>(&self, key: &str, value: &T) -> PluginResult<()>;

    /// Retrieve a value by key
    async fn retrieve<T: for<'de> serde::Deserialize<'de> + Send + Sync>(
        &self,
        key: &str,
    ) -> PluginResult<Option<T>>;

    /// Delete a value by key
    async fn delete(&self, key: &str) -> PluginResult<bool>;

    /// List all keys with optional prefix
    async fn list_keys(&self, prefix: Option<&str>) -> PluginResult<Vec<String>>;

    /// Clear all values
    async fn clear(&self) -> PluginResult<()>;
}

/// Event capability for plugins
#[async_trait]
pub trait EventCapability: Capability + Send + Sync {
    /// Publish an event
    async fn publish<T: Serialize + Send + Sync>(&self, topic: &str, event: &T)
    -> PluginResult<()>;

    /// Subscribe to events on a topic
    async fn subscribe(
        &self,
        topic: &str,
        callback: Box<dyn Fn(Value) + Send + Sync>,
    ) -> PluginResult<String>;

    /// Unsubscribe from a topic
    async fn unsubscribe(&self, subscription_id: &str) -> PluginResult<()>;
}

/// HTTP capability for plugins
#[async_trait]
pub trait HttpCapability: Capability + Send + Sync {
    /// Perform an HTTP GET request
    async fn get(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> PluginResult<HttpResponse>;

    /// Perform an HTTP POST request
    async fn post<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<String, String>>,
    ) -> PluginResult<HttpResponse>;

    /// Perform an HTTP PUT request
    async fn put<T: Serialize + Send + Sync>(
        &self,
        url: &str,
        body: &T,
        headers: Option<HashMap<String, String>>,
    ) -> PluginResult<HttpResponse>;

    /// Perform an HTTP DELETE request
    async fn delete(
        &self,
        url: &str,
        headers: Option<HashMap<String, String>>,
    ) -> PluginResult<HttpResponse>;
}

/// HTTP response from an HTTP capability
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body as bytes
    pub body: Vec<u8>,
}

/// Routing capability for plugins
#[async_trait]
pub trait RoutingCapability: Capability + Send + Sync {
    /// Register a handler for a route
    fn register_route(
        &mut self,
        method: &str,
        path: &str,
        handler: Box<dyn Fn(HttpRequest) -> PluginResult<HttpResponse> + Send + Sync>,
    ) -> PluginResult<()>;

    /// Unregister a route
    fn unregister_route(&mut self, method: &str, path: &str) -> PluginResult<()>;

    /// List all registered routes
    fn list_routes(&self) -> Vec<(String, String)>;
}

/// HTTP request for route handlers
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method
    pub method: String,

    /// Request path
    pub path: String,

    /// Request headers
    pub headers: HashMap<String, String>,

    /// Request body as bytes
    pub body: Vec<u8>,

    /// Request query parameters
    pub query_params: HashMap<String, String>,

    /// Path parameters
    pub path_params: HashMap<String, String>,
}
