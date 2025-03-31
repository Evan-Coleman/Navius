use crate::error::TemplateResult;
use async_trait::async_trait;
use serde::Serialize;
use std::sync::Arc;

/// Trait for rendering templates with custom contexts
#[async_trait]
pub trait TemplateRenderer: Send + Sync {
    /// Render a template with the given context
    async fn render<T>(&self, name: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync;

    /// Render a template string with the given context
    async fn render_string<T>(&self, template: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync;
}

/// Represents a template engine
#[async_trait]
pub trait TemplateEngine: TemplateRenderer + Send + Sync {
    /// Register a template from a string
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()>;

    /// Register a template from a file
    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()>;

    /// Register templates from a directory
    async fn register_templates_directory(&mut self, dir: &str, ext: &str) -> TemplateResult<()>;

    /// Check if a template exists
    async fn has_template(&self, name: &str) -> bool;

    /// Get the name of the template engine
    fn engine_name(&self) -> &str;

    /// Clear all registered templates
    async fn clear_templates(&mut self) -> TemplateResult<()>;

    /// Get a list of registered template names
    async fn get_template_names(&self) -> TemplateResult<Vec<String>>;
}

/// Factory for creating template engines
#[async_trait]
pub trait TemplateEngineFactory: Send + Sync {
    /// Get the name of the template engine
    fn name(&self) -> &str;

    /// Create a new template engine instance
    async fn create_engine(&self) -> TemplateResult<Box<dyn TemplateEngine>>;
}

/// Configuration for a template engine
pub trait TemplateEngineConfig: Send + Sync {
    /// Get the name of the template engine
    fn engine_name(&self) -> &str;

    /// Clone the configuration
    fn box_clone(&self) -> Box<dyn TemplateEngineConfig>;
}

impl Clone for Box<dyn TemplateEngineConfig> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

/// Wrapper for a template engine that adds metrics
pub struct MetricsTemplateEngine {
    /// Wrapped template engine
    engine: Box<dyn TemplateEngine>,
    /// Metrics provider
    metrics: Option<Arc<dyn TemplateMetrics>>,
}

/// Interface for template metrics
#[async_trait]
pub trait TemplateMetrics: Send + Sync {
    /// Record template render time
    async fn record_render_time(&self, template_name: &str, duration_ms: f64);

    /// Record template registration
    async fn record_template_registration(&self, template_name: &str);

    /// Record template rendering error
    async fn record_render_error(&self, template_name: &str, error: &str);

    /// Record template cache hit
    async fn record_cache_hit(&self, template_name: &str);

    /// Record template cache miss
    async fn record_cache_miss(&self, template_name: &str);
}

impl MetricsTemplateEngine {
    /// Create a new metrics-enabled template engine
    pub fn new(engine: Box<dyn TemplateEngine>, metrics: Arc<dyn TemplateMetrics>) -> Self {
        Self {
            engine,
            metrics: Some(metrics),
        }
    }

    /// Create a new metrics-enabled template engine without metrics (passthrough)
    pub fn without_metrics(engine: Box<dyn TemplateEngine>) -> Self {
        Self {
            engine,
            metrics: None,
        }
    }
}

#[async_trait]
impl TemplateRenderer for MetricsTemplateEngine {
    async fn render<T>(&self, name: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync,
    {
        use std::time::Instant;

        if let Some(metrics) = &self.metrics {
            let start = Instant::now();
            let result = self.engine.render(name, context).await;
            let duration = start.elapsed();

            match &result {
                Ok(_) => {
                    metrics
                        .record_render_time(name, duration.as_secs_f64() * 1000.0)
                        .await;
                }
                Err(e) => {
                    metrics.record_render_error(name, &e.to_string()).await;
                }
            }

            result
        } else {
            self.engine.render(name, context).await
        }
    }

    async fn render_string<T>(&self, template: &str, context: &T) -> TemplateResult<String>
    where
        T: Serialize + Send + Sync,
    {
        use std::time::Instant;

        if let Some(metrics) = &self.metrics {
            let start = Instant::now();
            let result = self.engine.render_string(template, context).await;
            let duration = start.elapsed();

            match &result {
                Ok(_) => {
                    metrics
                        .record_render_time("<inline>", duration.as_secs_f64() * 1000.0)
                        .await;
                }
                Err(e) => {
                    metrics
                        .record_render_error("<inline>", &e.to_string())
                        .await;
                }
            }

            result
        } else {
            self.engine.render_string(template, context).await
        }
    }
}

#[async_trait]
impl TemplateEngine for MetricsTemplateEngine {
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        let result = self.engine.register_template_string(name, template).await;

        if let Some(metrics) = &self.metrics {
            if result.is_ok() {
                metrics.record_template_registration(name).await;
            }
        }

        result
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        let result = self.engine.register_template_file(name, path).await;

        if let Some(metrics) = &self.metrics {
            if result.is_ok() {
                metrics.record_template_registration(name).await;
            }
        }

        result
    }

    async fn register_templates_directory(&mut self, dir: &str, ext: &str) -> TemplateResult<()> {
        self.engine.register_templates_directory(dir, ext).await
    }

    async fn has_template(&self, name: &str) -> bool {
        self.engine.has_template(name).await
    }

    fn engine_name(&self) -> &str {
        self.engine.engine_name()
    }

    async fn clear_templates(&mut self) -> TemplateResult<()> {
        self.engine.clear_templates().await
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
    use std::sync::Mutex;

    struct MockTemplateEngine {
        name: String,
        templates: HashMap<String, String>,
    }

    #[async_trait]
    impl TemplateRenderer for MockTemplateEngine {
        async fn render<T>(&self, name: &str, context: &T) -> TemplateResult<String>
        where
            T: Serialize + Send + Sync,
        {
            if let Some(template) = self.templates.get(name) {
                // In a real implementation, this would apply the context to the template
                // Here we just return the template as-is
                Ok(template.clone())
            } else {
                Err(TemplateError::template_not_found(name))
            }
        }

        async fn render_string<T>(&self, template: &str, _context: &T) -> TemplateResult<String>
        where
            T: Serialize + Send + Sync,
        {
            // Just return the template string as-is
            Ok(template.to_string())
        }
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

    struct MockMetrics {
        render_times: Mutex<Vec<(String, f64)>>,
        registrations: Mutex<Vec<String>>,
        errors: Mutex<Vec<(String, String)>>,
    }

    impl MockMetrics {
        fn new() -> Self {
            Self {
                render_times: Mutex::new(Vec::new()),
                registrations: Mutex::new(Vec::new()),
                errors: Mutex::new(Vec::new()),
            }
        }

        fn get_render_times(&self) -> Vec<(String, f64)> {
            self.render_times.lock().unwrap().clone()
        }

        fn get_registrations(&self) -> Vec<String> {
            self.registrations.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl TemplateMetrics for MockMetrics {
        async fn record_render_time(&self, template_name: &str, duration_ms: f64) {
            self.render_times
                .lock()
                .unwrap()
                .push((template_name.to_string(), duration_ms));
        }

        async fn record_template_registration(&self, template_name: &str) {
            self.registrations
                .lock()
                .unwrap()
                .push(template_name.to_string());
        }

        async fn record_render_error(&self, template_name: &str, error: &str) {
            self.errors
                .lock()
                .unwrap()
                .push((template_name.to_string(), error.to_string()));
        }

        async fn record_cache_hit(&self, _template_name: &str) {}

        async fn record_cache_miss(&self, _template_name: &str) {}
    }

    #[tokio::test]
    async fn test_metrics_template_engine() {
        let metrics = Arc::new(MockMetrics::new());

        let mock_engine = MockTemplateEngine {
            name: "mock".to_string(),
            templates: HashMap::new(),
        };

        let mut metrics_engine = MetricsTemplateEngine::new(Box::new(mock_engine), metrics.clone());

        // Register a template
        metrics_engine
            .register_template_string("greeting", "Hello, {{name}}!")
            .await
            .unwrap();

        // Render the template
        let context = json!({
            "name": "World"
        });

        let _output = metrics_engine.render("greeting", &context).await.unwrap();

        // Check metrics
        let registrations = metrics.get_registrations();
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0], "greeting");

        let render_times = metrics.get_render_times();
        assert_eq!(render_times.len(), 1);
        assert_eq!(render_times[0].0, "greeting");
    }
}
