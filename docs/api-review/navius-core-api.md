# navius-core API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- serde
- serde_json
- serde_yaml
- tokio
- tracing
- uuid
- async-trait
- thiserror
- regex
- config
- dotenvy
- chrono

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Timestamp(pub | types.rs | 21 | ✅ Complete |
| Version | types.rs | 74 | ⚠️ Partial |
| ComponentRef<T>(Arc<T>); | component.rs | 80 | ⚠️ Partial |
| DynComponentRef(Arc<dyn | component.rs | 110 | ⚠️ Partial |
| TypedComponentFactory<T: | component.rs | 199 | ⚠️ Partial |
| ComponentRegistry | component.rs | 233 | ⚠️ Partial |
| ApplicationBuilder | application.rs | 56 | ⚠️ Partial |
| Application | application.rs | 175 | ⚠️ Partial |
| ConfigValues | config_value.rs | 269 | ⚠️ Partial |
| Error | error.rs | 96 | ⚠️ Partial |
| Config | config.rs | 15 | ⚠️ Partial |
| PluginRegistry | lib.rs | 81 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| HttpMethod | types.rs | 112 | ⚠️ Partial |
| LifecyclePhase | component.rs | 19 | ⚠️ Partial |
| ComponentScope | component.rs | 65 | ⚠️ Partial |
| Environment | application.rs | 15 | ⚠️ Partial |
| ConfigError | errors.rs | 25 | ⚠️ Partial |
| ConfigValue | config_value.rs | 29 | ⚠️ Partial |
| ConfigSource | sources.rs | 24 | ⚠️ Partial |
| FileFormat | sources.rs | 69 | ✅ Complete |
| ErrorCode | error.rs | 19 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| Lifecycle: | component.rs | 27 | ✅ Complete |
| AsyncLifecycle: | component.rs | 46 | ⚠️ Partial |
| ComponentFactory: | component.rs | 160 | ⚠️ Partial |
| AsyncComponentFactory: | component.rs | 180 | ⚠️ Partial |
| ResultExt<T, | error.rs | 306 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| now() | types.rs | 25 | ✅ Complete |
| new(time: | types.rs | 30 | ⚠️ Partial |
| inner(&self) | types.rs | 35 | ⚠️ Partial |
| elapsed(&self) | types.rs | 40 | ⚠️ Partial |
| new(major: | types.rs | 85 | ✅ Complete |
| init() | mod.rs | 16 | ⚠️ Partial |
| init_application() | mod.rs | 21 | ⚠️ Partial |
| init_application_with_config( | mod.rs | 26 | ⚠️ Partial |
| new(component: | component.rs | 84 | ⚠️ Partial |
| into_raw(self) | component.rs | 89 | ⚠️ Partial |
| new<T: | component.rs | 114 | ⚠️ Partial |
| downcast<T: | component.rs | 119 | ⚠️ Partial |
| execute_type_id(&self) | component.rs | 124 | ⚠️ Partial |
| execute_lifecycle(&self, | component.rs | 130 | ⚠️ Partial |
| fn | component.rs | 142 | ⚠️ Partial |
| new(factory: | component.rs | 206 | ⚠️ Partial |
| new() | component.rs | 240 | ⚠️ Partial |
| register<T: | component.rs | 248 | ⚠️ Partial |
| register_factory<F>(&mut | component.rs | 260 | ⚠️ Partial |
| register_with_factory<T, | component.rs | 266 | ⚠️ Partial |
| get<T: | component.rs | 276 | ⚠️ Partial |
| try_get<T: | component.rs | 291 | ⚠️ Partial |
| fn | component.rs | 303 | ⚠️ Partial |
| has<T: | component.rs | 318 | ⚠️ Partial |
| component_types(&self) | component.rs | 324 | ⚠️ Partial |
| shutdown(&mut | component.rs | 341 | ⚠️ Partial |
| fn | component.rs | 358 | ⚠️ Partial |
| name(&self) | application.rs | 28 | ✅ Complete |
| from_name(name: | application.rs | 38 | ⚠️ Partial |
| new() | application.rs | 67 | ✅ Complete |
| with_config(config: | application.rs | 76 | ⚠️ Partial |
| with_environment(mut | application.rs | 85 | ⚠️ Partial |
| registry(&mut | application.rs | 91 | ⚠️ Partial |
| config(&self) | application.rs | 96 | ⚠️ Partial |
| environment(&self) | application.rs | 101 | ⚠️ Partial |
| add_component<T: | application.rs | 106 | ⚠️ Partial |
| add_factory<T, | application.rs | 112 | ⚠️ Partial |
| add_singleton<T, | application.rs | 122 | ⚠️ Partial |
| add_prototype<T, | application.rs | 133 | ⚠️ Partial |
| get<T: | application.rs | 144 | ⚠️ Partial |
| fn | application.rs | 149 | ⚠️ Partial |
| has<T: | application.rs | 154 | ⚠️ Partial |
| build(self) | application.rs | 159 | ⚠️ Partial |
| builder() | application.rs | 186 | ✅ Complete |
| registry(&self) | application.rs | 191 | ⚠️ Partial |
| config(&self) | application.rs | 196 | ⚠️ Partial |
| environment(&self) | application.rs | 201 | ⚠️ Partial |
| get<T: | application.rs | 206 | ⚠️ Partial |
| fn | application.rs | 217 | ⚠️ Partial |
| has<T: | application.rs | 228 | ⚠️ Partial |
| register_component<T: | application.rs | 237 | ⚠️ Partial |
| initialize(&self) | application.rs | 250 | ⚠️ Partial |
| shutdown(&self) | application.rs | 267 | ⚠️ Partial |
| fn | application.rs | 284 | ⚠️ Partial |
| random_id(prefix: | util.rs | 12 | ⚠️ Partial |
| current_time_millis() | util.rs | 23 | ⚠️ Partial |
| measure_time<F, | util.rs | 31 | ⚠️ Partial |
| parse_duration(duration_str: | util.rs | 43 | ⚠️ Partial |
| encode_query(params: | util.rs | 85 | ⚠️ Partial |
| parse_query(query: | util.rs | 101 | ⚠️ Partial |
| merge_maps<K, | util.rs | 121 | ⚠️ Partial |
| key_not_found(key: | errors.rs | 214 | ⚠️ Partial |
| type_error(expected: | errors.rs | 221 | ⚠️ Partial |
| load_error(source: | errors.rs | 229 | ⚠️ Partial |
| parse_error(source: | errors.rs | 237 | ⚠️ Partial |
| source_error(source: | errors.rs | 245 | ⚠️ Partial |
| env_error(var: | errors.rs | 253 | ⚠️ Partial |
| is_string(&self) | config_value.rs | 48 | ✅ Complete |
| is_integer(&self) | config_value.rs | 53 | ⚠️ Partial |
| is_float(&self) | config_value.rs | 58 | ⚠️ Partial |
| is_boolean(&self) | config_value.rs | 63 | ⚠️ Partial |
| is_array(&self) | config_value.rs | 68 | ⚠️ Partial |
| is_object(&self) | config_value.rs | 73 | ⚠️ Partial |
| is_null(&self) | config_value.rs | 78 | ⚠️ Partial |
| as_string(&self) | config_value.rs | 83 | ⚠️ Partial |
| as_integer(&self) | config_value.rs | 94 | ⚠️ Partial |
| as_float(&self) | config_value.rs | 119 | ⚠️ Partial |
| as_boolean(&self) | config_value.rs | 135 | ⚠️ Partial |
| as_array(&self) | config_value.rs | 169 | ⚠️ Partial |
| as_object(&self) | config_value.rs | 180 | ⚠️ Partial |
| type_name(&self) | config_value.rs | 191 | ⚠️ Partial |
| new() | config_value.rs | 275 | ⚠️ Partial |
| get(&self, | config_value.rs | 282 | ⚠️ Partial |
| set<K: | config_value.rs | 316 | ⚠️ Partial |
| has(&self, | config_value.rs | 321 | ⚠️ Partial |
| keys(&self) | config_value.rs | 326 | ⚠️ Partial |
| entries(&self) | config_value.rs | 331 | ⚠️ Partial |
| merge(&mut | config_value.rs | 336 | ⚠️ Partial |
| subset(&self, | config_value.rs | 344 | ⚠️ Partial |
| from_extension(path: | sources.rs | 99 | ⚠️ Partial |
| is_enabled(&self) | sources.rs | 117 | ⚠️ Partial |
| file<P: | sources.rs | 143 | ⚠️ Partial |
| environment(prefix: | sources.rs | 151 | ⚠️ Partial |
| command_line(prefix: | sources.rs | 159 | ⚠️ Partial |
| remote(url: | sources.rs | 166 | ⚠️ Partial |
| memory() | sources.rs | 174 | ⚠️ Partial |
| database(connection_string: | sources.rs | 179 | ⚠️ Partial |
| name(&self) | sources.rs | 187 | ⚠️ Partial |
| description(&self) | sources.rs | 199 | ⚠️ Partial |
| status_code(&self) | error.rs | 56 | ✅ Complete |
| default_message(&self) | error.rs | 72 | ⚠️ Partial |
| new(code: | error.rs | 111 | ✅ Complete |
| with_source<E>(mut | error.rs | 122 | ⚠️ Partial |
| with_details(mut | error.rs | 131 | ⚠️ Partial |
| with_request_id(mut | error.rs | 137 | ⚠️ Partial |
| configuration(message: | error.rs | 143 | ⚠️ Partial |
| validation(message: | error.rs | 148 | ⚠️ Partial |
| authentication(message: | error.rs | 153 | ⚠️ Partial |
| authorization(message: | error.rs | 158 | ⚠️ Partial |
| not_found(message: | error.rs | 163 | ⚠️ Partial |
| conflict(message: | error.rs | 168 | ⚠️ Partial |
| internal(message: | error.rs | 173 | ⚠️ Partial |
| external(message: | error.rs | 178 | ⚠️ Partial |
| timeout(message: | error.rs | 183 | ⚠️ Partial |
| database(message: | error.rs | 188 | ⚠️ Partial |
| cache(message: | error.rs | 193 | ⚠️ Partial |
| plugin(message: | error.rs | 198 | ⚠️ Partial |
| component(message: | error.rs | 203 | ⚠️ Partial |
| to_json(&self) | error.rs | 208 | ⚠️ Partial |
| is_code(&self, | error.rs | 229 | ⚠️ Partial |
| is_validation(&self) | error.rs | 234 | ⚠️ Partial |
| is_authentication(&self) | error.rs | 239 | ⚠️ Partial |
| is_authorization(&self) | error.rs | 244 | ⚠️ Partial |
| is_not_found(&self) | error.rs | 249 | ⚠️ Partial |
| is_conflict(&self) | error.rs | 254 | ⚠️ Partial |
| is_internal(&self) | error.rs | 259 | ⚠️ Partial |
| is_configuration(&self) | error.rs | 264 | ⚠️ Partial |
| new() | config.rs | 22 | ✅ Complete |
| default() | config.rs | 29 | ⚠️ Partial |
| from_file(_path: | config.rs | 34 | ⚠️ Partial |
| get<T: | config.rs | 40 | ⚠️ Partial |
| set<T: | config.rs | 57 | ⚠️ Partial |
| is_empty(&self) | config.rs | 66 | ⚠️ Partial |
| has(&self, | config.rs | 71 | ⚠️ Partial |
| keys(&self) | config.rs | 76 | ⚠️ Partial |
| version() | lib.rs | 42 | ⚠️ Partial |
| init() | lib.rs | 49 | ✅ Complete |
| init_with_config(config: | lib.rs | 57 | ✅ Complete |
| init_application() | lib.rs | 64 | ✅ Complete |
| init_application_with_environment(env: | lib.rs | 71 | ✅ Complete |
| new() | lib.rs | 87 | ⚠️ Partial |
| register<T: | lib.rs | 94 | ⚠️ Partial |
| get<T: | lib.rs | 100 | ⚠️ Partial |
| contains(&self, | lib.rs | 108 | ⚠️ Partial |
| names(&self) | lib.rs | 114 | ⚠️ Partial |

