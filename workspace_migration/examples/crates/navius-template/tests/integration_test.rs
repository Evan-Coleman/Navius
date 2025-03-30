#[cfg(feature = "handlebars")]
mod handlebars_tests {
    use navius_template::engine::TemplateEngine;
    use navius_template::error::TemplateResult;
    use navius_template::handlebars::HandlebarsTemplateEngineFactory;
    use navius_template::registry::TemplateEngineRegistryBuilder;
    use serde_json::json;

    #[tokio::test]
    async fn test_handlebars_engine_creation() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
            .build();

        let available = registry.available_engines();
        assert!(available.contains(&"handlebars".to_string()));

        let engine = registry.create_engine("handlebars").await?;
        assert!(engine.name().await? == "handlebars");

        Ok(())
    }

    #[tokio::test]
    async fn test_handlebars_template_rendering() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
            .build();

        let mut engine = registry.create_engine("handlebars").await?;

        // Register a template
        let template = "Hello, {{name}}!";
        engine
            .register_template_string("greeting", template)
            .await?;

        // Render with context
        let context = json!({ "name": "World" });
        let result = engine.render("greeting", &context).await?;

        assert_eq!(result, "Hello, World!");
        Ok(())
    }

    #[tokio::test]
    async fn test_handlebars_complex_template() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
            .build();

        let mut engine = registry.create_engine("handlebars").await?;

        // Register a more complex template
        let template = r#"
            {{#if items.length}}
            <ul>
                {{#each items}}
                <li>{{this}}</li>
                {{/each}}
            </ul>
            {{else}}
            <p>No items found.</p>
            {{/if}}
        "#;

        engine.register_template_string("list", template).await?;

        // Test with items
        let context_with_items = json!({
            "items": ["Apple", "Banana", "Cherry"]
        });
        let result_with_items = engine.render("list", &context_with_items).await?;
        assert!(result_with_items.contains("<li>Apple</li>"));
        assert!(result_with_items.contains("<li>Banana</li>"));
        assert!(result_with_items.contains("<li>Cherry</li>"));

        // Test without items
        let context_without_items = json!({ "items": [] });
        let result_without_items = engine.render("list", &context_without_items).await?;
        assert!(result_without_items.contains("No items found."));

        Ok(())
    }
}

#[cfg(feature = "tera")]
mod tera_tests {
    use navius_template::engine::TemplateEngine;
    use navius_template::error::TemplateResult;
    use navius_template::registry::TemplateEngineRegistryBuilder;
    use navius_template::tera::TeraTemplateEngineFactory;
    use serde_json::json;

    #[tokio::test]
    async fn test_tera_engine_creation() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(TeraTemplateEngineFactory::new()))
            .build();

        let available = registry.available_engines();
        assert!(available.contains(&"tera".to_string()));

        let engine = registry.create_engine("tera").await?;
        assert!(engine.name().await? == "tera");

        Ok(())
    }

    #[tokio::test]
    async fn test_tera_template_rendering() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(TeraTemplateEngineFactory::new()))
            .build();

        let mut engine = registry.create_engine("tera").await?;

        // Register a template
        let template = "Hello, {{ name }}!";
        engine
            .register_template_string("greeting", template)
            .await?;

        // Render with context
        let context = json!({ "name": "World" });
        let result = engine.render("greeting", &context).await?;

        assert_eq!(result, "Hello, World!");
        Ok(())
    }

    #[tokio::test]
    async fn test_tera_complex_template() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(TeraTemplateEngineFactory::new()))
            .build();

        let mut engine = registry.create_engine("tera").await?;

        // Register a more complex template
        let template = r#"
            {% if items %}
            <ul>
                {% for item in items %}
                <li>{{ item }}</li>
                {% endfor %}
            </ul>
            {% else %}
            <p>No items found.</p>
            {% endif %}
        "#;

        engine.register_template_string("list", template).await?;

        // Test with items
        let context_with_items = json!({
            "items": ["Apple", "Banana", "Cherry"]
        });
        let result_with_items = engine.render("list", &context_with_items).await?;
        assert!(result_with_items.contains("<li>Apple</li>"));
        assert!(result_with_items.contains("<li>Banana</li>"));
        assert!(result_with_items.contains("<li>Cherry</li>"));

        // Test without items
        let context_without_items = json!({ "items": [] });
        let result_without_items = engine.render("list", &context_without_items).await?;
        assert!(result_without_items.contains("No items found."));

        Ok(())
    }
}

#[cfg(all(feature = "handlebars", feature = "tera"))]
mod multi_engine_tests {
    use navius_template::cache::{CachedTemplateEngine, MemoryTemplateCache};
    use navius_template::engine::TemplateEngine;
    use navius_template::error::TemplateResult;
    use navius_template::handlebars::HandlebarsTemplateEngineFactory;
    use navius_template::registry::{DelegatingTemplateEngine, TemplateEngineRegistryBuilder};
    use navius_template::tera::TeraTemplateEngineFactory;
    use serde_json::json;
    use std::time::Duration;

    #[tokio::test]
    async fn test_delegating_engine() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
            .with_engine(Box::new(TeraTemplateEngineFactory::new()))
            .build();

        let handlebars_engine = registry.create_engine("handlebars").await?;
        let tera_engine = registry.create_engine("tera").await?;

        let mut delegating_engine = DelegatingTemplateEngine::new();
        delegating_engine.register_engine("hbs:", handlebars_engine);
        delegating_engine.register_engine("tera:", tera_engine);

        // Register templates
        delegating_engine
            .register_template_string("hbs:greeting", "Hello, {{name}}!")
            .await?;
        delegating_engine
            .register_template_string("tera:greeting", "Hello, {{ name }}!")
            .await?;

        // Render templates
        let context = json!({ "name": "World" });
        let hbs_result = delegating_engine.render("hbs:greeting", &context).await?;
        let tera_result = delegating_engine.render("tera:greeting", &context).await?;

        assert_eq!(hbs_result, "Hello, World!");
        assert_eq!(tera_result, "Hello, World!");

        Ok(())
    }

    #[tokio::test]
    async fn test_cached_template_engine() -> TemplateResult<()> {
        let registry = TemplateEngineRegistryBuilder::new()
            .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
            .build();

        let engine = registry.create_engine("handlebars").await?;
        let cache = MemoryTemplateCache::with_options(Duration::from_secs(60), 100);
        let mut cached_engine = CachedTemplateEngine::new(engine, cache);

        // Register template
        cached_engine
            .register_template_string("greeting", "Hello, {{name}}!")
            .await?;

        // Render template
        let context = json!({ "name": "World" });

        // First render should not be cached
        let first_result = cached_engine.render("greeting", &context).await?;
        assert_eq!(first_result, "Hello, World!");

        // Second render should be cached
        let second_result = cached_engine.render("greeting", &context).await?;
        assert_eq!(second_result, "Hello, World!");

        // Check if template is in cache
        let cache_ref = cached_engine.cache();
        assert!(
            cache_ref
                .has("greeting:d6cd6520c1f66f98276e0be6a0116105")
                .await
        );

        Ok(())
    }
}
