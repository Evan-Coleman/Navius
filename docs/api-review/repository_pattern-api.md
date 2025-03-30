# repository_pattern API Inventory

**Version:** 0.1.0  
**Documentation Coverage:** 100%  
**Status:** ✅ Good

## Dependencies

- navius-core
- async-trait
- thiserror
- serde
- tracing
- futures
- tokio
- redis
- metrics

## Public Structs

| Name | File | Line | Documentation |
|------|------|------|---------------|
| CacheInvalidator<C: | invalidation.rs | 42 | ✅ Complete |
| CacheConfig | config.rs | 5 | ✅ Complete |
| RedisConfig | redis.rs | 15 | ⚠️ Partial |
| RedisCache | redis.rs | 39 | ⚠️ Partial |
| CacheTimer | metrics.rs | 124 | ⚠️ Partial |
| CacheConnectionManager<C: | connection.rs | 11 | ⚠️ Partial |
| CacheOptions | operations.rs | 29 | ⚠️ Partial |

## Public Enums

| Name | File | Line | Documentation |
|------|------|------|---------------|
| InvalidationStrategy | invalidation.rs | 10 | ⚠️ Partial |
| CacheError | error.rs | 6 | ⚠️ Partial |
| CacheOperation | metrics.rs | 13 | ⚠️ Partial |
| CacheResult | metrics.rs | 59 | ⚠️ Partial |

## Public Traits

| Name | File | Line | Documentation |
|------|------|------|---------------|
| CacheInvalidation<C: | invalidation.rs | 23 | ✅ Complete |
| CacheKey: | operations.rs | 8 | ⚠️ Partial |
| CacheOperations: | operations.rs | 55 | ⚠️ Partial |
| Cache: | operations.rs | 441 | ⚠️ Partial |

## Public Functions

| Name | File | Line | Documentation |
|------|------|------|---------------|
| new(cache: | invalidation.rs | 53 | ✅ Complete |
| new(url: | config.rs | 44 | ⚠️ Partial |
| for_testing() | config.rs | 55 | ⚠️ Partial |
| prefixed_key(&self, | config.rs | 69 | ⚠️ Partial |
| with_connect_timeout(mut | config.rs | 74 | ⚠️ Partial |
| with_max_connections(mut | config.rs | 80 | ⚠️ Partial |
| with_trace(mut | config.rs | 86 | ⚠️ Partial |
| with_metrics(mut | config.rs | 92 | ⚠️ Partial |
| fn | redis.rs | 49 | ✅ Complete |
| as_str(&self) | metrics.rs | 40 | ✅ Complete |
| as_str(&self) | metrics.rs | 72 | ✅ Complete |
| record_cache_metric( | metrics.rs | 87 | ✅ Complete |
| record_cache_metric( | metrics.rs | 113 | ⚠️ Partial |
| new(operation: | metrics.rs | 135 | ✅ Complete |
| success(self) | metrics.rs | 144 | ⚠️ Partial |
| error(self) | metrics.rs | 155 | ⚠️ Partial |
| hit(self) | metrics.rs | 161 | ⚠️ Partial |
| miss(self) | metrics.rs | 167 | ⚠️ Partial |
| new(cache: | connection.rs | 20 | ✅ Complete |
| cache(&self) | connection.rs | 28 | ⚠️ Partial |
| config(&self) | connection.rs | 33 | ⚠️ Partial |
| fn | connection.rs | 43 | ⚠️ Partial |
| new() | operations.rs | 42 | ⚠️ Partial |
| ttl(mut | operations.rs | 47 | ⚠️ Partial |

