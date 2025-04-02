use crate::engine::{TemplateEngine, TemplateEngineFactory, TemplateRenderer};
use crate::error::{TemplateError, TemplateResult};
use erased_serde::Serialize as ErasedSerialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Registry for template engines
pub struct TemplateEngineRegistry {
    /// Available template engine factories
    factories: Arc<Mutex<HashMap<String, Box<dyn TemplateEngineFactory>>>>,
}

impl TemplateEngineRegistry {
    /// Create a new template engine registry
    pub fn new() -> Self {
        Self {
            factories: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a template engine factory
    pub fn register(&mut self, factory: Box<dyn TemplateEngineFactory>) -> TemplateResult<()> {
        let mut factories = self.factories.lock().unwrap();
        factories.insert(factory.name().to_string(), factory);
        Ok(())
    }

    /// Create a template engine by name
    pub async fn create_engine(&self, name: &str) -> TemplateResult<Box<dyn TemplateEngine>> {
        let factories = self.factories.lock().unwrap();

        match factories.get(name) {
            Some(factory) => factory.create_engine().await,
            None => Err(TemplateError::engine_not_found(name)),
        }
    }

    /// Get available engine names
    pub fn available_engines(&self) -> Vec<String> {
        let factories = self.factories.lock().unwrap();
        factories.keys().cloned().collect()
    }

    /// Check if an engine is available
    pub fn has_engine(&self, name: &str) -> bool {
        let factories = self.factories.lock().unwrap();
        factories.contains_key(name)
    }
}

impl Default for TemplateEngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for template engine registry
pub struct TemplateEngineRegistryBuilder {
    /// Registry being built
    registry: TemplateEngineRegistry,
}

impl TemplateEngineRegistryBuilder {
    /// Create a new template engine registry builder
    pub fn new() -> Self {
        Self {
            registry: TemplateEngineRegistry::new(),
        }
    }

    /// Register a template engine factory
    pub fn with_engine(mut self, factory: Box<dyn TemplateEngineFactory>) -> Self {
        let _ = self.registry.register(factory);
        self
    }

    /// Build the registry
    pub fn build(self) -> TemplateEngineRegistry {
        self.registry
    }
}

impl Default for TemplateEngineRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating template engines with fallback
pub struct FallbackTemplateEngineFactory {
    /// Name of this factory
    name: String,
    /// Primary factory
    primary: Box<dyn TemplateEngineFactory>,
    /// Fallback factory
    fallback: Box<dyn TemplateEngineFactory>,
}

impl FallbackTemplateEngineFactory {
    /// Create a new fallback template engine factory
    pub fn new(
        name: impl Into<String>,
        primary: Box<dyn TemplateEngineFactory>,
        fallback: Box<dyn TemplateEngineFactory>,
    ) -> Self {
        Self {
            name: name.into(),
            primary,
            fallback,
        }
    }
}

#[async_trait::async_trait]
impl TemplateEngineFactory for FallbackTemplateEngineFactory {
    fn name(&self) -> &str {
        &self.name
    }

    async fn create_engine(&self) -> TemplateResult<Box<dyn TemplateEngine>> {
        match self.primary.create_engine().await {
            Ok(engine) => Ok(engine),
            Err(_) => self.fallback.create_engine().await,
        }
    }
}

/// Template engine that delegates to another engine based on template name prefix
pub struct DelegatingTemplateEngine {
    name: String,
    default_engine: Box<dyn TemplateEngine + Send + Sync>,
    engines: HashMap<String, Box<dyn TemplateEngine + Send + Sync>>,
}

impl DelegatingTemplateEngine {
    /// Create a new delegating template engine
    pub fn new(default_engine: Box<dyn TemplateEngine + Send + Sync>) -> Self {
        Self {
            name: "delegating".to_string(),
            default_engine,
            engines: HashMap::new(),
        }
    }

    /// Add a delegate engine for a specific prefix
    pub fn with_delegate(
        mut self,
        prefix: impl Into<String>,
        engine: Box<dyn TemplateEngine + Send + Sync>,
    ) -> Self {
        self.engines.insert(prefix.into(), engine);
        self
    }

    /// Get the engine for a template name
    fn get_engine_for_template(&self, name: &str) -> &Box<dyn TemplateEngine + Send + Sync> {
        for (prefix, engine) in &self.engines {
            if name.starts_with(prefix) {
                return engine;
            }
        }

        &self.default_engine
    }

    /// Get the mutable engine for a template name
    fn get_engine_for_template_mut(
        &mut self,
        name: &str,
    ) -> &mut Box<dyn TemplateEngine + Send + Sync> {
        // First check if any prefix matches
        for (prefix, _) in &self.engines {
            if name.starts_with(prefix) {
                let prefix = prefix.clone();
                return self.engines.get_mut(&prefix).unwrap();
            }
        }

        // If no prefix matches, use the default engine
        &mut self.default_engine
    }

    /// Get the engine for a string template (always returns the default engine)
    fn get_engine_for_string(&self) -> &Box<dyn TemplateEngine + Send + Sync> {
        &self.default_engine
    }
}

#[async_trait::async_trait]
impl TemplateEngine for DelegatingTemplateEngine {
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        // Delegate to the right engine
        self.get_engine_for_template_mut(name)
            .register_template_string(name, template)
            .await
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        // Delegate to the right engine
        self.get_engine_for_template_mut(name)
            .register_template_file(name, path)
            .await
    }

    async fn register_templates_directory(&mut self, dir: &str, ext: &str) -> TemplateResult<()> {
        // First register with default engine
        self.default_engine
            .register_templates_directory(dir, ext)
            .await?;

        // Then register with all delegate engines
        for engine in self.engines.values_mut() {
            engine.register_templates_directory(dir, ext).await?;
        }

        Ok(())
    }

    async fn has_template(&self, name: &str) -> bool {
        self.get_engine_for_template(name).has_template(name).await
    }

    fn engine_name(&self) -> &str {
        &self.name
    }

    async fn clear_templates(&mut self) -> TemplateResult<()> {
        // Clear all engines
        for engine in self.engines.values_mut() {
            engine.clear_templates().await?;
        }

        // Clear default engine
        self.default_engine.clear_templates().await?;

        Ok(())
    }

    async fn get_template_names(&self) -> TemplateResult<Vec<String>> {
        let mut names = Vec::new();

        // Get names from all engines
        for engine in self.engines.values() {
            names.extend(engine.get_template_names().await?);
        }

        // Get names from default engine
        names.extend(self.default_engine.get_template_names().await?);

        // Remove duplicates
        names.sort();
        names.dedup();

        Ok(names)
    }
}

// Helper function to make dyn ErasedSerialize Send across threads by converting to json
fn to_json_value(context: &dyn ErasedSerialize) -> TemplateResult<serde_json::Value> {
    // Serialize to a Vec<u8> using serde_json
    let mut buffer = Vec::new();
    {
        let mut serializer = serde_json::Serializer::new(&mut buffer);
        let mut erased = <dyn erased_serde::Serializer>::erase(&mut serializer);
        context.erased_serialize(&mut erased).map_err(|e| {
            crate::error::TemplateError::render_error(
                "<context>",
                format!("Failed to serialize context: {}", e),
            )
        })?;
    }

    // Deserialize from the buffer
    serde_json::from_slice(&buffer).map_err(|e| {
        crate::error::TemplateError::render_error(
            "<context>",
            format!("Failed to deserialize context: {}", e),
        )
    })
}

impl TemplateRenderer for DelegatingTemplateEngine {
    fn render<'a, 'b>(
        &'a self,
        name: &'b str,
        context: &'b dyn ErasedSerialize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TemplateResult<String>> + Send + 'a>>
    {
        let engine = self.get_engine_for_template(name);
        let name_owned = name.to_string();

        // Convert context to JSON value to make it Send
        let context_result = to_json_value(context);
        match context_result {
            Ok(context_value) => {
                Box::pin(async move { engine.render(&name_owned, &context_value).await })
            }
            Err(e) => Box::pin(async move { Err(e) }),
        }
    }

    fn render_string<'a, 'b>(
        &'a self,
        template: &'b str,
        context: &'b dyn ErasedSerialize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = TemplateResult<String>> + Send + 'a>>
    {
        let engine = self.get_engine_for_string();
        let template_owned = template.to_string();

        // Convert context to JSON value to make it Send
        let context_result = to_json_value(context);
        match context_result {
            Ok(context_value) => {
                Box::pin(async move { engine.render_string(&template_owned, &context_value).await })
            }
            Err(e) => Box::pin(async move { Err(e) }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TemplateError;

    #[derive(Default)]
    struct MockTemplateEngine {
        name: String,
        templates: std::collections::HashMap<String, String>,
    }

    impl MockTemplateEngine {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                templates: std::collections::HashMap::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl TemplateRenderer for MockTemplateEngine {
        async fn render<T>(&self, name: &str, _context: &T) -> TemplateResult<String>
        where
            T: serde::Serialize + Send + Sync,
        {
            self.templates
                .get(name)
                .cloned()
                .ok_or_else(|| TemplateError::TemplateNotFound(name.to_string()))
        }

        async fn render_string<T>(&self, template: &str, _context: &T) -> TemplateResult<String>
        where
            T: serde::Serialize + Send + Sync,
        {
            Ok(template.to_string())
        }
    }

    #[async_trait::async_trait]
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
                .insert(name.to_string(), format!("mock file template: {}", name));
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

    struct MockTemplateEngineFactory {
        name: String,
        should_fail: bool,
    }

    impl MockTemplateEngineFactory {
        fn new(name: impl Into<String>, should_fail: bool) -> Self {
            Self {
                name: name.into(),
                should_fail,
            }
        }
    }

    #[async_trait::async_trait]
    impl TemplateEngineFactory for MockTemplateEngineFactory {
        fn name(&self) -> &str {
            &self.name
        }

        async fn create_engine(&self) -> TemplateResult<Box<dyn TemplateEngine>> {
            if self.should_fail {
                Err(TemplateError::engine_error(
                    &self.name,
                    "Failed to create engine".to_string(),
                ))
            } else {
                Ok(Box::new(MockTemplateEngine::new(self.name.clone())))
            }
        }
    }

    #[tokio::test]
    async fn test_template_registry() {
        let mut registry = TemplateEngineRegistry::new();

        // Register factories
        registry.register(Box::new(MockTemplateEngineFactory::new(
            "handlebars",
            false,
        )))?;
        registry.register(Box::new(MockTemplateEngineFactory::new("tera", false)))?;

        // Check available engines
        let available = registry.available_engines();
        assert_eq!(available.len(), 2);
        assert!(available.contains(&"handlebars".to_string()));
        assert!(available.contains(&"tera".to_string()));

        // Create an engine
        let mut engine = registry.create_engine("handlebars").await?;

        // Register and render a template
        engine
            .register_template_string("greeting", "Hello, {{name}}!")
            .await?;

        let result = engine
            .render("greeting", &serde_json::json!({"name": "World"}))
            .await?;
        assert_eq!(result, "Hello, {{name}}!");

        // Try to create a non-existent engine
        let result = registry.create_engine("missing").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TemplateError::EngineNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_fallback_factory() {
        let factory = FallbackTemplateEngineFactory::new(
            "fallback",
            Box::new(MockTemplateEngineFactory::new("primary", true)), // This will fail
            Box::new(MockTemplateEngineFactory::new("secondary", false)), // This should be used
        );

        let engine = factory.create_engine().await?;
        assert_eq!(engine.engine_name(), "secondary");
    }

    #[tokio::test]
    async fn test_delegating_engine() {
        let mut handlebars = MockTemplateEngine::new("handlebars");
        handlebars
            .register_template_string("greeting", "Hello from Handlebars!")
            .await?;

        let mut tera = MockTemplateEngine::new("tera");
        tera.register_template_string("tera:greeting", "Hello from Tera!")
            .await?;

        let delegating = DelegatingTemplateEngine::new(Box::new(handlebars))
            .with_delegate("tera:", Box::new(tera));

        // Render using default engine
        let result = delegating
            .render("greeting", &serde_json::json!({}))
            .await?;
        assert_eq!(result, "Hello from Handlebars!");

        // Render using delegate engine
        let result = delegating
            .render("tera:greeting", &serde_json::json!({}))
            .await?;
        assert_eq!(result, "Hello from Tera!");
    }

    #[tokio::test]
    async fn test_registry_builder() {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(MockTemplateEngineFactory::new(
                "handlebars",
                false,
            )))
            .with_engine(Box::new(MockTemplateEngineFactory::new("tera", false)))
            .build();

        assert!(registry.has_engine("handlebars"));
        assert!(registry.has_engine("tera"));
        assert!(!registry.has_engine("missing"));
    }
}
