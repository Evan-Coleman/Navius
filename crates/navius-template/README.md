# Navius Template

Template rendering system for the Navius framework with support for multiple template engines.

## Features

- **Multiple template engines**: Support for Handlebars, Tera, and Liquid template engines
- **Unified interface**: Consistent API regardless of the underlying template engine
- **Async-first design**: All operations are asynchronous for optimal performance
- **Template caching**: Built-in caching mechanism to improve rendering performance
- **Pluggable architecture**: Easy to add support for additional template engines
- **Engine delegation**: Route templates to different engines based on naming conventions

## Quick Start

Add the crate to your dependencies with desired template engine features:

```toml
[dependencies]
navius-template = { version = "0.1.0", features = ["handlebars"] }
```

Basic usage example:

```rust
use navius_template::error::TemplateResult;
use navius_template::registry::TemplateEngineRegistryBuilder;
use navius_template::handlebars::HandlebarsTemplateEngineFactory;
use serde_json::json;

#[tokio::main]
async fn main() -> TemplateResult<()> {
    // Create a template engine registry with Handlebars engine
    let registry = TemplateEngineRegistryBuilder::new()
        .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
        .build();
    
    // Create a Handlebars engine instance
    let mut engine = registry.create_engine("handlebars").await?;
    
    // Register a template
    engine.register_template_string("greeting", "Hello, {{name}}!").await?;
    
    // Render the template with context data
    let context = json!({ "name": "World" });
    let result = engine.render("greeting", &context).await?;
    
    assert_eq!(result, "Hello, World!");
    Ok(())
}
```

## Template Engines

The crate supports multiple template engines, each enabled via a feature flag:

- `handlebars` - [Handlebars](https://handlebarsjs.com/) templates (`{{`, `}}` syntax)
- `tera` - [Tera](https://tera.netlify.app/) templates (Jinja2-like syntax)
- `liquid` - [Liquid](https://shopify.github.io/liquid/) templates (used by Shopify)
- `all-engines` - Enables all supported template engines

## Advanced Usage

### Template Caching

The crate provides a caching layer to improve rendering performance:

```rust
use navius_template::cache::{CachedTemplateEngine, MemoryTemplateCache};
use std::time::Duration;

// Create a memory cache with 5-minute expiration and 100 max entries
let cache = MemoryTemplateCache::with_options(
    Duration::from_secs(300), 
    100
);

// Wrap any template engine with the cache
let cached_engine = CachedTemplateEngine::new(engine, cache);
```

### Engine Delegation

You can route templates to different engines based on name prefixes:

```rust
use navius_template::registry::DelegatingTemplateEngine;

// Create a delegating engine
let mut delegating_engine = DelegatingTemplateEngine::new();

// Register engines with different prefixes
delegating_engine.register_engine("hbs:", handlebars_engine);
delegating_engine.register_engine("tera:", tera_engine);

// Register and render templates using the appropriate prefix
delegating_engine.register_template_string("hbs:greeting", "Hello, {{name}}!").await?;
delegating_engine.register_template_string("tera:greeting", "Hello, {{ name }}!").await?;

// The engine will automatically route to the correct implementation
let result1 = delegating_engine.render("hbs:greeting", &context).await?;
let result2 = delegating_engine.render("tera:greeting", &context).await?;
```

### Registry Builder

Create a registry with multiple template engines:

```rust
let registry = TemplateEngineRegistryBuilder::new()
    .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
    .with_engine(Box::new(TeraTemplateEngineFactory::new()))
    .with_engine(Box::new(LiquidTemplateEngineFactory::new()))
    .build();
```

## Template Engine Interface

All template engines implement the `TemplateEngine` trait:

```rust
#[async_trait]
pub trait TemplateEngine: Send + Sync {
    /// Returns the name of the template engine
    async fn name(&self) -> TemplateResult<String>;
    
    /// Registers a template from a string
    async fn register_template_string(&mut self, name: &str, template: &str) -> TemplateResult<()>;
    
    /// Registers a template from a file
    async fn register_template_file(&mut self, name: &str, path: &str) -> TemplateResult<()>;
    
    /// Renders a registered template
    async fn render<T: Serialize + Send + Sync>(&self, name: &str, data: &T) -> TemplateResult<String>;
    
    /// Renders a template string directly
    async fn render_string<T: Serialize + Send + Sync>(&self, template: &str, data: &T) -> TemplateResult<String>;
    
    /// Gets all registered template names
    async fn get_template_names(&self) -> TemplateResult<HashSet<String>>;
    
    /// Checks if a template exists
    async fn has_template(&self, name: &str) -> TemplateResult<bool>;
    
    /// Removes a template
    async fn remove_template(&mut self, name: &str) -> TemplateResult<()>;
}
```

## Examples

More examples can be found in the `examples` directory:

- `basic_template.rs` - Basic usage with Handlebars
- `advanced_template.rs` - Advanced usage with multiple engines and caching

Run examples with:

```
cargo run --example basic_template --features handlebars
cargo run --example advanced_template --features "handlebars tera"
```

## License

Licensed under either of:

- MIT license
- Apache License, Version 2.0

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 