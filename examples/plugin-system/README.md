# Plugin System Integration Example

This example demonstrates the integration of plugin system components in the Navius framework. It showcases how to design and implement a plugin-based application architecture with a focus on extensibility, dependency management, and lifecycle hooks.

## Key Concepts Demonstrated

1. **Plugin Discovery and Loading**:
   - Automatic discovery of plugins in the application
   - Dynamic loading of plugins based on dependencies
   - Support for manual and automatic plugin registration

2. **Plugin Lifecycle Management**:
   - Plugin initialization and startup
   - Graceful plugin shutdown
   - Resource management with proper cleanup

3. **Component Registry Integration**:
   - Registration of plugin-provided components
   - Dependency resolution between plugins
   - Type-safe component retrieval

4. **Capability-Based Design**:
   - Plugins exposing capabilities through traits
   - Standardized capability interfaces
   - Capability composition for complex functionality

5. **Cross-Plugin Communication**:
   - Service-based communication between plugins
   - Event-based notification system
   - Dependency management for plugin interactions

6. **Dynamic Plugin Loading**:
   - Loading plugins from compiled shared libraries
   - Runtime discovery of plugin capabilities
   - Integration with existing plugin registry

## Plugin Types Implemented

### Core Infrastructure Plugins

- **LoggingPlugin**: Provides logging capabilities to the application
- **ConfigPlugin**: Provides configuration management
- **StoragePlugin**: Provides data storage functionality

### Application Functionality Plugins

- **ApiPlugin**: Provides HTTP API functionality
- **UserPlugin**: Provides user management capabilities
- **AnalyticsPlugin**: Provides analytics functionalities

### Dynamic Plugins

- **DynamicPlugin**: A plugin loaded at runtime from a dynamic library

## Configuration and Extensibility

The example demonstrates multiple configuration sources:

- **Environment Variables**: Configuration from environment variables
- **Configuration Files**: Loading from configuration files
- **Memory Configuration**: Default and runtime configuration values
- **Composite Configuration**: Combining multiple configuration sources

## Running the Example

First, build the dynamic plugins:

```bash
# Build the dynamic plugins
./build_plugins.sh
```

Then run the example:

```bash
# From the workspace_migration/examples/integration/plugin-system directory
cargo run
```

This will start the example application with all plugins loaded. The application will:

1. Initialize all built-in plugins
2. Discover and load dynamic plugins
3. Start the HTTP API
4. Run until terminated with Ctrl+C
5. Gracefully shut down all plugins in reverse dependency order

## Testing

The example includes tests for plugin lifecycle and dependency resolution:

```bash
# Run the tests
cargo test
```

## Project Structure

- `src/lib.rs` - Core library with plugin system integration
- `src/main.rs` - Application entry point
- `src/plugins.rs` - Plugin implementations
- `src/capabilities.rs` - Capability trait definitions
- `src/services.rs` - Service implementations for capabilities
- `src/api.rs` - HTTP API implementation
- `src/config.rs` - Configuration management utilities
- `plugins/` - Dynamic plugin implementations
- `tests/` - Integration tests for plugin lifecycle

## Key Design Patterns

1. **Plugin Pattern**: Encapsulating functionality in standalone, dynamically loadable units
2. **Capability Pattern**: Standardized interfaces for plugin functionality
3. **Dependency Injection**: Component-based architecture with dependency inversion
4. **Factory Pattern**: Creating components with controlled lifecycle
5. **Observer Pattern**: Event-based communication between plugins
6. **Composite Pattern**: Combined configuration sources with unified interface

## Best Practices Demonstrated

- Clean separation of concerns between plugins
- Proper resource management with initialization/cleanup
- Type-safe component registry with clear ownership
- Configuration management across multiple sources
- Robust error handling and logging
- Comprehensive testing of plugin lifecycle
- Clear dependency management between plugins

## Dynamic Plugin Implementation

The dynamic plugin implementation showcases:

1. **Library Structure**: How to structure a dynamically loadable plugin
2. **Export Functions**: Required functions for loading by the plugin system
3. **Capability Registration**: How to expose capabilities to the main application
4. **Dependency Management**: How to depend on capabilities from other plugins
5. **Lifecycle Hooks**: Proper initialization, startup, and shutdown procedures

## Implementation Notes

This example is designed to serve as a reference implementation for building plugin-based applications with Navius. It demonstrates best practices for plugin development, lifecycle management, and inter-plugin communication. The design is focused on extensibility, allowing new plugins to be added with minimal changes to existing code.

The capability-based design ensures that plugins can provide functionality through well-defined interfaces, promoting loose coupling and enhancing testability. The dependency management system ensures that plugins are initialized and started in the correct order, with proper handling of circular dependencies. 