use crate::engine::{TemplateEngine, TemplateEngineFactory};
use crate::error::{TemplateError, TemplateResult};
use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tera::{Context, Tera};

/// Implementation of the [TemplateEngine] for Tera templates.
pub struct TeraTemplateEngine {
    engine: Arc<Mutex<Tera>>,
    template_names: Arc<Mutex<HashSet<String>>>,
}

impl TeraTemplateEngine {
    /// Creates a new [TeraTemplateEngine] instance.
    pub fn new() -> Self {
        let tera = Tera::default();

        Self {
            engine: Arc::new(Mutex::new(tera)),
            template_names: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Helper to convert serde_json::Value to Tera Context
    fn create_context<T: Serialize + Send + Sync>(&self, data: &T) -> TemplateResult<Context> {
        let json_value = serde_json::to_value(data).map_err(|e| {
            TemplateError::RenderError(format!("Failed to serialize data to JSON: {}", e))
        })?;

        let mut context = Context::new();

        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                context.insert(key, &value);
            }
        }

        Ok(context)
    }
}

#[async_trait]
impl TemplateEngine for TeraTemplateEngine {
    async fn name(&self) -> TemplateResult<String> {
        Ok("tera".to_string())
    }

    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        let mut engine = self
            .engine
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on Tera engine".into()))?;

        let mut template_names = self.template_names.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on template names".into())
        })?;

        engine.add_raw_template(name, template).map_err(|e| {
            TemplateError::RegistrationError(format!("Failed to register Tera template: {}", e))
        })?;

        template_names.insert(name.to_string());

        Ok(())
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        let template_content = fs::read_to_string(path).map_err(|e| {
            TemplateError::RegistrationError(format!("Failed to read template file: {}", e))
        })?;

        let template_name = if name.is_empty() {
            Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| TemplateError::RegistrationError("Invalid template path".into()))?
                .to_string()
        } else {
            name.to_string()
        };

        self.register_template_string(&template_name, &template_content)
            .await
    }

    async fn render<T: Serialize + Send + Sync>(
        &self,
        name: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let engine = self
            .engine
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on Tera engine".into()))?;

        let context = self.create_context(data)?;

        engine
            .render(name, &context)
            .map_err(|e| TemplateError::RenderError(format!("Tera render error: {}", e)))
    }

    async fn render_string<T: Serialize + Send + Sync>(
        &self,
        template: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let engine = self
            .engine
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on Tera engine".into()))?;

        let context = self.create_context(data)?;

        let render_result = engine
            .render_str(template, &context)
            .map_err(|e| TemplateError::RenderError(format!("Tera render error: {}", e)))?;

        Ok(render_result)
    }

    async fn get_template_names(&self) -> TemplateResult<HashSet<String>> {
        let template_names = self.template_names.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on template names".into())
        })?;

        Ok(template_names.clone())
    }

    async fn has_template(&self, name: &str) -> TemplateResult<bool> {
        let template_names = self.template_names.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on template names".into())
        })?;

        Ok(template_names.contains(name))
    }

    async fn remove_template(&mut self, name: &str) -> TemplateResult<()> {
        // Note: Tera does not provide a direct way to remove templates
        // The workaround is to create a new engine and re-register all templates except the one to remove
        let template_names = {
            let mut names = self.template_names.lock().map_err(|_| {
                TemplateError::Internal("Failed to acquire lock on template names".into())
            })?;

            names.remove(name);
            names.clone()
        };

        if !template_names.is_empty() {
            let old_engine = self.engine.lock().map_err(|_| {
                TemplateError::Internal("Failed to acquire lock on Tera engine".into())
            })?;

            let mut new_engine = Tera::default();

            for template_name in &template_names {
                if let Ok(template) = old_engine.get_raw_template(template_name) {
                    new_engine
                        .add_raw_template(template_name, template)
                        .map_err(|e| {
                            TemplateError::Internal(format!(
                                "Failed to re-register template: {}",
                                e
                            ))
                        })?;
                }
            }

            drop(old_engine);

            *self.engine.lock().map_err(|_| {
                TemplateError::Internal("Failed to acquire lock on Tera engine".into())
            })? = new_engine;
        } else {
            // If we removed the last template, just create a fresh engine
            *self.engine.lock().map_err(|_| {
                TemplateError::Internal("Failed to acquire lock on Tera engine".into())
            })? = Tera::default();
        }

        Ok(())
    }
}

/// Factory for creating [TeraTemplateEngine] instances.
pub struct TeraTemplateEngineFactory;

impl TeraTemplateEngineFactory {
    /// Creates a new [TeraTemplateEngineFactory] instance.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TemplateEngineFactory for TeraTemplateEngineFactory {
    async fn create(&self) -> TemplateResult<Box<dyn TemplateEngine>> {
        Ok(Box::new(TeraTemplateEngine::new()))
    }

    fn name(&self) -> String {
        "tera".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_render() -> TemplateResult<()> {
        let mut engine = TeraTemplateEngine::new();

        // Register a template
        let template = "Hello, {{ name }}!";
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
        let mut engine = TeraTemplateEngine::new();

        // Register a template with conditionals
        let template = "{% if show %}Visible{% else %}Hidden{% endif %}";
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
        let mut engine = TeraTemplateEngine::new();

        // Register a template with iteration
        let template =
            "{% for item in items %}{{ item }}{% if not loop.last %}, {% endif %}{% endfor %}";
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
        let mut engine = TeraTemplateEngine::new();

        // Register two templates
        engine
            .register_template_string("test1", "test content 1")
            .await?;
        engine
            .register_template_string("test2", "test content 2")
            .await?;

        // Verify they exist
        assert!(engine.has_template("test1").await?);
        assert!(engine.has_template("test2").await?);

        // Remove one
        engine.remove_template("test1").await?;

        // Verify it's gone but the other remains
        assert!(!engine.has_template("test1").await?);
        assert!(engine.has_template("test2").await?);

        // Get template names
        let names = engine.get_template_names().await?;
        assert_eq!(names.len(), 1);
        assert!(names.contains("test2"));

        Ok(())
    }

    #[tokio::test]
    async fn test_render_string() -> TemplateResult<()> {
        let engine = TeraTemplateEngine::new();

        // Render a template string directly
        let template = "Hello, {{ name }}!";
        let data = json!({
            "name": "World"
        });

        let result = engine.render_string(template, &data).await?;
        assert_eq!(result, "Hello, World!");

        Ok(())
    }

    #[tokio::test]
    async fn test_factory() -> TemplateResult<()> {
        let factory = TeraTemplateEngineFactory::new();
        assert_eq!(factory.name(), "tera");

        let engine = factory.create().await?;
        assert_eq!(engine.name().await?, "tera");

        Ok(())
    }
}
