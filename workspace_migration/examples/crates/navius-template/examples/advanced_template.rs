use navius_template::cache::{CachedTemplateEngine, MemoryTemplateCache};
use navius_template::engine::TemplateEngine;
use navius_template::error::TemplateResult;
use navius_template::registry::{DelegatingTemplateEngine, TemplateEngineRegistryBuilder};
use serde_json::json;
use std::time::Duration;

#[cfg(not(any(feature = "handlebars", feature = "tera")))]
fn main() {
    println!("This example requires both 'handlebars' and 'tera' features to be enabled.");
    println!(
        "Try running with: cargo run --example advanced_template --features \"handlebars tera\""
    );
}

#[cfg(all(feature = "handlebars", feature = "tera"))]
#[tokio::main]
async fn main() -> TemplateResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Import template engine implementations when features are enabled
    use navius_template::handlebars::HandlebarsTemplateEngineFactory;
    use navius_template::tera::TeraTemplateEngineFactory;

    println!("Navius Template - Advanced Example");
    println!("=================================");

    // Create a template engine registry with multiple engines
    let registry = TemplateEngineRegistryBuilder::new()
        .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
        .with_engine(Box::new(TeraTemplateEngineFactory::new()))
        .build();

    // Get available engines
    let available_engines = registry.available_engines();
    println!("Available engines: {:?}", available_engines);

    // Create individual engines
    let handlebars_engine = registry.create_engine("handlebars").await?;
    let tera_engine = registry.create_engine("tera").await?;

    // Create a delegating engine that dispatches to different engines based on template name prefix
    let mut delegating_engine = DelegatingTemplateEngine::new();
    delegating_engine.register_engine("hbs:", handlebars_engine);
    delegating_engine.register_engine("tera:", tera_engine);

    // Set up a template cache with a 5-minute expiration and 100 max entries
    let cache = MemoryTemplateCache::with_options(Duration::from_secs(300), 100);

    // Wrap the delegating engine with a cache
    let mut cached_engine = CachedTemplateEngine::new(delegating_engine, cache);

    // Register templates for different engines
    let handlebars_template = r#"
        <div class="handlebars">
            <h1>Handlebars Template</h1>
            <p>Hello, {{name}}!</p>
            {{#if list}}
            <ul>
                {{#each list}}
                <li>{{this}}</li>
                {{/each}}
            </ul>
            {{/if}}
        </div>
    "#;

    let tera_template = r#"
        <div class="tera">
            <h1>Tera Template</h1>
            <p>Hello, {{ name }}!</p>
            {% if list %}
            <ul>
                {% for item in list %}
                <li>{{ item }}</li>
                {% endfor %}
            </ul>
            {% endif %}
        </div>
    "#;

    // Register templates with appropriate prefixes
    cached_engine
        .register_template_string("hbs:welcome", handlebars_template)
        .await?;
    cached_engine
        .register_template_string("tera:welcome", tera_template)
        .await?;

    // Create context data
    let context = json!({
        "name": "Developer",
        "list": ["Item 1", "Item 2", "Item 3"]
    });

    // Render both templates and measure performance
    println!("\nRendering templates (first time, uncached)...");

    let start_time = std::time::Instant::now();
    let handlebars_result = cached_engine.render("hbs:welcome", &context).await?;
    let handlebars_time = start_time.elapsed();

    let start_time = std::time::Instant::now();
    let tera_result = cached_engine.render("tera:welcome", &context).await?;
    let tera_time = start_time.elapsed();

    println!("\nHandlebars template rendered in: {:?}", handlebars_time);
    println!("Tera template rendered in: {:?}", tera_time);

    // Render again to demonstrate caching
    println!("\nRendering templates again (cached)...");

    let start_time = std::time::Instant::now();
    let _ = cached_engine.render("hbs:welcome", &context).await?;
    let handlebars_cached_time = start_time.elapsed();

    let start_time = std::time::Instant::now();
    let _ = cached_engine.render("tera:welcome", &context).await?;
    let tera_cached_time = start_time.elapsed();

    println!(
        "Handlebars template rendered in: {:?} (cached)",
        handlebars_cached_time
    );
    println!("Tera template rendered in: {:?} (cached)", tera_cached_time);

    // Show performance improvement
    let handlebars_improvement =
        handlebars_time.as_nanos() as f64 / handlebars_cached_time.as_nanos() as f64;
    let tera_improvement = tera_time.as_nanos() as f64 / tera_cached_time.as_nanos() as f64;

    println!("\nPerformance improvement:");
    println!(
        "Handlebars: {:.2}x faster with caching",
        handlebars_improvement
    );
    println!("Tera: {:.2}x faster with caching", tera_improvement);

    // Show the rendered templates
    println!("\nHandlebars template output:");
    println!("{}", handlebars_result);

    println!("\nTera template output:");
    println!("{}", tera_result);

    Ok(())
}
