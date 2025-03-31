use crate::engine::TemplateEngine;
use crate::error::TemplateResult;
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Interface for a template cache
#[async_trait]
pub trait TemplateCache: Send + Sync {
    /// Get a cached template result
    async fn get(&self, key: &str) -> Option<String>;

    /// Set a cached template result
    async fn set(&self, key: &str, value: String) -> TemplateResult<()>;

    /// Remove a cached template result
    async fn remove(&self, key: &str) -> TemplateResult<()>;

    /// Clear all cached template results
    async fn clear(&self) -> TemplateResult<()>;

    /// Check if a key exists in the cache
    async fn contains(&self, key: &str) -> bool;
}

/// In-memory template cache
pub struct MemoryTemplateCache {
    /// Cached template results
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// Maximum age for cached entries
    max_age: Option<Duration>,
    /// Maximum entries in the cache
    max_entries: Option<usize>,
}

/// Entry in the template cache
struct CacheEntry {
    /// Cached template result
    value: String,
    /// When the entry was created
    created_at: Instant,
    /// When the entry was last accessed
    last_accessed: Instant,
}

impl MemoryTemplateCache {
    /// Create a new memory template cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_age: None,
            max_entries: None,
        }
    }

    /// Create a new memory template cache with a maximum age
    pub fn with_max_age(max_age: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_age: Some(max_age),
            max_entries: None,
        }
    }

    /// Create a new memory template cache with a maximum number of entries
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_age: None,
            max_entries: Some(max_entries),
        }
    }

    /// Create a new memory template cache with a maximum age and entries
    pub fn with_limits(max_age: Duration, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            max_age: Some(max_age),
            max_entries: Some(max_entries),
        }
    }

    /// Cleanup expired entries and enforce max entries
    fn cleanup(&self) -> TemplateResult<()> {
        let mut cache = self.cache.lock().unwrap();

        // Remove expired entries
        if let Some(max_age) = self.max_age {
            let now = Instant::now();
            cache.retain(|_, entry| now.duration_since(entry.created_at) < max_age);
        }

        // Enforce max entries
        if let Some(max_entries) = self.max_entries {
            if cache.len() > max_entries {
                // Sort entries by last accessed time
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

                // Remove oldest entries
                let to_remove = entries.len() - max_entries;
                for i in 0..to_remove {
                    cache.remove(entries[i].0);
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl TemplateCache for MemoryTemplateCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get_mut(key) {
            // Update last accessed time
            entry.last_accessed = Instant::now();

            // Check if expired
            if let Some(max_age) = self.max_age {
                if entry.last_accessed.duration_since(entry.created_at) > max_age {
                    cache.remove(key);
                    return None;
                }
            }

            Some(entry.value.clone())
        } else {
            None
        }
    }

    async fn set(&self, key: &str, value: String) -> TemplateResult<()> {
        let mut cache = self.cache.lock().unwrap();

        let now = Instant::now();
        cache.insert(
            key.to_string(),
            CacheEntry {
                value,
                created_at: now,
                last_accessed: now,
            },
        );

        // Run cleanup after inserting
        drop(cache);
        self.cleanup()?;

        Ok(())
    }

    async fn remove(&self, key: &str) -> TemplateResult<()> {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(key);
        Ok(())
    }

    async fn clear(&self) -> TemplateResult<()> {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        Ok(())
    }

    async fn contains(&self, key: &str) -> bool {
        let cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(key) {
            // Check if expired
            if let Some(max_age) = self.max_age {
                let now = Instant::now();
                if now.duration_since(entry.created_at) > max_age {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }
}

/// Template engine with caching
pub struct CachedTemplateEngine {
    /// Underlying template engine
    engine: Box<dyn TemplateEngine>,
    /// Template cache
    cache: Box<dyn TemplateCache>,
}

impl CachedTemplateEngine {
    /// Create a new cached template engine
    pub fn new(engine: Box<dyn TemplateEngine>, cache: impl TemplateCache + 'static) -> Self {
        Self {
            engine,
            cache: Box::new(cache),
        }
    }

    /// Generate a cache key for a template and context
    fn generate_cache_key<T>(&self, template_name: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync,
    {
        // Generate a key from the template name and serialized context
        let context_json = serde_json::to_string(context)
            .map_err(|e| crate::error::TemplateError::serialization_error(e.to_string()))?;

        Ok(format!("{}:{}", template_name, context_json))
    }
}

#[async_trait]
impl TemplateEngine for CachedTemplateEngine {
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        self.engine.register_template_string(name, template).await
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        self.engine.register_template_file(name, path).await
    }

    async fn register_templates_directory(&mut self, dir: &str, ext: &str) -> TemplateResult<()> {
        self.engine.register_templates_directory(dir, ext).await
    }

    async fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name).await
    }

    async fn render<T>(&self, name: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync,
    {
        // Generate cache key
        let cache_key = self.generate_cache_key(name, context)?;

        // Check cache
        if let Some(cached_result) = self.cache.get(&cache_key).await {
            return Ok(cached_result);
        }

        // Render template
        let result = self.engine.render(name, context).await?;

        // Cache result
        self.cache.set(&cache_key, result.clone()).await?;

        Ok(result)
    }

    async fn render_string<T>(&self, template: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync,
    {
        // For render_string, we don't use caching since the template is provided as a string
        // and might be different each time even with the same context
        self.engine.render_string(template, context).await
    }

    fn engine_name(&self) -> &str {
        self.engine.engine_name()
    }

    async fn clear_templates(&mut self) -> TemplateResult<()> {
        // Clear both templates and cache
        self.engine.clear_templates().await?;
        self.cache.clear().await?;
        Ok(())
    }

    async fn get_template_names(&self) -> TemplateResult<Vec<String>> {
        self.engine.get_template_names().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TemplateError;
    use serde_json::json;
    use std::collections::HashMap;

    struct MockTemplateEngine {
        name: String,
        templates: HashMap<String, String>,
        render_count: Mutex<HashMap<String, usize>>,
    }

    #[async_trait]
    impl TemplateEngine for MockTemplateEngine {
        async fn register_template_string(
            &mut self,
            name: &str,
            template: &str,
        ) -> TemplateResult<()> {
            self.templates
                .insert(name.to_string(), template.to_string());
            Ok(())
        }

        async fn register_template_file(&mut self, name: &str, _path: &str) -> TemplateResult<()> {
            self.templates
                .insert(name.to_string(), "Mock file template".to_string());
            Ok(())
        }

        async fn register_templates_directory(
            &mut self,
            _dir: &str,
            _ext: &str,
        ) -> TemplateResult<()> {
            Ok(())
        }

        async fn has_template(&self, name: &str) -> bool {
            self.templates.contains_key(name)
        }

        async fn render<T>(&self, name: &str, _context: &T) -> TemplateResult<String>
        where
            T: Serialize + Send + Sync,
        {
            // Increment render count
            let mut render_count = self.render_count.lock().unwrap();
            *render_count.entry(name.to_string()).or_insert(0) += 1;

            match self.templates.get(name) {
                Some(template) => Ok(template.clone()),
                None => Err(TemplateError::template_not_found(name)),
            }
        }

        async fn render_string<T>(&self, template: &str, _context: &T) -> TemplateResult<String>
        where
            T: Serialize + Send + Sync,
        {
            Ok(template.to_string())
        }

        fn engine_name(&self) -> &str {
            &self.name
        }

        async fn clear_templates(&mut self) -> TemplateResult<()> {
            self.templates.clear();
            Ok(())
        }

        async fn get_template_names(&self) -> TemplateResult<Vec<String>> {
            Ok(self.templates.keys().cloned().collect())
        }
    }

    #[tokio::test]
    async fn test_memory_template_cache() {
        let cache = MemoryTemplateCache::new();

        // Set a value
        cache.set("test", "value".to_string()).await.unwrap();

        // Get the value
        let value = cache.get("test").await.unwrap();
        assert_eq!(value, "value");

        // Check contains
        assert!(cache.contains("test").await);

        // Remove the value
        cache.remove("test").await.unwrap();

        // Check it's gone
        assert!(!cache.contains("test").await);
        assert!(cache.get("test").await.is_none());
    }

    #[tokio::test]
    async fn test_cached_template_engine() {
        let mock_engine = MockTemplateEngine {
            name: "mock".to_string(),
            templates: {
                let mut templates = HashMap::new();
                templates.insert("greeting".to_string(), "Hello, {{name}}!".to_string());
                templates
            },
            render_count: Mutex::new(HashMap::new()),
        };

        let engine = Box::new(mock_engine) as Box<dyn TemplateEngine>;
        let cache = MemoryTemplateCache::new();
        let cached_engine = CachedTemplateEngine::new(engine, cache);

        // First render - should call the underlying engine
        let context = json!({ "name": "World" });
        let result1 = cached_engine.render("greeting", &context).await.unwrap();
        assert_eq!(result1, "Hello, {{name}}!");

        // Second render with same context - should use cache
        let result2 = cached_engine.render("greeting", &context).await.unwrap();
        assert_eq!(result2, "Hello, {{name}}!");

        // Check render count - should be 1 since the second call used the cache
        let render_count = cached_engine
            .engine
            .downcast_ref::<MockTemplateEngine>()
            .unwrap()
            .render_count
            .lock()
            .unwrap()
            .get("greeting")
            .cloned()
            .unwrap_or(0);

        assert_eq!(render_count, 1);

        // Render with different context - should call the underlying engine again
        let context2 = json!({ "name": "Rust" });
        let _result3 = cached_engine.render("greeting", &context2).await.unwrap();

        // Check render count - should be 2 for different context
        let render_count = cached_engine
            .engine
            .downcast_ref::<MockTemplateEngine>()
            .unwrap()
            .render_count
            .lock()
            .unwrap()
            .get("greeting")
            .cloned()
            .unwrap_or(0);

        assert_eq!(render_count, 2);
    }

    #[tokio::test]
    async fn test_memory_cache_with_max_age() {
        let cache = MemoryTemplateCache::with_max_age(Duration::from_millis(50));

        // Set a value
        cache.set("test", "value".to_string()).await.unwrap();

        // Get the value immediately
        let value = cache.get("test").await.unwrap();
        assert_eq!(value, "value");

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(100));

        // Value should be expired
        assert!(cache.get("test").await.is_none());
    }
}
