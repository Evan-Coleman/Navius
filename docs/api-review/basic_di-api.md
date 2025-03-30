# basic_di API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- navius-core
- async-trait
- thiserror
- tracing
- tokio
- serde
- serde_json
- log

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| ComponentRef<T>(Arc<T>); | registry.rs | 127 | ⚠️ Partial |
| DynComponentRef(Arc<dyn | registry.rs | 154 | ⚠️ Partial |
| TypedComponentFactory<T, | registry.rs | 220 | ⚠️ Partial |
| ComponentRegistry | registry.rs | 259 | ⚠️ Partial |
| Binder<'a> | config.rs | 27 | ✅ Complete |
| ConfigRegistry | config.rs | 81 | ⚠️ Partial |
| ConfigRef<T: | config.rs | 142 | ⚠️ Partial |
| MemoryConfigProvider | application.rs | 30 | ✅ Complete |
| ApplicationBuilder | application.rs | 97 | ⚠️ Partial |
| Application | application.rs | 245 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| LifecyclePhase | registry.rs | 21 | ⚠️ Partial |
| ComponentScope | registry.rs | 97 | ⚠️ Partial |
| Error | error.rs | 6 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Lifecycle: | registry.rs | 41 | ⚠️ Partial |
| AsyncLifecycle: | registry.rs | 69 | ⚠️ Partial |
| ComponentFactory: | registry.rs | 187 | ⚠️ Partial |
| AsyncComponentFactory: | registry.rs | 207 | ⚠️ Partial |
| IntoError<T> | error.rs | 151 | ⚠️ Partial |
| ConfigBinding | config.rs | 21 | ⚠️ Partial |
| ConfigPrefix | config.rs | 169 | ⚠️ Partial |
| Configurable: | config.rs | 175 | ✅ Complete |
| Component: | macros.rs | 161 | ⚠️ Partial |
| ConfigProvider: | application.rs | 18 | ⚠️ Partial |
| ApplicationPlugin: | application.rs | 79 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(component: | registry.rs | 131 | ⚠️ Partial |
| into_arc(self) | registry.rs | 136 | ⚠️ Partial |
| into_raw(self) | registry.rs | 141 | ⚠️ Partial |
| new<T: | registry.rs | 158 | ⚠️ Partial |
| downcast<T: | registry.rs | 163 | ⚠️ Partial |
| execute_lifecycle(&self, | registry.rs | 170 | ⚠️ Partial |
| fn | registry.rs | 178 | ⚠️ Partial |
| new(factory: | registry.rs | 228 | ⚠️ Partial |
| new() | registry.rs | 267 | ⚠️ Partial |
| register<T: | registry.rs | 276 | ⚠️ Partial |
| register_with_qualifier<T: | registry.rs | 292 | ⚠️ Partial |
| register_factory<F>(&self, | registry.rs | 316 | ⚠️ Partial |
| register_with_factory<T, | registry.rs | 322 | ⚠️ Partial |
| register_with_factory_and_qualifier<T, | registry.rs | 332 | ⚠️ Partial |
| get<T: | registry.rs | 351 | ⚠️ Partial |
| get_by_qualifier<T: | registry.rs | 387 | ⚠️ Partial |
| fn | registry.rs | 434 | ⚠️ Partial |
| fn | registry.rs | 472 | ⚠️ Partial |
| has<T: | registry.rs | 521 | ⚠️ Partial |
| has_qualifier(&self, | registry.rs | 528 | ⚠️ Partial |
| component_types(&self) | registry.rs | 533 | ⚠️ Partial |
| qualifiers(&self) | registry.rs | 553 | ⚠️ Partial |
| remove<T: | registry.rs | 558 | ⚠️ Partial |
| remove_by_qualifier(&self, | registry.rs | 586 | ⚠️ Partial |
| shutdown(&self) | registry.rs | 615 | ⚠️ Partial |
| fn | registry.rs | 635 | ⚠️ Partial |
| new(message: | error.rs | 86 | ⚠️ Partial |
| component_not_found<T: | error.rs | 91 | ⚠️ Partial |
| qualifier_not_found(qualifier: | error.rs | 98 | ⚠️ Partial |
| initialization_failed<E: | error.rs | 105 | ⚠️ Partial |
| shutdown_failed<E: | error.rs | 112 | ⚠️ Partial |
| config_not_found(key: | error.rs | 119 | ⚠️ Partial |
| config_binding_failed<E: | error.rs | 126 | ⚠️ Partial |
| plugin_initialization_failed<E: | error.rs | 133 | ⚠️ Partial |
| missing_dependency(name: | error.rs | 140 | ⚠️ Partial |
| new(provider: | config.rs | 33 | ⚠️ Partial |
| new(provider: | config.rs | 88 | ⚠️ Partial |
| get_or_bind<T: | config.rs | 96 | ⚠️ Partial |
| get_all_prefixes(&self) | config.rs | 123 | ⚠️ Partial |
| new(config: | config.rs | 148 | ⚠️ Partial |
| into_inner(self) | config.rs | 155 | ⚠️ Partial |
| config_ref<T: | config.rs | 191 | ⚠️ Partial |
| new() | application.rs | 36 | ⚠️ Partial |
| set<T: | application.rs | 43 | ⚠️ Partial |
| new() | application.rs | 105 | ⚠️ Partial |
| with_config_provider(mut | application.rs | 114 | ⚠️ Partial |
| with_component<T: | application.rs | 120 | ⚠️ Partial |
| with_component_and_qualifier<T: | application.rs | 128 | ⚠️ Partial |
| with_factory<T, | application.rs | 146 | ⚠️ Partial |
| with_factory_and_qualifier<T, | application.rs | 156 | ⚠️ Partial |
| with_plugin<P: | application.rs | 172 | ⚠️ Partial |
| with_config<T: | application.rs | 178 | ⚠️ Partial |
| with_config_as_component(self) | application.rs | 192 | ⚠️ Partial |
| fn | application.rs | 207 | ⚠️ Partial |
| builder() | application.rs | 252 | ⚠️ Partial |
| get<T: | application.rs | 257 | ⚠️ Partial |
| get_by_qualifier<T: | application.rs | 262 | ⚠️ Partial |
| config<T: | application.rs | 270 | ⚠️ Partial |
| config_keys_with_prefix(&self, | application.rs | 275 | ⚠️ Partial |
| shutdown(&self) | application.rs | 280 | ⚠️ Partial |
| fn | application.rs | 285 | ⚠️ Partial |

