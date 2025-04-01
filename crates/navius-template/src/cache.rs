use crate::engine::{TemplateEngine, TemplateMetrics, TemplateRenderer};
use crate::error::{TemplateError, TemplateResult};
use async_trait::async_trait;
use erased_serde::Serialize as ErasedSerialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, instrument, trace, warn};

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
#[derive(Debug)]
pub struct CachedTemplateEngine {
    /// Underlying template engine
    engine: Box<dyn TemplateEngine>,
    /// Template cache
    cache: Box<dyn TemplateCache>,
    /// Optional metrics provider
    metrics: Option<Arc<dyn TemplateMetrics>>,
}

impl CachedTemplateEngine {
    /// Create a new cached template engine
    pub fn new(
        engine: Box<dyn TemplateEngine>,
        cache: impl TemplateCache + 'static,
        metrics: Option<Arc<dyn TemplateMetrics>>,
    ) -> Self {
        Self {
            engine,
            cache: Box::new(cache),
            metrics,
        }
    }

    /// Generate a cache key for a template and context
    #[instrument(skip(self, context), level = "trace")]
    fn generate_cache_key(
        &self,
        template_name: &str,
        context: &dyn ErasedSerialize,
    ) -> TemplateResult<String> {
        let mut context_bytes = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut context_bytes);
        context
            .erased_serialize(&mut serializer)
            .map_err(|e| TemplateError::SerializationError(e.to_string()))?;
        let context_hash = format!("{:x}", md5::compute(context_bytes));
        Ok(format!("template:{}:{}", template_name, context_hash))
    }

    // Helper to access the underlying engine for tests if needed
    #[cfg(test)]
    fn underlying_engine(&self) -> &dyn TemplateEngine {
        &*self.engine
    }
}

#[async_trait]
impl TemplateEngine for CachedTemplateEngine {
    // --- TemplateEngine specific methods ---
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        self.cache.remove(name).await?; // Invalidate cache
        self.engine.register_template_string(name, template).await
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        self.cache.remove(name).await?; // Invalidate cache
        self.engine.register_template_file(name, path).await
    }

    async fn register_templates_directory(&mut self, dir: &str, ext: &str) -> TemplateResult<()> {
        self.cache.clear().await?; // Invalidate cache
        self.engine.register_templates_directory(dir, ext).await
    }

    async fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name).await
    }

    fn engine_name(&self) -> &str {
        // Delegate, maybe prefix with "Cached "?
        self.engine.engine_name()
    }

    async fn clear_templates(&mut self) -> TemplateResult<()> {
        self.cache.clear().await?;
        self.engine.clear_templates().await
    }

    async fn get_template_names(&self) -> TemplateResult<Vec<String>> {
        self.engine.get_template_names().await
    }
}

// --- Explicit implementation of TemplateRenderer ---
#[async_trait]
impl TemplateRenderer for CachedTemplateEngine {
    #[instrument(skip(self, context), level = "debug")]
    async fn render(&self, name: &str, context: &dyn ErasedSerialize) -> TemplateResult<String> {
        let cache_key = self.generate_cache_key(name, context)?;

        if let Some(cached_result) = self.cache.get(&cache_key).await {
            trace!(template_name = %name, cache_key = %cache_key, "Template cache hit");
            if let Some(metrics) = &self.metrics {
                metrics.record_cache_hit(name).await;
            }
            return Ok(cached_result);
        }

        trace!(template_name = %name, cache_key = %cache_key, "Template cache miss");
        if let Some(metrics) = &self.metrics {
            metrics.record_cache_miss(name).await;
        }

        let rendered_result = self.engine.render(name, context).await?;
        self.cache.set(&cache_key, rendered_result.clone()).await?;
        Ok(rendered_result)
    }

    #[instrument(skip(self, template, context), level = "debug")]
    async fn render_string(
        &self,
        template: &str,
        context: &dyn ErasedSerialize,
    ) -> TemplateResult<String> {
        trace!("Rendering inline string template, bypassing cache.");
        self.engine.render_string(template, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{MockMetrics, TemplateEngine, TemplateRenderer};
    use crate::error::TemplateError;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::fmt;
    use std::sync::Mutex;
    use std::time::Duration;

    // --- MockTemplateEngine needs update for erased_serde ---
    struct MockTemplateEngine {
        name: String,
        templates: Mutex<HashMap<String, String>>,
        render_count: Mutex<HashMap<String, usize>>,
        fail_render: bool,
    }

    // Implement Debug manually for MockTemplateEngine if needed
    impl fmt::Debug for MockTemplateEngine {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MockTemplateEngine")
                .field("name", &self.name)
                // Avoid locking mutex in debug if possible, or handle poison
                // .field("templates", &self.templates)
                // .field("render_count", &self.render_count)
                .field("fail_render", &self.fail_render)
                .finish()
        }
    }

    impl MockTemplateEngine {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                templates: Mutex::new(HashMap::new()),
                render_count: Mutex::new(HashMap::new()),
                fail_render: false,
            }
        }
        fn set_fail_render(&mut self, fail: bool) {
            self.fail_render = fail;
        }
        fn get_render_count(&self, name: &str) -> usize {
            let counts = self.render_count.lock().unwrap();
            *counts.get(name).unwrap_or(&0)
        }
        async fn add_template(&self, name: &str, content: &str) {
            let mut templates = self.templates.lock().unwrap();
            templates.insert(name.to_string(), content.to_string());
        }
    }

    // Implement TemplateEngine for the mock
    #[async_trait]
    impl TemplateEngine for MockTemplateEngine {
        async fn register_template_string(
            &mut self,
            name: &str,
            template: &str,
        ) -> TemplateResult<()> {
            let mut templates = self.templates.lock().unwrap();
            templates.insert(name.to_string(), template.to_string());
            Ok(())
        }

        async fn register_template_file(&mut self, name: &str, _path: &str) -> TemplateResult<()> {
            let mut templates = self.templates.lock().unwrap();
            templates.insert(name.to_string(), name.to_string()); // Mock content
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
            let templates = self.templates.lock().unwrap();
            templates.contains_key(name)
        }

        fn engine_name(&self) -> &str {
            &self.name
        }

        async fn clear_templates(&mut self) -> TemplateResult<()> {
            let mut templates = self.templates.lock().unwrap();
            templates.clear();
            Ok(())
        }

        async fn get_template_names(&self) -> TemplateResult<Vec<String>> {
            let templates = self.templates.lock().unwrap();
            Ok(templates.keys().cloned().collect())
        }
    }

    // Implement TemplateRenderer for the mock using erased_serde
    #[async_trait]
    impl TemplateRenderer for MockTemplateEngine {
        async fn render(
            &self,
            name: &str,
            _context: &dyn ErasedSerialize, // Use erased type
        ) -> TemplateResult<String> {
            if self.fail_render {
                // Needs name and message
                return Err(TemplateError::render_error(
                    name,
                    "Mock forced render error",
                ));
            }
            let templates = self.templates.lock().unwrap();
            let mut counts = self.render_count.lock().unwrap();
            let count = counts.entry(name.to_string()).or_insert(0);
            *count += 1;

            match templates.get(name) {
                Some(template) => Ok(template.clone()),
                // Correctly takes one argument
                None => Err(TemplateError::template_not_found(name)),
            }
        }

        async fn render_string(
            &self,
            template: &str,
            _context: &dyn ErasedSerialize, // Use erased type
        ) -> TemplateResult<String> {
            if self.fail_render {
                // Needs a "name" for the error, using "<inline>" for strings
                return Err(TemplateError::render_error(
                    "<inline>",
                    "Mock forced render_string error",
                ));
            }
            Ok(template.to_string())
        }
    }

    // Mock TemplateCache implementation (simplified)
    #[derive(Debug)]
    struct MockTemplateCache {
        cache: Mutex<HashMap<String, String>>,
    }
    impl MockTemplateCache {
        fn new() -> Self {
            Self {
                cache: Mutex::new(HashMap::new()),
            }
        }
    }
    #[async_trait]
    impl TemplateCache for MockTemplateCache {
        async fn get(&self, key: &str) -> Option<String> {
            self.cache.lock().unwrap().get(key).cloned()
        }
        async fn set(&self, key: &str, value: String) -> TemplateResult<()> {
            self.cache.lock().unwrap().insert(key.to_string(), value);
            Ok(())
        }
        async fn remove(&self, key: &str) -> TemplateResult<()> {
            self.cache.lock().unwrap().remove(key);
            Ok(())
        }
        async fn clear(&self) -> TemplateResult<()> {
            self.cache.lock().unwrap().clear();
            Ok(())
        }
        async fn contains(&self, key: &str) -> bool {
            self.cache.lock().unwrap().contains_key(key)
        }
    }

    #[tokio::test]
    async fn test_cached_template_engine() {
        let mock_engine = MockTemplateEngine::new("mock");
        mock_engine.add_template("test", "Hello Template!").await;
        let mock_engine_box: Box<dyn TemplateEngine> = Box::new(mock_engine);

        let mock_cache = MockTemplateCache::new();
        let mock_metrics = Arc::new(MockMetrics::new());
        let cached_engine =
            CachedTemplateEngine::new(mock_engine_box, mock_cache, Some(mock_metrics.clone()));

        // Need to use erased_serde::serialize for the context
        let context = serde_json::json!({ "name": "World" });
        let erased_context = &context as &dyn ErasedSerialize;

        // First render - cache miss
        let result1 = cached_engine.render("test", erased_context).await.unwrap();
        assert_eq!(result1, "Hello Template!"); // Mock engine just returns template
        // Access mock engine correctly (might need helper or downcast)
        let mock_ref = cached_engine
            .underlying_engine()
            .downcast_ref::<MockTemplateEngine>()
            .unwrap();
        assert_eq!(mock_ref.get_render_count("test"), 1);
        assert_eq!(mock_metrics.get_cache_misses(), 1);
        assert_eq!(mock_metrics.get_cache_hits(), 0);

        // Second render - cache hit
        let result2 = cached_engine.render("test", erased_context).await.unwrap();
        assert_eq!(result2, "Hello Template!");
        assert_eq!(mock_ref.get_render_count("test"), 1); // Render NOT called again
        assert_eq!(mock_metrics.get_cache_misses(), 1);
        assert_eq!(mock_metrics.get_cache_hits(), 1);

        // Render different context - cache miss
        let context2 = serde_json::json!({ "name": "Universe" });
        let erased_context2 = &context2 as &dyn ErasedSerialize;
        let result3 = cached_engine.render("test", erased_context2).await.unwrap();
        assert_eq!(result3, "Hello Template!");
        assert_eq!(mock_ref.get_render_count("test"), 2); // Render called again
        assert_eq!(mock_metrics.get_cache_misses(), 2);
        assert_eq!(mock_metrics.get_cache_hits(), 1);
    }

    // ... other tests ...
}
