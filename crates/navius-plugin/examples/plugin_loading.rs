use navius_plugin::{PluginConfig, PluginLifecycleStage, PluginLoader, PluginRegistry, plugin};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("Plugin Loading Example");
    println!("=====================");

    // Create a plugin registry
    let registry = PluginRegistry::new();

    // Create a simple plugin using the builder
    let plugin1 = plugin!(
        "SimplePlugin1",
        "1.0.0",
        "A simple example plugin",
        "Navius Team"
    )
    .with_tag("example")
    .build();

    // Create a second plugin that depends on the first
    let plugin2 = plugin!(
        "SimplePlugin2",
        "1.0.0",
        "Another example plugin",
        "Navius Team"
    )
    .with_tag("example")
    .with_dependency("SimplePlugin1", "1.0.0")
    .build();

    // Register plugins
    println!("\nRegistering plugins...");
    let plugin1_id = registry.register_plugin(plugin1).await?;
    println!("Registered plugin: {}", plugin1_id);

    let plugin2_id = registry.register_plugin(plugin2).await?;
    println!("Registered plugin: {}", plugin2_id);

    // Create plugin loader
    let mut loader = PluginLoader::new();

    // Add search paths
    loader.add_search_path("./plugins");

    // Find available plugin libraries
    println!("\nScanning for plugin libraries...");
    let plugins = loader.find_plugins()?;

    if plugins.is_empty() {
        println!(
            "No plugin libraries found. This is expected if you haven't built any dynamic plugins."
        );
    } else {
        for (i, path) in plugins.iter().enumerate() {
            println!("Found plugin {}: {}", i + 1, path.display());
        }

        // Load the plugins (commented out since we don't have any dynamic plugins in this example)
        // for path in plugins {
        //     match loader.load_plugin(path) {
        //         Ok(plugin) => {
        //             let plugin_id = plugin.id().to_string();
        //             registry.register_plugin(plugin).await?;
        //             println!("Loaded and registered plugin: {}", plugin_id);
        //         }
        //         Err(e) => {
        //             eprintln!("Failed to load plugin: {}", e);
        //         }
        //     }
        // }
    }

    // Resolve dependencies
    println!("\nResolving dependencies...");
    registry.resolve_dependencies()?;
    println!("Dependencies resolved successfully");

    // Initialize the plugins
    println!("\nInitializing plugins...");

    // Create configuration
    let mut config = PluginConfig::new();
    config.set("example_setting", "example_value");

    // Initialize first plugin
    registry
        .initialize_plugin("SimplePlugin1", config.clone())
        .await?;
    println!("Initialized plugin: SimplePlugin1");

    // Initialize second plugin
    registry.initialize_plugin("SimplePlugin2", config).await?;
    println!("Initialized plugin: SimplePlugin2");

    // Start the plugins
    println!("\nStarting plugins...");
    registry.start_all_plugins().await?;
    println!("All plugins started");

    // Check plugin health
    println!("\nChecking plugin health...");
    let health = registry.health_check_all().await;

    for (id, status) in health {
        println!("Plugin {} health: {}", id, status);
    }

    // Get plugin by ID
    println!("\nGetting plugin by ID...");
    if let Some(plugin) = registry.get_plugin("SimplePlugin1") {
        let plugin_guard = plugin.read().unwrap();
        println!(
            "Found plugin: {}, version: {}, stage: {:?}",
            plugin_guard.metadata().name,
            plugin_guard.metadata().version,
            plugin_guard.lifecycle_stage()
        );
    }

    // Find plugins by tag
    println!("\nFinding plugins by tag...");
    let tagged_plugins = registry.find_plugins_by_tag("example");
    println!("Found {} plugins with tag 'example':", tagged_plugins.len());

    for plugin_id in tagged_plugins {
        println!("  - {}", plugin_id);
    }

    // Stop all plugins
    println!("\nStopping plugins...");
    registry.stop_all_plugins().await?;
    println!("All plugins stopped");

    // Unregister a plugin
    println!("\nUnregistering plugin...");
    registry.unregister_plugin("SimplePlugin2").await?;
    println!("Unregistered plugin: SimplePlugin2");

    // Try to unregister the first plugin (should fail because it's a dependency)
    println!("\nTrying to unregister dependency plugin...");
    match registry.unregister_plugin("SimplePlugin1").await {
        Ok(_) => println!("Unexpected: Unregistered plugin: SimplePlugin1"),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\nPlugin loading example completed");

    Ok(())
}
