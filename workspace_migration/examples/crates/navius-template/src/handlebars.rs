use crate::engine::{TemplateEngine, TemplateEngineFactory};
use crate::error::{TemplateError, TemplateResult};
use async_trait::async_trait;
use handlebars::Handlebars;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Implementation of the [TemplateEngine] for Handlebars templates.
pub struct HandlebarsTemplateEngine {
    engine: Arc<Mutex<Handlebars<'static>>>,
}

impl HandlebarsTemplateEngine {
    /// Creates a new [HandlebarsTemplateEngine] instance.
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        // Enable strict mode for better error reporting
        handlebars.set_strict_mode(true);

        Self {
            engine: Arc::new(Mutex::new(handlebars)),
        }
    }
}

#[async_trait]
impl TemplateEngine for HandlebarsTemplateEngine {
    async fn name(&self) -> TemplateResult<String> {
        Ok("handlebars".to_string())
    }

    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        let mut engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        engine
            .register_template_string(name, template)
            .map_err(|e| {
                TemplateError::RegistrationError(format!(
                    "Failed to register Handlebars template: {}",
                    e
                ))
            })?;

        Ok(())
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        let mut engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        engine.register_template_file(name, path).map_err(|e| {
            TemplateError::RegistrationError(format!(
                "Failed to register Handlebars template file: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn render<T: Serialize + Send + Sync>(
        &self,
        name: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        engine
            .render(name, data)
            .map_err(|e| TemplateError::RenderError(format!("Handlebars render error: {}", e)))
    }

    async fn render_string<T: Serialize + Send + Sync>(
        &self,
        template: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        let render_result = engine
            .render_template(template, data)
            .map_err(|e| TemplateError::RenderError(format!("Handlebars render error: {}", e)))?;

        Ok(render_result)
    }

    async fn get_template_names(&self) -> TemplateResult<HashSet<String>> {
        let engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        Ok(engine.get_templates().keys().cloned().collect())
    }

    async fn has_template(&self, name: &str) -> TemplateResult<bool> {
        let engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        Ok(engine.has_template(name))
    }

    async fn remove_template(&mut self, name: &str) -> TemplateResult<()> {
        let mut engine = self.engine.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Handlebars engine".into())
        })?;

        engine.unregister_template(name);
        Ok(())
    }
}

/// Factory for creating [HandlebarsTemplateEngine] instances.
pub struct HandlebarsTemplateEngineFactory;

impl HandlebarsTemplateEngineFactory {
    /// Creates a new [HandlebarsTemplateEngineFactory] instance.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TemplateEngineFactory for HandlebarsTemplateEngineFactory {
    async fn create(&self) -> TemplateResult<Box<dyn TemplateEngine>> {
        Ok(Box::new(HandlebarsTemplateEngine::new()))
    }

    fn name(&self) -> String {
        "handlebars".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_render() -> TemplateResult<()> {
        let mut engine = HandlebarsTemplateEngine::new();

        // Register a template
        let template = "Hello, {{name}}!";
        engine
            .register_template_string("greeting", template)
            .await?;

        // Test rendering
        let data = json!({
            "name": "World"
        });

        let result = engine.render("greeting", &data).await?;
        assert_eq!(result, "Hello, World!");

        Ok(())
    }

    #[tokio::test]
    async fn test_conditional_template() -> TemplateResult<()> {
        let mut engine = HandlebarsTemplateEngine::new();

        // Register a template with conditionals
        let template = "{{#if show}}Visible{{else}}Hidden{{/if}}";
        engine
            .register_template_string("conditional", template)
            .await?;

        // Test with show=true
        let data_true = json!({
            "show": true
        });
        let result_true = engine.render("conditional", &data_true).await?;
        assert_eq!(result_true, "Visible");

        // Test with show=false
        let data_false = json!({
            "show": false
        });
        let result_false = engine.render("conditional", &data_false).await?;
        assert_eq!(result_false, "Hidden");

        Ok(())
    }

    #[tokio::test]
    async fn test_iteration() -> TemplateResult<()> {
        let mut engine = HandlebarsTemplateEngine::new();

        // Register a template with iteration
        let template = "{{#each items}}{{this}}{{#unless @last}}, {{/unless}}{{/each}}";
        engine.register_template_string("list", template).await?;

        // Test with list of items
        let data = json!({
            "items": ["Apple", "Banana", "Cherry"]
        });

        let result = engine.render("list", &data).await?;
        assert_eq!(result, "Apple, Banana, Cherry");

        Ok(())
    }

    #[tokio::test]
    async fn test_removing_template() -> TemplateResult<()> {
        let mut engine = HandlebarsTemplateEngine::new();

        // Register a template
        engine
            .register_template_string("test", "test content")
            .await?;

        // Verify it exists
        assert!(engine.has_template("test").await?);

        // Remove it
        engine.remove_template("test").await?;

        // Verify it's gone
        assert!(!engine.has_template("test").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_factory() -> TemplateResult<()> {
        let factory = HandlebarsTemplateEngineFactory::new();
        assert_eq!(factory.name(), "handlebars");

        let engine = factory.create().await?;
        assert_eq!(engine.name().await?, "handlebars");

        Ok(())
    }
}
