---
title: "Server Customization System Examples"
description: "Practical code examples for using the Server Customization System in Navius applications"
category: examples
tags:
  - examples
  - server-customization
  - features
  - optimization
  - code
related:
  - ../guides/features/server-customization-cli.md
  - ../reference/configuration/feature-config.md
  - ../feature-system.md
last_updated: March 26, 2024
version: 1.0
---

# Server Customization System Examples

This document provides practical examples of how to use the Server Customization System in Navius applications.

## Basic Feature Configuration

```rust
use navius::core::features::{FeatureRegistry, RuntimeFeatures};
use navius::app::AppBuilder;

fn main() {
    // Create a new feature registry with default features
    let mut registry = FeatureRegistry::new();
    
    // Enable specific features
    registry.enable("caching").unwrap();
    registry.enable("metrics").unwrap();
    registry.enable("security").unwrap();
    
    // Disable features you don't need
    registry.disable("tracing").unwrap();
    registry.disable("websocket").unwrap();
    
    // Resolve dependencies (this will enable any dependencies of the enabled features)
    registry.resolve_dependencies().unwrap();
    
    // Create runtime features from the registry
    let runtime_features = RuntimeFeatures::from_registry(&registry);
    
    // Build your application with the configured features
    let app = AppBuilder::new()
        .with_features(runtime_features)
        .build();
        
    // Start the server
    app.start().unwrap();
}
```

## Loading Features from Configuration

```rust
use navius::core::features::{FeatureRegistry, FeatureConfig};
use navius::core::config::ConfigLoader;

fn load_features_from_config() -> FeatureRegistry {
    // Load configuration
    let config = ConfigLoader::new()
        .with_file("config/app.yaml")
        .load()
        .unwrap();
    
    // Extract features configuration
    let features_config = config.get_section("features").unwrap();
    
    // Create feature registry from configuration
    let mut registry = FeatureRegistry::from_config(&features_config);
    
    // You can still make runtime adjustments
    if cfg!(debug_assertions) {
        // Enable development features in debug mode
        registry.enable("debug_tools").unwrap();
    }
    
    // Finalize by resolving dependencies
    registry.resolve_dependencies().unwrap();
    
    registry
}
```

## Feature-Conditional Code Execution

```rust
use navius::core::features::RuntimeFeatures;

// Using feature check in functions
fn initialize_metrics(features: &RuntimeFeatures) {
    if !features.is_enabled("metrics") {
        println!("Metrics disabled, skipping initialization");
        return;
    }
    
    println!("Initializing metrics subsystem...");
    
    // Check for advanced metrics
    if features.is_enabled("advanced_metrics") {
        println!("Initializing advanced metrics...");
        // Initialize advanced metrics collectors
    }
}

// Using the convenience macro
fn setup_services(app_state: &AppState) {
    // This code only runs if the "caching" feature is enabled
    when_feature_enabled!(app_state, "caching", {
        println!("Setting up cache service...");
        let cache_service = CacheService::new().await.unwrap();
        app_state.register_service("cache", cache_service);
    });
    
    // This code only runs if the "redis_caching" feature is enabled
    when_feature_enabled!(app_state, "redis_caching", {
        println!("Setting up Redis cache provider...");
        let redis_provider = RedisProvider::new("redis://localhost:6379").await.unwrap();
        app_state.register_cache_provider("redis", redis_provider);
    });
}
```

## Feature Dependency Example

```rust
use navius::core::features::{FeatureRegistry, FeatureInfo};

fn setup_feature_dependencies() -> FeatureRegistry {
    let mut registry = FeatureRegistry::new();
    
    // Define features with dependencies
    let metrics = FeatureInfo::new("metrics")
        .with_description("Basic metrics collection")
        .with_default_enabled(true);
    
    let advanced_metrics = FeatureInfo::new("advanced_metrics")
        .with_description("Advanced metrics and custom reporters")
        .with_dependency("metrics")  // Depends on basic metrics
        .with_default_enabled(false);
    
    let redis_caching = FeatureInfo::new("redis_caching")
        .with_description("Redis cache provider")
        .with_dependency("caching")  // Depends on basic caching
        .with_default_enabled(true);
    
    // Register features
    registry.register(metrics).unwrap();
    registry.register(advanced_metrics).unwrap();
    registry.register(redis_caching).unwrap();
    
    // When we enable advanced_metrics, it will automatically enable metrics
    registry.enable("advanced_metrics").unwrap();
    
    // Resolve all dependencies
    registry.resolve_dependencies().unwrap();
    
    // We didn't explicitly enable "metrics", but it will be enabled
    // as a dependency of "advanced_metrics"
    assert!(registry.is_enabled("metrics"));
    
    registry
}
```

## Using the Feature CLI

The Server Customization System includes a CLI tool for managing features. Here's how to use it:

```bash
# List all available features
features_cli list

# Enable a specific feature
features_cli enable caching

# Disable a feature
features_cli disable tracing

# Show current feature status
features_cli status

# Create a custom server build with selected features
features_cli build --output=my-custom-server.bin

# Save current feature configuration to a file
features_cli save my-features.json

# Load features from a configuration file
features_cli load my-features.json
```

## Conditional Compilation with Cargo Features

You can also use Cargo's feature flags for compile-time feature selection:

```toml
# Cargo.toml
[features]
default = ["metrics", "caching", "security"]
metrics = []
advanced_metrics = ["metrics"]
tracing = []
caching = []
redis_caching = ["caching"]
security = []
```

Then in your code:

```rust
// This code only compiles if the "metrics" feature is enabled
#[cfg(feature = "metrics")]
pub mod metrics {
    pub fn initialize() {
        println!("Initializing metrics...");
    }
    
    // This code only compiles if both "metrics" and "advanced_metrics" features are enabled
    #[cfg(feature = "advanced_metrics")]
    pub fn initialize_advanced() {
        println!("Initializing advanced metrics...");
    }
}

// This function only exists if the "caching" feature is enabled
#[cfg(feature = "caching")]
pub fn setup_cache() {
    println!("Setting up cache...");
    
    // This code only compiles if both "caching" and "redis_caching" features are enabled
    #[cfg(feature = "redis_caching")]
    {
        println!("Setting up Redis cache provider...");
    }
}
```

## Feature Configuration File Example

```yaml
# features.yaml
enabled:
  - core
  - api
  - rest
  - security
  - caching
  - redis_caching
  - metrics

disabled:
  - tracing
  - advanced_metrics
  - websocket
  - graphql

configuration:
  caching:
    memory_cache_enabled: true
    memory_cache_size: 10000
    redis_enabled: true
    redis_url: "redis://localhost:6379"
  
  security:
    rate_limit_enabled: true
    rate_limit_requests_per_minute: 100
```

## Custom Feature Registration

```rust
use navius::core::features::{FeatureRegistry, FeatureInfo, FeatureCategory};

fn register_custom_features() -> FeatureRegistry {
    let mut registry = FeatureRegistry::new();
    
    // Define a custom feature category
    let api_category = FeatureCategory::new("api")
        .with_description("API related features");
    
    // Register the category
    registry.register_category(api_category);
    
    // Create custom features
    let custom_api = FeatureInfo::new("custom_api")
        .with_description("Custom API endpoints")
        .with_category("api")
        .with_default_enabled(false);
    
    let custom_auth = FeatureInfo::new("custom_auth")
        .with_description("Custom authentication provider")
        .with_category("auth")
        .with_dependency("auth")
        .with_default_enabled(false);
    
    // Register custom features
    registry.register(custom_api).unwrap();
    registry.register(custom_auth).unwrap();
    
    // Enable custom features
    registry.enable("custom_api").unwrap();
    
    // Resolve dependencies
    registry.resolve_dependencies().unwrap();
    
    registry
}
```

## Feature Status Display

The feature system includes utilities for displaying feature status:

```rust
use navius::core::features::{FeatureRegistry, FeatureStatusPrinter};

fn display_feature_status(registry: &FeatureRegistry) {
    let printer = FeatureStatusPrinter::new(registry);
    
    // Print a summary of enabled/disabled features
    printer.print_summary();
    
    // Print detailed information about all features
    printer.print_detailed();
    
    // Print information about a specific feature
    printer.print_feature("caching");
    
    // Print dependency tree
    printer.print_dependency_tree();
}
```

## Feature Documentation Generation

```rust
use navius::core::features::{FeatureRegistry, FeatureDocGenerator};
use std::fs::File;

fn generate_feature_documentation(registry: &FeatureRegistry) {
    let doc_generator = FeatureDocGenerator::new(registry);
    
    // Generate documentation for all features
    let docs = doc_generator.generate_all();
    
    // Write to a markdown file
    let mut file = File::create("features.md").unwrap();
    doc_generator.write_markdown(&mut file, &docs).unwrap();
    
    // Generate configuration examples
    let examples = doc_generator.generate_configuration_examples();
    
    // Write to a YAML file
    let mut example_file = File::create("feature-examples.yaml").unwrap();
    doc_generator.write_yaml_examples(&mut example_file, &examples).unwrap();
}
```

## Feature Visualization

The Server Customization System includes tools for visualizing feature dependencies:

```rust
use navius::core::features::{FeatureRegistry, FeatureVisualizer};
use std::fs::File;

fn generate_feature_visualization(registry: &FeatureRegistry) {
    let visualizer = FeatureVisualizer::new(registry);
    
    // Generate a DOT graph of feature dependencies
    let dot_graph = visualizer.generate_dot_graph();
    
    // Write to a DOT file
    let mut dot_file = File::create("features.dot").unwrap();
    dot_file.write_all(dot_graph.as_bytes()).unwrap();
    
    // Generate a dependency tree in ASCII
    let ascii_tree = visualizer.generate_ascii_tree();
    println!("{}", ascii_tree);
    
    // Generate an HTML visualization
    let html = visualizer.generate_html();
    let mut html_file = File::create("features.html").unwrap();
    html_file.write_all(html.as_bytes()).unwrap();
}
``` 