---
title: "Server Customization CLI Guide"
description: "Detailed instructions for using the Server Customization System's CLI tool to manage features and create optimized server builds"
category: guides
tags:
  - features
  - server-customization
  - cli
  - performance
  - optimization
  - deployment
related:
  - ../../reference/configuration/feature-config.md
  - ../../examples/server-customization-example.md
  - ../../feature-system.md
last_updated: March 27, 2025
version: 1.0
---

# Server Customization CLI Guide

This guide provides detailed instructions for using the Server Customization System's CLI tool to manage features and create customized server builds.

## Overview

The Server Customization CLI allows you to:

- List available features and their status
- Enable or disable specific features
- Create custom server builds with selected features
- Save and load feature configurations
- Visualize feature dependencies
- Analyze and optimize dependencies

## Installation

The CLI tool is included with the Navius repository. To build it:

```bash
# Using cargo
cargo build --bin features_cli

# The binary will be available at
./target/debug/features_cli
```

## Command Reference

### Listing Features

To view all available features:

```bash
features_cli list
```

Example output:

```
Available Features:
✅ core                 Core server functionality (required)
✅ config               Configuration system (required)
✅ metrics              Metrics collection and reporting
❌ advanced_metrics     Advanced metrics and custom reporters
✅ caching              Caching system
✅ redis_caching        Redis cache provider
❌ tracing              Distributed tracing
✅ security             Security features
...
```

To get more detailed information:

```bash
features_cli list --verbose
```

### Getting Feature Status

To check the current status of features:

```bash
features_cli status
```

Example output:

```
Feature Status:
Enabled features: 12
Disabled features: 8
Required dependencies: 3

Enabled:
- core (required)
- config (required)
- error_handling (required)
- metrics
- caching
- redis_caching
- security
...

Disabled:
- advanced_metrics
- tracing
- websocket
...
```

To check a specific feature:

```bash
features_cli status metrics
```

### Enabling Features

To enable a specific feature:

```bash
features_cli enable metrics
```

This will also automatically enable any dependencies.

To enable multiple features:

```bash
features_cli enable metrics caching security
```

### Disabling Features

To disable a feature:

```bash
features_cli disable tracing
```

Note that you cannot disable features that others depend on without first disabling the dependent features.

```bash
# This will fail if advanced_metrics depends on metrics
features_cli disable metrics

# You need to disable advanced_metrics first
features_cli disable advanced_metrics
features_cli disable metrics
```

### Creating Custom Builds

To create a custom server build with only the features you need:

```bash
features_cli build --output=my-custom-server
```

To specify features for the build:

```bash
features_cli build --features=core,metrics,caching,security --output=minimal-server
```

### Saving and Loading Configurations

To save your current feature configuration:

```bash
features_cli save my-config.json
```

To load a saved configuration:

```bash
features_cli load my-config.json
```

You can also save in different formats:

```bash
features_cli save --format=yaml my-config.yaml
```

### Visualizing Dependencies

To visualize feature dependencies:

```bash
features_cli visualize
```

This will print an ASCII representation of the dependency tree.

For graphical output:

```bash
features_cli visualize --format=dot --output=dependencies.dot
```

You can then use tools like Graphviz to render the DOT file:

```bash
dot -Tpng dependencies.dot -o dependencies.png
```

### Analyzing and Optimizing

To analyze your feature selection:

```bash
features_cli analyze
```

This will show information about:
- Size impact of each feature
- Dependencies between features
- Potentially unused features
- Optimization suggestions

To analyze a specific feature:

```bash
features_cli analyze metrics
```

## Interactive Mode

The CLI tool also provides an interactive mode:

```bash
features_cli interactive
```

In interactive mode, you can:
- Navigate through features using arrow keys
- Toggle features on/off with the space bar
- View details about each feature
- See immediate feedback on dependencies
- Save your configuration when done

## Configuration Files

You can also define features using configuration files:

```yaml
# features.yaml
enabled:
  - core
  - metrics
  - caching
  - security
disabled:
  - tracing
  - advanced_metrics
configuration:
  metrics:
    prometheus_enabled: true
    collect_interval_seconds: 15
```

Load this configuration using:

```bash
features_cli load features.yaml
```

## Environment Variables

You can configure the CLI behavior using environment variables:

| Variable | Description |
|----------|-------------|
| `NAVIUS_FEATURES_FILE` | Default features file to load |
| `NAVIUS_FEATURES_FORMAT` | Default format for saving (json, yaml) |
| `NAVIUS_FEATURES_CONFIG` | Path to the main configuration file |
| `NAVIUS_FEATURES_OUTPUT_DIR` | Default output directory for builds |

## Best Practices

1. **Start Minimal**: Begin with only essential features enabled
2. **Use Configuration Files**: Save your feature configuration for consistency across environments
3. **Analyze First**: Use the analyze command to optimize your feature set before building
4. **Check Dependencies**: Be aware of feature dependencies when making changes
5. **Version Control**: Store your feature configurations in version control

## Troubleshooting

### Common Issues

#### "Cannot disable required feature"

Core features cannot be disabled. These include:
- core
- config
- error_handling

#### "Cannot disable dependency"

You're trying to disable a feature that others depend on. Disable the dependent features first.

#### "Feature not found"

Check the feature name with the `list` command. Feature names are case-sensitive.

#### "Configuration file format not supported"

The CLI supports JSON and YAML formats. Check your file extension.

## Examples

### Minimal API Server

```bash
features_cli enable core api rest security
features_cli disable metrics tracing caching websocket
features_cli build --output=minimal-api-server
```

### Analytics Server

```bash
features_cli enable core metrics advanced_metrics tracing structured_logging
features_cli disable api websocket
features_cli build --output=analytics-server
```

### Production Ready Server

```bash
features_cli load production-features.yaml
features_cli enable security rate_limiting
features_cli build --output=production-server
``` 