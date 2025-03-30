use async_trait::async_trait;

use crate::component::{Component, ComponentRegistry, Scope};
use crate::error::PluginResult;
use crate::plugin::{Plugin, PluginBuilder, PluginRegistry};

// A simple service for testing
#[derive(Clone, Debug)]
struct GreetingService {
    greeting: String,
}

impl GreetingService {
    fn new(greeting: &str) -> Self {
        Self {
            greeting: greeting.to_string(),
        }
    }

    fn greet(&self, name: &str) -> String {
        format!("{} {}", self.greeting, name)
    }
}

// A test plugin implementation
#[derive(Clone)]
struct TestPlugin {
    id: String,
    description: String,
    dependencies: Vec<String>,
    initialized: bool,
    shutdown: bool,
}

impl TestPlugin {
    fn new(id: &str, description: &str, dependencies: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            dependencies,
            initialized: false,
            shutdown: false,
        }
    }
}

#[async_trait]
impl Plugin for TestPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }

    async fn initialize(&self, registry: &ComponentRegistry) -> PluginResult<()> {
        // Register a component with the registry
        registry.register_instance(
            Some(format!("{}-greeting", self.id)),
            GreetingService::new(&format!("Hello from {}", self.id)),
        )?;

        Ok(())
    }

    async fn shutdown(&self, _registry: &ComponentRegistry) -> PluginResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    #[test]
    fn test_component_registry() -> PluginResult<()> {
        // Create a component registry
        let registry = ComponentRegistry::new();

        // Register a singleton component
        let service = GreetingService::new("Hello");
        registry.register_instance(None, service.clone())?;

        // Get the component
        let component: Component<GreetingService> = registry.get_by_type(None)?;

        // Check that it's the same instance
        assert_eq!(component.instance().greeting, "Hello");

        // Register a component with a name
        let french_service = GreetingService::new("Bonjour");
        registry.register_instance(Some("french".to_string()), french_service)?;

        // Get the component by name
        let french_component: Component<GreetingService> = registry.get_by_type(Some("french"))?;
        assert_eq!(french_component.instance().greeting, "Bonjour");

        // Test prototype scope
        let service = GreetingService::new("Hola");
        registry.register(Scope::Prototype, Some("spanish".to_string()), move || {
            service.clone()
        })?;

        // Get multiple instances
        let component1: Component<GreetingService> = registry.get_by_type(Some("spanish"))?;
        let component2: Component<GreetingService> = registry.get_by_type(Some("spanish"))?;

        // Check that they're equal (since we're cloning the same instance in the factory)
        assert_eq!(component1.instance().greeting, "Hola");
        assert_eq!(component2.instance().greeting, "Hola");

        Ok(())
    }

    #[test]
    fn test_plugin_registry() -> PluginResult<()> {
        let rt = Runtime::new().unwrap();

        // Create a plugin registry
        let registry = PluginRegistry::new();

        // Create plugins with dependencies
        let plugin1 = TestPlugin::new("plugin1", "First plugin", vec![]);
        let plugin2 = TestPlugin::new("plugin2", "Second plugin", vec!["plugin1".to_string()]);
        let plugin3 = TestPlugin::new(
            "plugin3",
            "Third plugin",
            vec!["plugin1".to_string(), "plugin2".to_string()],
        );

        // Register the plugins
        registry.register_plugin(plugin1)?;
        registry.register_plugin(plugin2)?;
        registry.register_plugin(plugin3)?;

        // Initialize the plugins
        rt.block_on(registry.initialize_all())?;

        // Check that all plugins are initialized
        assert!(registry.is_initialized("plugin1"));
        assert!(registry.is_initialized("plugin2"));
        assert!(registry.is_initialized("plugin3"));

        // Get a component registered by a plugin
        let component: Component<GreetingService> = registry
            .component_registry()
            .get_by_type(Some("plugin1-greeting"))?;

        assert_eq!(component.instance().greeting, "Hello from plugin1");

        // Shut down the plugins
        rt.block_on(registry.shutdown_all())?;

        // Check that all plugins are no longer initialized
        assert!(!registry.is_initialized("plugin1"));
        assert!(!registry.is_initialized("plugin2"));
        assert!(!registry.is_initialized("plugin3"));

        Ok(())
    }

    #[test]
    fn test_simple_plugin() -> PluginResult<()> {
        let rt = Runtime::new().unwrap();

        // Create a plugin registry
        let registry = PluginRegistry::new();

        // Create a simple plugin
        let plugin = PluginBuilder::new("simple")
            .description("A simple plugin")
            .dependency("core")
            .build();

        // Register the plugin
        registry.register_plugin(plugin)?;

        // Try to initialize (will fail due to missing dependency)
        let result = rt.block_on(registry.initialize_plugin("simple"));
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_circular_dependency_detection() -> PluginResult<()> {
        let rt = Runtime::new().unwrap();

        // Create a plugin registry
        let registry = PluginRegistry::new();

        // Create plugins with circular dependencies
        let plugin1 = TestPlugin::new("plugin1", "First plugin", vec!["plugin3".to_string()]);
        let plugin2 = TestPlugin::new("plugin2", "Second plugin", vec!["plugin1".to_string()]);
        let plugin3 = TestPlugin::new("plugin3", "Third plugin", vec!["plugin2".to_string()]);

        // Register the plugins
        registry.register_plugin(plugin1)?;
        registry.register_plugin(plugin2)?;
        registry.register_plugin(plugin3)?;

        // Initialize the plugins (should fail due to circular dependency)
        let result = rt.block_on(registry.initialize_all());
        assert!(result.is_err());

        Ok(())
    }
}
