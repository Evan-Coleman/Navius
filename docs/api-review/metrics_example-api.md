# metrics_example API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- navius-cache
- navius-core
- async-trait
- redis
- serde
- serde_json
- thiserror
- tracing
- tokio
- futures
- rand
- metrics
- uuid

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| RedisInvalidator | invalidation.rs | 18 | ⚠️ Partial |
| RedisCacheConfig | config.rs | 7 | ⚠️ Partial |
| ScriptInfo | lua.rs | 66 | ⚠️ Partial |
| RedisLuaManager | lua.rs | 74 | ✅ Complete |
| TimedOperation | metrics.rs | 152 | ⚠️ Partial |
| RedisPipelineBuilder | pipeline.rs | 51 | ⚠️ Partial |
| RedisPipeline | pipeline.rs | 145 | ✅ Complete |
| PoolStats | connection.rs | 16 | ⚠️ Partial |
| RedisConnectionManager | connection.rs | 88 | ⚠️ Partial |
| RedisCache | operations.rs | 23 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| RedisCacheError | error.rs | 9 | ✅ Complete |
| ConnectionHealth | connection.rs | 35 | ✅ Complete |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| RedisLuaScripting: | lua.rs | 18 | ⚠️ Partial |
| RedisPipeline: | pipeline.rs | 23 | ⚠️ Partial |
| Pipeline | pipeline.rs | 138 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(connection_manager: | invalidation.rs | 25 | ✅ Complete |
| handle_redis_result<T>( | error.rs | 62 | ⚠️ Partial |
| new(url: | config.rs | 149 | ⚠️ Partial |
| with_ttl(url: | config.rs | 173 | ⚠️ Partial |
| high_availability(url: | config.rs | 180 | ⚠️ Partial |
| prefixed_key<K: | config.rs | 204 | ⚠️ Partial |
| connection_timeout(&self) | config.rs | 213 | ⚠️ Partial |
| command_timeout(&self) | config.rs | 218 | ⚠️ Partial |
| idle_timeout(&self) | config.rs | 223 | ⚠️ Partial |
| max_lifetime(&self) | config.rs | 228 | ⚠️ Partial |
| health_check_interval(&self) | config.rs | 233 | ⚠️ Partial |
| circuit_reset_timeout(&self) | config.rs | 238 | ⚠️ Partial |
| new(connection_manager: | lua.rs | 83 | ✅ Complete |
| register(&self, | lua.rs | 91 | ⚠️ Partial |
| get_script(&self, | lua.rs | 105 | ⚠️ Partial |
| script_exists(&self, | lua.rs | 111 | ⚠️ Partial |
| get_script_hash(&self, | lua.rs | 116 | ⚠️ Partial |
| fn | lua.rs | 126 | ⚠️ Partial |
| fn | lua.rs | 424 | ⚠️ Partial |
| record_operation<T>(name: | metrics.rs | 60 | ✅ Complete |
| record_operation_duration(name: | metrics.rs | 71 | ⚠️ Partial |
| record_operation_success(name: | metrics.rs | 78 | ⚠️ Partial |
| record_operation_error<T>(name: | metrics.rs | 84 | ⚠️ Partial |
| record_connection_pool_stats(size: | metrics.rs | 106 | ⚠️ Partial |
| record_connection_acquisition(duration: | metrics.rs | 113 | ⚠️ Partial |
| record_connection_health(health: | metrics.rs | 120 | ⚠️ Partial |
| new(name: | metrics.rs | 159 | ⚠️ Partial |
| record<T>(&self, | metrics.rs | 167 | ⚠️ Partial |
| record_success(&self) | metrics.rs | 172 | ⚠️ Partial |
| record_error(&self, | metrics.rs | 178 | ⚠️ Partial |
| new() | pipeline.rs | 60 | ✅ Complete |
| get(mut | pipeline.rs | 68 | ⚠️ Partial |
| set(mut | pipeline.rs | 75 | ⚠️ Partial |
| setex(mut | pipeline.rs | 82 | ⚠️ Partial |
| del(mut | pipeline.rs | 89 | ⚠️ Partial |
| exists(mut | pipeline.rs | 96 | ⚠️ Partial |
| incr(mut | pipeline.rs | 103 | ⚠️ Partial |
| cmd(mut | pipeline.rs | 110 | ⚠️ Partial |
| build(self) | pipeline.rs | 121 | ⚠️ Partial |
| operation_count(&self) | pipeline.rs | 126 | ⚠️ Partial |
| new(connection_manager: | pipeline.rs | 156 | ✅ Complete |
| cmd<T: | pipeline.rs | 166 | ⚠️ Partial |
| set<V: | pipeline.rs | 177 | ⚠️ Partial |
| get(&mut | pipeline.rs | 184 | ⚠️ Partial |
| del(&mut | pipeline.rs | 191 | ⚠️ Partial |
| exists(&mut | pipeline.rs | 198 | ⚠️ Partial |
| expire(&mut | pipeline.rs | 205 | ⚠️ Partial |
| ttl(&mut | pipeline.rs | 212 | ⚠️ Partial |
| hmset<K: | pipeline.rs | 219 | ⚠️ Partial |
| hget(&mut | pipeline.rs | 230 | ⚠️ Partial |
| hdel(&mut | pipeline.rs | 237 | ⚠️ Partial |
| hgetall(&mut | pipeline.rs | 244 | ⚠️ Partial |
| rpush<V: | pipeline.rs | 251 | ⚠️ Partial |
| lpush<V: | pipeline.rs | 258 | ⚠️ Partial |
| rpop(&mut | pipeline.rs | 265 | ⚠️ Partial |
| lpop(&mut | pipeline.rs | 272 | ⚠️ Partial |
| lrange(&mut | pipeline.rs | 279 | ⚠️ Partial |
| llen(&mut | pipeline.rs | 286 | ⚠️ Partial |
| sadd<M: | pipeline.rs | 293 | ⚠️ Partial |
| srem<M: | pipeline.rs | 300 | ⚠️ Partial |
| smembers(&mut | pipeline.rs | 307 | ⚠️ Partial |
| fn | connection.rs | 108 | ✅ Complete |
| fn | connection.rs | 183 | ⚠️ Partial |
| fn | connection.rs | 384 | ⚠️ Partial |
| fn | connection.rs | 414 | ⚠️ Partial |
| prefixed_key<K: | connection.rs | 486 | ⚠️ Partial |
| key_prefix(&self) | connection.rs | 495 | ⚠️ Partial |
| fn | connection.rs | 500 | ⚠️ Partial |
| config(&self) | connection.rs | 508 | ⚠️ Partial |
| fn | connection.rs | 513 | ⚠️ Partial |
| new(connection_manager: | operations.rs | 34 | ✅ Complete |
| with_serializer<S: | operations.rs | 39 | ⚠️ Partial |
| connection_manager(&self) | operations.rs | 58 | ⚠️ Partial |
| serializer(&self) | operations.rs | 63 | ⚠️ Partial |
| lua_manager(&self) | operations.rs | 68 | ⚠️ Partial |
| with_lua_scripting(mut | operations.rs | 73 | ⚠️ Partial |
| fn | operations.rs | 83 | ⚠️ Partial |
| fn | operations.rs | 103 | ⚠️ Partial |
| fn | operations.rs | 119 | ⚠️ Partial |
| fn | operations.rs | 173 | ⚠️ Partial |
| fn | operations.rs | 206 | ⚠️ Partial |
| fn | operations.rs | 219 | ⚠️ Partial |
| fn | operations.rs | 232 | ⚠️ Partial |
| fn | operations.rs | 245 | ⚠️ Partial |
| fn | operations.rs | 261 | ⚠️ Partial |
| fn | operations.rs | 280 | ⚠️ Partial |
| fn | operations.rs | 298 | ⚠️ Partial |
| fn | operations.rs | 314 | ⚠️ Partial |
| fn | operations.rs | 330 | ⚠️ Partial |
| fn | operations.rs | 355 | ⚠️ Partial |
| fn | operations.rs | 373 | ⚠️ Partial |
| fn | operations.rs | 389 | ⚠️ Partial |
| fn | operations.rs | 405 | ⚠️ Partial |
| fn | operations.rs | 423 | ⚠️ Partial |
| fn | operations.rs | 443 | ⚠️ Partial |
| fn | operations.rs | 460 | ⚠️ Partial |

