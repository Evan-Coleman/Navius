use crate::error::{PluginHealth, PluginResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

/// Represents metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Unique identifier for the plugin
    pub id: String,

    /// Version of the plugin
    pub version: String,

    /// Human-readable name for the plugin
    pub name: String,

    /// Description of the plugin's functionality
    pub description: String,

    /// Author of the plugin
    pub author: String,

    /// Optional tags for categorization
    pub tags: Vec<String>,

    /// Creation timestamp
    pub created_at: String,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();

        Self {
            id: id.into(),
            version: version.into(),
            name: name.into(),
            description: description.into(),
            author: author.into(),
            tags: Vec::new(),
            created_at: timestamp,
        }
    }

    /// Add a tag to the plugin metadata
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags to the plugin metadata
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }
}

/// Configuration for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Key-value pairs for plugin configuration
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }
}

impl PluginConfig {
    /// Create a new empty plugin configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a plugin configuration from a HashMap
    pub fn from_map(settings: HashMap<String, serde_json::Value>) -> Self {
        Self { settings }
    }

    /// Get a configuration value as a string
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.settings
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Get a configuration value as a number
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.settings.get(key).and_then(|v| v.as_f64())
    }

    /// Get a configuration value as a boolean
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.settings.get(key).and_then(|v| v.as_bool())
    }

    /// Get a configuration value as a JSON value
    pub fn get_value(&self, key: &str) -> Option<&serde_json::Value> {
        self.settings.get(key)
    }

    /// Set a configuration value
    pub fn set<T>(&mut self, key: impl Into<String>, value: T) -> &mut Self
    where
        T: Serialize,
    {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.settings.insert(key.into(), json_value);
        }
        self
    }

    /// Check if a configuration key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.settings.contains_key(key)
    }

    /// Get all configuration keys
    pub fn keys(&self) -> Vec<&String> {
        self.settings.keys().collect()
    }
}

/// Represents the lifecycle stage of a plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginLifecycleStage {
    /// Plugin has been created but not initialized
    Created,

    /// Plugin has been initialized but not started
    Initialized,

    /// Plugin is running
    Started,

    /// Plugin has been stopped
    Stopped,

    /// Plugin has been unloaded
    Unloaded,

    /// Plugin has failed and is in an error state
    Failed,
}

impl std::fmt::Display for PluginLifecycleStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginLifecycleStage::Created => write!(f, "Created"),
            PluginLifecycleStage::Initialized => write!(f, "Initialized"),
            PluginLifecycleStage::Started => write!(f, "Started"),
            PluginLifecycleStage::Stopped => write!(f, "Stopped"),
            PluginLifecycleStage::Unloaded => write!(f, "Unloaded"),
            PluginLifecycleStage::Failed => write!(f, "Failed"),
        }
    }
}

/// Information about a dependency on another plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    /// ID of the required plugin
    pub id: String,

    /// Required version of the plugin
    pub version: String,

    /// Whether this dependency is optional
    pub optional: bool,
}

impl PluginDependency {
    /// Create a new required plugin dependency
    pub fn new(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            optional: false,
        }
    }

    /// Create a new optional plugin dependency
    pub fn optional(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            optional: true,
        }
    }
}

/// Trait for handling plugin messages asynchronously
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a message sent to the plugin
    async fn handle_message(&self, message: Value) -> PluginResult<Option<Value>>;
}

/// Trait for handling plugin lifecycle operations asynchronously
#[async_trait::async_trait]
pub trait PluginLifecycle: Send + Sync {
    /// Initialize the plugin with the provided configuration
    async fn initialize(&mut self, config: PluginConfig) -> PluginResult<()>;

    /// Start the plugin
    async fn start(&mut self) -> PluginResult<()>;

    /// Stop the plugin
    async fn stop(&mut self) -> PluginResult<()>;

    /// Check the health of the plugin
    async fn health_check(&self) -> PluginHealth;
}

/// Represents a plugin in the system
pub trait Plugin: Send + Sync + Debug + MessageHandler + PluginLifecycle {
    /// Get the plugin ID
    fn id(&self) -> &str;

    /// Get the plugin version
    fn version(&self) -> &str;

    /// Get the plugin's metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Get the plugin's current lifecycle stage
    fn lifecycle_stage(&self) -> PluginLifecycleStage;

    /// Get plugin dependencies
    fn dependencies(&self) -> Vec<PluginDependency> {
        Vec::new()
    }

    /// Get plugin capabilities
    fn capabilities(&self) -> HashMap<String, Arc<dyn std::any::Any + Send + Sync>> {
        HashMap::new()
    }

    /// Get a specific capability by ID
    fn get_capability(&self, capability_id: &str) -> Option<Arc<dyn std::any::Any + Send + Sync>> {
        self.capabilities().get(capability_id).cloned()
    }
}

// The plugin macro has been moved to lib.rs to avoid redefinition
