use navius_template::error::TemplateResult;
use navius_template::registry::TemplateEngineRegistryBuilder;
use serde_json::json;

#[cfg(not(feature = "handlebars"))]
fn main() {
    println!("This example requires the 'handlebars' feature to be enabled.");
    println!("Try running with: cargo run --example basic_template --features handlebars");
}

#[cfg(feature = "handlebars")]
#[tokio::main]
async fn main() -> TemplateResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Import handlebars implementation when feature is enabled
    use navius_template::handlebars::HandlebarsTemplateEngineFactory;

    println!("Navius Template - Basic Example");
    println!("===============================");

    // Create a template engine registry
    let registry = TemplateEngineRegistryBuilder::new()
        .with_engine(Box::new(HandlebarsTemplateEngineFactory::new()))
        .build();

    // Get available engines
    let available_engines = registry.available_engines();
    println!("Available engines: {:?}", available_engines);

    // Create an engine
    let mut engine = registry.create_engine("handlebars").await?;

    // Define a template
    let template_content = r#"
        <h1>Hello, {{name}}!</h1>
        <p>Welcome to {{project_name}}.</p>
        
        {{#if show_features}}
        <h2>Features:</h2>
        <ul>
            {{#each features}}
            <li>{{this}}</li>
            {{/each}}
        </ul>
        {{/if}}
    "#;

    // Register the template
    engine
        .register_template_string("welcome", template_content)
        .await?;

    // Create the template context
    let context = json!({
        "name": "World",
        "project_name": "Navius Template System",
        "show_features": true,
        "features": [
            "Multiple template engines",
            "Unified interface",
            "Async rendering",
            "Template caching"
        ]
    });

    // Render the template
    println!("\nRendering template 'welcome'...\n");
    let result = engine.render("welcome", &context).await?;
    println!("{}", result);

    // Render a template string directly
    println!("\nRendering a template string...\n");
    let inline_template = "The current time is {{time}}.";
    let inline_context = json!({
        "time": chrono::Local::now().format("%H:%M:%S").to_string()
    });

    let result = engine
        .render_string(inline_template, &inline_context)
        .await?;
    println!("{}", result);

    // Show available templates
    let template_names = engine.get_template_names().await?;
    println!("\nRegistered templates: {:?}", template_names);

    Ok(())
}
