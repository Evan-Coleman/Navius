use crate::engine::{TemplateEngine, TemplateEngineFactory};
use crate::error::{TemplateError, TemplateResult};
use async_trait::async_trait;
use liquid::{ParserBuilder, ValueView};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::{Arc, Mutex};

/// Implementation of the [TemplateEngine] for Liquid templates.
pub struct LiquidTemplateEngine {
    templates: Arc<Mutex<HashMap<String, String>>>,
    parser: Arc<Mutex<liquid::Parser>>,
}

impl LiquidTemplateEngine {
    /// Creates a new [LiquidTemplateEngine] instance.
    pub fn new() -> Self {
        let parser = ParserBuilder::with_stdlib()
            .build()
            .expect("Failed to build Liquid parser");

        Self {
            templates: Arc::new(Mutex::new(HashMap::new())),
            parser: Arc::new(Mutex::new(parser)),
        }
    }

    /// Helper method to convert a Serialize to liquid::Object
    fn create_object<T: Serialize + Send + Sync>(
        &self,
        data: &T,
    ) -> TemplateResult<liquid::Object> {
        let value = serde_json::to_value(data).map_err(|e| {
            TemplateError::RenderError(format!("Failed to serialize data to JSON: {}", e))
        })?;

        // Convert serde_json::Value to liquid::Object
        self.json_to_liquid_object(value)
    }

    /// Helper to convert serde_json::Value to liquid::Object
    fn json_to_liquid_object(&self, value: serde_json::Value) -> TemplateResult<liquid::Object> {
        let mut object = liquid::Object::new();

        match value {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    object.insert(key, self.json_to_liquid_value(value)?);
                }
            }
            _ => {
                return Err(TemplateError::RenderError(
                    "Expected object for Liquid context".into(),
                ));
            }
        }

        Ok(object)
    }

    /// Helper to convert serde_json::Value to liquid::Value
    fn json_to_liquid_value(&self, value: serde_json::Value) -> TemplateResult<liquid::Value> {
        match value {
            serde_json::Value::Null => Ok(liquid::Value::Nil),
            serde_json::Value::Bool(b) => Ok(liquid::Value::scalar(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(liquid::Value::scalar(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(liquid::Value::scalar(f))
                } else {
                    Err(TemplateError::RenderError("Unsupported number type".into()))
                }
            }
            serde_json::Value::String(s) => Ok(liquid::Value::scalar(s)),
            serde_json::Value::Array(arr) => {
                let mut liquid_array = Vec::new();
                for item in arr {
                    liquid_array.push(self.json_to_liquid_value(item)?);
                }
                Ok(liquid::Value::Array(liquid_array))
            }
            serde_json::Value::Object(map) => {
                let mut liquid_object = liquid::Object::new();
                for (key, value) in map {
                    liquid_object.insert(key, self.json_to_liquid_value(value)?);
                }
                Ok(liquid::Value::Object(liquid_object))
            }
        }
    }
}

#[async_trait]
impl TemplateEngine for LiquidTemplateEngine {
    async fn name(&self) -> TemplateResult<String> {
        Ok("liquid".to_string())
    }

    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()> {
        let mut templates = self
            .templates
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on templates".into()))?;

        let parser = self.parser.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Liquid parser".into())
        })?;

        // Verify the template can be parsed
        parser.parse(template).map_err(|e| {
            TemplateError::RegistrationError(format!("Failed to parse Liquid template: {}", e))
        })?;

        templates.insert(name.to_string(), template.to_string());

        Ok(())
    }

    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()> {
        let template_content = fs::read_to_string(path).map_err(|e| {
            TemplateError::RegistrationError(format!("Failed to read template file: {}", e))
        })?;

        self.register_template_string(name, &template_content).await
    }

    async fn render<T: Serialize + Send + Sync>(
        &self,
        name: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let templates = self
            .templates
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on templates".into()))?;

        let template_str = templates
            .get(name)
            .ok_or_else(|| TemplateError::RenderError(format!("Template not found: {}", name)))?;

        self.render_string(template_str, data).await
    }

    async fn render_string<T: Serialize + Send + Sync>(
        &self,
        template: &str,
        data: &T,
    ) -> TemplateResult<String> {
        let parser = self.parser.lock().map_err(|_| {
            TemplateError::Internal("Failed to acquire lock on Liquid parser".into())
        })?;

        let template = parser.parse(template).map_err(|e| {
            TemplateError::RenderError(format!("Failed to parse Liquid template: {}", e))
        })?;

        let globals = self.create_object(data)?;

        template
            .render(&globals)
            .map_err(|e| TemplateError::RenderError(format!("Liquid render error: {}", e)))
    }

    async fn get_template_names(&self) -> TemplateResult<HashSet<String>> {
        let templates = self
            .templates
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on templates".into()))?;

        Ok(templates.keys().cloned().collect())
    }

    async fn has_template(&self, name: &str) -> TemplateResult<bool> {
        let templates = self
            .templates
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on templates".into()))?;

        Ok(templates.contains_key(name))
    }

    async fn remove_template(&mut self, name: &str) -> TemplateResult<()> {
        let mut templates = self
            .templates
            .lock()
            .map_err(|_| TemplateError::Internal("Failed to acquire lock on templates".into()))?;

        templates.remove(name);

        Ok(())
    }
}

/// Factory for creating [LiquidTemplateEngine] instances.
pub struct LiquidTemplateEngineFactory;

impl LiquidTemplateEngineFactory {
    /// Creates a new [LiquidTemplateEngineFactory] instance.
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TemplateEngineFactory for LiquidTemplateEngineFactory {
    async fn create(&self) -> TemplateResult<Box<dyn TemplateEngine>> {
        Ok(Box::new(LiquidTemplateEngine::new()))
    }

    fn name(&self) -> String {
        "liquid".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_render() -> TemplateResult<()> {
        let mut engine = LiquidTemplateEngine::new();

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
        let mut engine = LiquidTemplateEngine::new();

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
        let mut engine = LiquidTemplateEngine::new();

        // Register a template with iteration
        let template = "{% for item in items %}{{ item }}{% unless forloop.last %}, {% endunless %}{% endfor %}";
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
        let mut engine = LiquidTemplateEngine::new();

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
    async fn test_complex_data() -> TemplateResult<()> {
        let engine = LiquidTemplateEngine::new();

        // Test with complex nested data
        let template = "{{ user.name }} is {{ user.age }} years old and likes {% for hobby in user.hobbies %}{{ hobby }}{% unless forloop.last %}, {% endunless %}{% endfor %}.";

        let data = json!({
            "user": {
                "name": "John",
                "age": 30,
                "hobbies": ["reading", "cycling", "programming"]
            }
        });

        let result = engine.render_string(template, &data).await?;
        assert_eq!(
            result,
            "John is 30 years old and likes reading, cycling, programming."
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_factory() -> TemplateResult<()> {
        let factory = LiquidTemplateEngineFactory::new();
        assert_eq!(factory.name(), "liquid");

        let engine = factory.create().await?;
        assert_eq!(engine.name().await?, "liquid");

        Ok(())
    }
}
