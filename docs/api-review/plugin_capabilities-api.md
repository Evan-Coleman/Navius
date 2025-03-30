# plugin_capabilities API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 99%  
**Status:** ✅ Good

## Dependencies

- async-trait
- chrono
- libloading
- serde
- serde_json
- semver
- thiserror
- tokio
- tracing
- navius-core

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PluginRegistry | registry.rs | 12 | ⚠️ Partial |
| BasePlugin | base.rs | 15 | ⚠️ Partial |
| PluginBuilder | base.rs | 204 | ⚠️ Partial |
| PluginLoader | loader.rs | 16 | ✅ Complete |
| SharedPluginLoader | loader.rs | 170 | ⚠️ Partial |
| PluginMetadata | plugin.rs | 12 | ⚠️ Partial |
| PluginConfig | plugin.rs | 74 | ⚠️ Partial |
| PluginDependency | plugin.rs | 179 | ⚠️ Partial |
| RouteInfo | capability.rs | 167 | ⚠️ Partial |
| RouteResponse | capability.rs | 180 | ✅ Complete |
| HttpResponse | capability.rs | 330 | ⚠️ Partial |
| HttpRequest | capability.rs | 361 | ✅ Complete |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PluginError | error.rs | 10 | ⚠️ Partial |
| PluginHealth | error.rs | 166 | ⚠️ Partial |
| PluginLifecycleStage | plugin.rs | 144 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| PluginRegistryManager: | registry.rs | 380 | ⚠️ Partial |
| Plugin: | plugin.rs | 212 | ⚠️ Partial |
| Capability: | capability.rs | 11 | ⚠️ Partial |
| ConfigurationCapability: | capability.rs | 52 | ⚠️ Partial |
| HealthCheckCapability: | capability.rs | 65 | ✅ Complete |
| LoggingCapability: | capability.rs | 74 | ✅ Complete |
| StorageCapability: | capability.rs | 93 | ✅ Complete |
| EventCapability: | capability.rs | 109 | ✅ Complete |
| HttpCapability: | capability.rs | 126 | ⚠️ Partial |
| RoutingCapability: | capability.rs | 141 | ✅ Complete |
| RouteHandler: | capability.rs | 154 | ✅ Complete |
| LoggingCapability: | capability.rs | 205 | ⚠️ Partial |
| HealthCheckCapability: | capability.rs | 224 | ✅ Complete |
| ConfigurationCapability: | capability.rs | 234 | ✅ Complete |
| StorageCapability: | capability.rs | 256 | ✅ Complete |
| EventCapability: | capability.rs | 278 | ✅ Complete |
| HttpCapability: | capability.rs | 296 | ⚠️ Partial |
| RoutingCapability: | capability.rs | 343 | ✅ Complete |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new() | registry.rs | 25 | ✅ Complete |
| fn | registry.rs | 34 | ⚠️ Partial |
| get_plugin(&self, | registry.rs | 53 | ⚠️ Partial |
| get_all_plugins(&self) | registry.rs | 58 | ⚠️ Partial |
| fn | registry.rs | 68 | ⚠️ Partial |
| fn | registry.rs | 108 | ⚠️ Partial |
| fn | registry.rs | 135 | ⚠️ Partial |
| fn | registry.rs | 158 | ⚠️ Partial |
| fn | registry.rs | 181 | ⚠️ Partial |
| fn | registry.rs | 193 | ⚠️ Partial |
| resolve_dependencies(&self) | registry.rs | 205 | ⚠️ Partial |
| fn | registry.rs | 300 | ⚠️ Partial |
| fn | registry.rs | 316 | ⚠️ Partial |
| get_plugin_capability( | registry.rs | 331 | ⚠️ Partial |
| find_plugins_by_tag(&self, | registry.rs | 355 | ⚠️ Partial |
| new(metadata: | base.rs | 37 | ✅ Complete |
| with_dependency(mut | base.rs | 49 | ⚠️ Partial |
| with_optional_dependency( | base.rs | 55 | ⚠️ Partial |
| with_capability<T: | base.rs | 66 | ⚠️ Partial |
| with_typed_capability<T: | base.rs | 78 | ⚠️ Partial |
| set_health(&self, | base.rs | 84 | ⚠️ Partial |
| new( | base.rs | 217 | ✅ Complete |
| with_tag(mut | base.rs | 232 | ⚠️ Partial |
| with_tags(mut | base.rs | 238 | ⚠️ Partial |
| with_dependency(mut | base.rs | 244 | ⚠️ Partial |
| with_optional_dependency( | base.rs | 250 | ⚠️ Partial |
| with_capability<T: | base.rs | 261 | ⚠️ Partial |
| with_named_capability<T: | base.rs | 270 | ⚠️ Partial |
| build(self) | base.rs | 280 | ⚠️ Partial |
| new() | loader.rs | 27 | ⚠️ Partial |
| add_search_path(&mut | loader.rs | 35 | ⚠️ Partial |
| find_plugins(&self) | loader.rs | 41 | ⚠️ Partial |
| load_plugin(&mut | loader.rs | 90 | ⚠️ Partial |
| scan_and_load(&mut | loader.rs | 132 | ⚠️ Partial |
| new() | loader.rs | 177 | ✅ Complete |
| add_search_path(&self, | loader.rs | 184 | ⚠️ Partial |
| find_plugins(&self) | loader.rs | 200 | ⚠️ Partial |
| load_plugin(&self, | loader.rs | 215 | ⚠️ Partial |
| scan_and_load(&self) | loader.rs | 230 | ⚠️ Partial |
| extern | loader.rs | 256 | ❌ Missing |
| new( | plugin.rs | 37 | ✅ Complete |
| with_tag(mut | plugin.rs | 58 | ⚠️ Partial |
| with_tags(mut | plugin.rs | 64 | ⚠️ Partial |
| new() | plugin.rs | 89 | ⚠️ Partial |
| from_map(settings: | plugin.rs | 94 | ⚠️ Partial |
| get_string(&self, | plugin.rs | 99 | ⚠️ Partial |
| get_number(&self, | plugin.rs | 106 | ⚠️ Partial |
| get_bool(&self, | plugin.rs | 111 | ⚠️ Partial |
| get_value(&self, | plugin.rs | 116 | ⚠️ Partial |
| set<T>(&mut | plugin.rs | 121 | ⚠️ Partial |
| contains_key(&self, | plugin.rs | 132 | ⚠️ Partial |
| keys(&self) | plugin.rs | 137 | ⚠️ Partial |
| new(id: | plugin.rs | 192 | ✅ Complete |
| optional(id: | plugin.rs | 201 | ⚠️ Partial |
| downcast_capability<T: | capability.rs | 192 | ✅ Complete |
| downcast_capability_mut<T: | capability.rs | 197 | ⚠️ Partial |

