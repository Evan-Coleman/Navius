---
title: "Redis-rs Integration Roadmap"
description: "Integration of redis-rs crate into navius-cache"
category: roadmap
tags:
  - cache
  - redis
  - integration
  - plugin system
last_updated: May 31, 2024
version: 1.3
---

# Redis-rs Integration Roadmap

## Overview
This roadmap outlines the plan to integrate the redis-rs crate into the navius-cache system. After unsuccessful attempts with the custom navius-cache-redis implementation, we'll leverage the well-maintained redis-rs crate to provide optional Redis capabilities for our caching layer through a plugin architecture.

## Current Status
- The custom navius-cache-redis crate has been removed due to implementation challenges
- We've identified redis-rs as a mature, well-tested alternative with a rich feature set
- Initial analysis of redis-rs capabilities and integration approach is complete
- Research findings and detailed mapping between navius-cache interfaces and redis-rs documented
- Basic plugin structure has been created with core operations implemented
- All Priority 1 features for the first release have been implemented
- Comprehensive documentation and examples have been added
- Testing utilities for both real Redis and mock Redis implementations are complete
- Plugin is ready for integration testing and evaluation

## Target State
- A fully functional Redis cache implementation using the redis-rs crate
- Integration with the navius-cache system through the plugin architecture
- Optional activation via feature flags and configuration options
- Comprehensive test coverage and examples
- Potential improvements to navius-cache interfaces to better align with Redis capabilities

## First Release Focus
For the initial release, we focused on reliability and core functionality:

### Priority 1 (Completed)
- ✅ Basic key-value operations (get/set/delete)
- ✅ Batch operations (get_many/set_many/delete_many)
- ✅ Connection pooling with proper error handling
- ✅ Health checks and automatic reconnection
- ✅ TTL and expiration management
- ✅ Key prefixing and configuration
- ✅ Comprehensive error handling
- ✅ Increment/counter operations
- ✅ Basic list operations (push, pop, range, length)
- ✅ Robust connection management with retry and reconnection logic

### Priority 2 (Deferred Features)
- ⏳ Advanced Redis collections (sets, sorted sets, complex hash operations)
- ⏳ Redis Cluster support
- ⏳ TLS encryption
- ⏳ Redis JSON support
- ⏳ PubSub functionality
- ⏳ Transaction support
- ⏳ Advanced Redis commands

## Implementation Progress Tracking

### Phase 1: Research and Analysis
1. **Dependency Evaluation**
   - [x] Analyze redis-rs crate features and compatibility with our requirements
   - [x] Identify necessary feature flags to include/exclude
   - [x] Evaluate TLS support options (rustls vs native-tls)
   - [x] Review licensing and dependency tree for compliance

   *Updated at: May 30, 2024 - Completed initial analysis of redis-rs features and compatibility*

2. **Cache Interface Mapping**
   - [x] Compare redis-rs API with our navius-cache interfaces
   - [x] Identify gaps and overlaps in functionality
   - [x] Determine required adapter patterns for integration
   - [x] Document extension points for Redis-specific features

   *Updated at: May 30, 2024 - Completed interface mapping analysis and adapter pattern design*

3. **Plugin System Integration Analysis**
   - [x] Review navius plugin system architecture
   - [x] Design Redis cache plugin registration mechanism
   - [x] Determine configuration requirements for the Redis plugin
   - [x] Plan feature flag structure for optional Redis support

   *Updated at: May 30, 2024 - Completed plugin system integration analysis and configuration planning*

4. **Interface Improvement Analysis**
   - [x] Identify potential improvements to navius-cache interfaces
   - [x] Evaluate changes that would better align with Redis capabilities
   - [x] Determine backward compatibility strategy

   *Updated at: May 30, 2024 - Added analysis of potential interface improvements*

### Phase 2: Implementation
1. **Basic Redis Plugin Structure** 
   - [x] Create navius-cache-redis-plugin crate with minimal structure
   - [x] Set up proper cargo.toml with redis-rs dependency
   - [x] Implement plugin registration mechanism
   - [x] Add feature flags for optional Redis support

   *Updated at: May 31, 2024 - Completed basic plugin structure implementation*

2. **Core Cache Operations (Priority 1)** 
   - [x] Implement adapter for basic get/set operations
   - [x] Add support for key expiration
   - [x] Implement batch operations via pipelining
   - [x] Add atomic operations support (incr/decr)
   - [x] Complete list operations (pop, length, range)
   - [x] Test and verify implemented operations

   *Updated at: May 31, 2024 - Completed all Priority 1 core operations with unit tests*

3. **Connection Management (Priority 1)**
   - [x] Implement connection pooling
   - [x] Add health check functionality
   - [x] Improve reconnection handling and error recovery
   - [x] Implement timeout and retry strategies

   *Updated at: May 31, 2024 - Completed enhanced connection manager with retry logic and error recovery*

4. **Advanced Cache Functionality (Priority 2)** 
   - [ ] Implement collections support (sets, sorted sets, lists, hashes)
   - [ ] Add support for pub/sub if needed
   - [ ] Implement Redis-specific features as extensions
   - [ ] Add transaction support

   *Updated at: Not started - Deferred to future releases*

5. **Security Features (Priority 2)**
   - [ ] Implement TLS support options
   - [ ] Support for cluster connections if needed
   - [ ] Add Redis authentication handling

   *Updated at: Not started - Basic authentication supported, advanced features deferred*

6. **Interface Improvements** (if decided)
   - [ ] Update CacheOperations trait to better align with Redis capabilities
   - [ ] Enhance error handling for Redis-specific scenarios
   - [ ] Add extension traits for Redis-specific features
   - [ ] Ensure backward compatibility with existing cache implementations

   *Updated at: Not started - Deferred to future releases after gathering usage feedback*

### Phase 3: Testing and Documentation
1. **Test Infrastructure**
   - [x] Create unit test suite for Redis plugin
   - [x] Add test utilities for Redis testing
   - [x] Implement integration tests with an in-memory mock Redis
   - [ ] Setup CI pipeline for Redis tests

   *Updated at: May 31, 2024 - Completed test utilities and integration tests with mock Redis*

2. **Documentation and Examples**
   - [x] Create basic API documentation
   - [x] Add comprehensive usage examples
   - [x] Document configuration options and error handling
   - [x] Provide framework integration examples (e.g., Axum)
   - [ ] Add performance benchmarks

   *Updated at: May 31, 2024 - Completed comprehensive documentation and examples*

3. **Security Review**
   - [ ] Analyze security implications of Redis integration
   - [ ] Ensure proper TLS configuration options
   - [ ] Document security best practices
   - [ ] Review error handling for sensitive information leakage

   *Updated at: Not started*

## Implementation Status
- **Overall Progress**: 75% complete
- **Last Updated**: May 31, 2024
- **Next Milestone**: Complete CI/CD integration and benchmark performance
- **Current Focus**: Finalizing integration tests with real Redis servers

## Success Criteria
1. ✅ All Priority 1 features are fully implemented and tested
2. ✅ Plugin system successfully loads Redis implementation when configured
3. ✅ Comprehensive test utilities are available for developers
4. ⏳ Basic performance tests show acceptable latency (<10ms per operation)
5. ✅ Documentation provides clear usage examples and configuration guidelines
6. ✅ Error handling is comprehensive and gracefully manages Redis failures

## Technical Approach

### Redis-rs Feature Analysis
The redis-rs crate offers several optional features that we've evaluated:

1. **Runtime Options**
   - `tokio-comp`: Tokio runtime support for async operations - **RECOMMENDED** (as our project uses Tokio)
   - `async-std-comp`: Async-std runtime support - **NOT NEEDED** (we use Tokio)
   
2. **TLS Options**
   - `tls-rustls`: RusTLS for TLS support - **FUTURE ENHANCEMENT**
   - `tls-native-tls`: Native TLS support (OpenSSL) - **FUTURE ENHANCEMENT**
   
3. **Functionality**
   - `connection-manager`: Automatic reconnection support - **RECOMMENDED** (for reliability)
   - `cluster`: Redis cluster support - **FUTURE ENHANCEMENT**
   - `cluster-async`: Async cluster support - **FUTURE ENHANCEMENT**
   - `json`: RedisJSON support - **FUTURE ENHANCEMENT**
   - `ahash`: Performance optimizations - **RECOMMENDED** (7-10% performance improvement)

Our recommended feature set for the initial implementation:
```toml
[dependencies]
redis = { version = "0.29.2", features = ["tokio-comp", "connection-manager", "ahash"] }
```

Additional features can be enabled via optional feature flags in our plugin crate.

### Cache Interface Mapping

Based on our analysis of the navius-cache interfaces, we've mapped the operations to redis-rs as follows:

#### Basic Key-Value Operations (Priority 1)
| navius-cache | redis-rs |
|-------------|----------|
| `get<K, V>(&self, key: K) -> CacheResult<Option<V>>` | `redis::AsyncCommands::get(&mut conn, key)` |
| `set<K, V>(&self, key: K, value: &V, options: Option<CacheOptions>) -> CacheResult<()>` | `redis::AsyncCommands::set_ex(&mut conn, key, value, ttl.as_secs())` |
| `delete<K>(&self, key: K) -> CacheResult<bool>` | `redis::AsyncCommands::del(&mut conn, key)` |
| `exists<K>(&self, key: K) -> CacheResult<bool>` | `redis::AsyncCommands::exists(&mut conn, key)` |
| `expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>` | `redis::AsyncCommands::expire(&mut conn, key, ttl.as_secs() as usize)` |

#### Batch Operations (Priority 1)
| navius-cache | redis-rs |
|-------------|----------|
| `get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>` | `redis::AsyncCommands::get(&mut conn, keys)` |
| `set_many<K, V>(&self, entries: Vec<(K, V)>, options: Option<CacheOptions>) -> CacheResult<()>` | Pipeline multiple `set` operations |
| `delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>` | `redis::AsyncCommands::del(&mut conn, keys)` |

#### List Operations (Priority 1)
| navius-cache | redis-rs |
|-------------|----------|
| `list_push_right<K, V>(&self, key: K, value: &V) -> CacheResult<usize>` | `redis::AsyncCommands::rpush(&mut conn, key, value)` |
| `list_push_left<K, V>(&self, key: K, value: &V) -> CacheResult<usize>` | `redis::AsyncCommands::lpush(&mut conn, key, value)` |
| `list_pop_right<K, V>(&self, key: K) -> CacheResult<Option<V>>` | `redis::AsyncCommands::rpop(&mut conn, key)` |
| `list_pop_left<K, V>(&self, key: K) -> CacheResult<Option<V>>` | `redis::AsyncCommands::lpop(&mut conn, key)` |
| `list_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>` | `redis::AsyncCommands::lrange(&mut conn, key, start, stop)` |
| `list_length<K>(&self, key: K) -> CacheResult<usize>` | `redis::AsyncCommands::llen(&mut conn, key)` |
| `list_remove<K, V>(&self, key: K, count: i32, value: &V) -> CacheResult<usize>` | `redis::AsyncCommands::lrem(&mut conn, key, count, value)` |

#### Hash Operations (Priority 2)
| navius-cache | redis-rs |
|-------------|----------|
| `hash_get<K, F, V>(&self, key: K, field: F) -> CacheResult<Option<V>>` | `redis::AsyncCommands::hget(&mut conn, key, field)` |
| `hash_set<K, F, V>(&self, key: K, field: F, value: &V) -> CacheResult<bool>` | `redis::AsyncCommands::hset(&mut conn, key, field, value)` |
| `hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>` | `redis::AsyncCommands::hexists(&mut conn, key, field)` |
| `hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>` | `redis::AsyncCommands::hdel(&mut conn, key, fields)` |
| `hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>` | `redis::AsyncCommands::hincr(&mut conn, key, field, amount)` |
| `hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>` | `redis::AsyncCommands::hgetall(&mut conn, key)` |

#### Set Operations (Priority 2)
| navius-cache | redis-rs |
|-------------|----------|
| `set_add<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>` | `redis::AsyncCommands::sadd(&mut conn, key, values)` |
| `set_remove<K, V>(&self, key: K, values: Vec<V>) -> CacheResult<usize>` | `redis::AsyncCommands::srem(&mut conn, key, values)` |
| `set_contains<K, V>(&self, key: K, value: &V) -> CacheResult<bool>` | `redis::AsyncCommands::sismember(&mut conn, key, value)` |
| `set_members<K, V>(&self, key: K) -> CacheResult<Vec<V>>` | `redis::AsyncCommands::smembers(&mut conn, key)` |
| `set_length<K>(&self, key: K) -> CacheResult<usize>` | `redis::AsyncCommands::scard(&mut conn, key)` |

#### Sorted Set Operations (Priority 2)
| navius-cache | redis-rs |
|-------------|----------|
| `zset_add<K, V>(&self, key: K, items: Vec<(f64, V)>) -> CacheResult<usize>` | `redis::AsyncCommands::zadd(&mut conn, key, items)` |
| `zset_remove<K, V>(&self, key: K, members: Vec<V>) -> CacheResult<usize>` | `redis::AsyncCommands::zrem(&mut conn, key, members)` |
| `zset_score<K, V>(&self, key: K, member: &V) -> CacheResult<Option<f64>>` | `redis::AsyncCommands::zscore(&mut conn, key, member)` |
| `zset_range<K, V>(&self, key: K, start: isize, stop: isize) -> CacheResult<Vec<V>>` | `redis::AsyncCommands::zrange(&mut conn, key, start, stop)` |
| `zset_length<K>(&self, key: K) -> CacheResult<usize>` | `redis::AsyncCommands::zcard(&mut conn, key)` |

### Testing Utilities

We've implemented two key testing utilities to simplify both integration and unit testing:

1. **TestRedisServer**: A wrapper for a real Redis connection that:
   - Creates isolated test namespaces with unique prefixes
   - Handles automatic cleanup of test data
   - Provides helper methods for testing with real Redis servers

2. **MockRedis**: An in-memory mock Redis implementation that:
   - Implements the full `Cache` and `CacheOperations` traits
   - Simulates Redis functionality without requiring a Redis server
   - Supports TTL, serialization, and all Core Operations
   - Makes unit testing fast and reliable

These utilities enable both unit testing without Redis dependencies and integration testing with actual Redis instances.

### Examples

We've provided comprehensive examples to demonstrate Redis cache usage:

1. **Basic Usage**: Demonstrates core operations like get, set, delete with TTL management
2. **Advanced Configuration**: Shows how to configure connection pooling, timeouts, and Redis options
3. **Error Handling**: Illustrates proper error handling for different Redis failure scenarios
4. **Framework Integration**: Demonstrates how to integrate with Axum web framework

These examples provide developers with practical guidance for integrating the Redis cache into their applications.

### Potential Interface Improvements

After analyzing both the navius-cache interfaces and redis-rs capabilities, we've identified several potential improvements that would make the navius-cache interfaces work better with Redis:

1. **Pipeline Support**: Add explicit pipeline support to the CacheOperations trait to leverage Redis's efficient pipelining capability. This would benefit both Redis and in-memory implementations:

```rust
async fn pipeline<F, R>(&self, operations: F) -> CacheResult<R>
where
    F: FnOnce(&mut Pipeline) -> R + Send + 'static,
    R: Send + 'static;
```

2. **Bulk Operations Optimization**: Enhance batch operations to include options like whether atomicity is required, allowing Redis to optimize commands.

These improvements are deferred to future releases after gathering usage feedback from the initial implementation.

## References
- [Redis-rs Documentation](https://docs.rs/redis/latest/redis/)
- [Redis-rs GitHub Repository](https://github.com/redis-rs/redis-rs)
- Navius Cache Provider Guide (CACHE_PROVIDER_GUIDE.md)
- [Redis-rs Feature Analysis and Mapping](./redis-rs-integration.md#technical-approach)

*Updated at: May 31, 2024 - Added comprehensive documentation examples and testing utilities* 