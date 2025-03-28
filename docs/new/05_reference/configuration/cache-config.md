---
title: "Cache Configuration Reference"
description: "Detailed reference for all configuration options available in the Navius caching system"
category: reference
tags:
  - reference
  - configuration
  - caching
  - redis
  - settings
related:
  - ../../guides/caching-strategies.md
  - ../../guides/features/caching.md
  - ../patterns/caching-patterns.md
last_updated: March 26, 2025
version: 1.0
---

# Cache Configuration Reference

This document provides detailed information about all configuration options for the Navius caching system.

## Overview

Navius provides a flexible and configurable caching system that can be customized to suit different environments and use cases. The cache configuration is defined in the application configuration files.

## Configuration Structure

The cache configuration section in your application configuration looks like this:

```yaml
cache:
  # Global cache configuration
  enabled: true
  default_provider: "memory"
  default_ttl_seconds: 300  # 5 minutes
  
  # Provider-specific configurations
  providers:
    - name: "memory"
      enabled: true
      type: "moka"
      capacity: 10000  # items
      eviction_policy: "LRU"
      ttl_seconds: 300  # 5 minutes
    
    - name: "redis"
      enabled: true
      type: "redis"
      connection_string: "redis://localhost:6379"
      ttl_seconds: 3600  # 1 hour
      connection_pool_size: 10
      timeout_ms: 500
      
  # Two-tier cache configuration
  two_tier:
    enabled: true
    fast_cache_provider: "memory"
    slow_cache_provider: "redis"
    fast_cache_ttl_seconds: 60   # 1 minute
    slow_cache_ttl_seconds: 3600 # 1 hour
    promotion_enabled: true
```

## Configuration Options

### Global Cache Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enables or disables the entire caching system |
| `default_provider` | string | `"memory"` | The name of the default cache provider to use |
| `default_ttl_seconds` | integer | `300` | Default time-to-live in seconds for cache entries |

### Provider Configuration

Each provider has the following configuration options:

#### Common Provider Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `name` | string | (required) | Unique identifier for the provider |
| `enabled` | boolean | `true` | Enables or disables this provider |
| `type` | string | (required) | Type of provider (e.g., "moka", "redis") |
| `ttl_seconds` | integer | Global default | Default TTL for this provider |

#### Memory Provider Options (type: "moka")

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `capacity` | integer | `10000` | Maximum number of items in the cache |
| `eviction_policy` | string | `"LRU"` | Eviction policy, one of: "LRU", "LFU", "FIFO" |
| `time_to_idle_seconds` | integer | None | Time after which an entry is evicted if not accessed |
| `expire_after_access_seconds` | integer | None | Time after which an entry is evicted after last access |
| `expire_after_write_seconds` | integer | None | Time after which an entry is evicted after creation |

#### Redis Provider Options (type: "redis")

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `connection_string` | string | `"redis://localhost:6379"` | Redis connection URI |
| `connection_pool_size` | integer | `5` | Size of the connection pool |
| `timeout_ms` | integer | `1000` | Connection timeout in milliseconds |
| `retry_attempts` | integer | `3` | Number of retry attempts for failed operations |
| `retry_delay_ms` | integer | `100` | Delay between retry attempts in milliseconds |
| `cluster_mode` | boolean | `false` | Enable Redis cluster mode |
| `sentinel_mode` | boolean | `false` | Enable Redis sentinel mode |
| `sentinel_master` | string | `"mymaster"` | Name of the sentinel master |
| `username` | string | None | Redis username (for Redis 6.0+) |
| `password` | string | None | Redis password |
| `database` | integer | `0` | Redis database index |

### Two-Tier Cache Configuration

The two-tier cache combines two separate cache providers (typically memory and Redis) into a unified caching system.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enables or disables the two-tier cache |
| `fast_cache_provider` | string | `"memory"` | The name of the provider to use as the fast cache |
| `slow_cache_provider` | string | `"redis"` | The name of the provider to use as the slow cache |
| `fast_cache_ttl_seconds` | integer | `60` | TTL for the fast cache in seconds |
| `slow_cache_ttl_seconds` | integer | `3600` | TTL for the slow cache in seconds |
| `promotion_enabled` | boolean | `true` | Enable automatic promotion from slow to fast cache |
| `promotion_lock_ms` | integer | `10` | Promotion lock timeout in milliseconds |

## Environment Variables

You can also configure the cache using environment variables. These will override the values in the configuration files.

| Environment Variable | Description |
|---------------------|-------------|
| `NAVIUS_CACHE_ENABLED` | Enable/disable the entire cache (`true`/`false`) |
| `NAVIUS_CACHE_DEFAULT_PROVIDER` | The default cache provider name |
| `NAVIUS_CACHE_DEFAULT_TTL` | Default TTL in seconds |
| `NAVIUS_CACHE_MEMORY_ENABLED` | Enable/disable the memory cache |
| `NAVIUS_CACHE_MEMORY_CAPACITY` | Memory cache capacity |
| `NAVIUS_CACHE_REDIS_ENABLED` | Enable/disable the Redis cache |
| `NAVIUS_CACHE_REDIS_URL` | Redis connection URL |
| `NAVIUS_CACHE_REDIS_PASSWORD` | Redis password |
| `NAVIUS_CACHE_TWO_TIER_ENABLED` | Enable/disable the two-tier cache |

## Configuration Examples

### Basic Memory-Only Setup

```yaml
cache:
  enabled: true
  default_provider: "memory"
  providers:
    - name: "memory"
      type: "moka"
      capacity: 5000
```

### Production Redis Setup

```yaml
cache:
  enabled: true
  default_provider: "redis"
  providers:
    - name: "redis"
      type: "redis"
      connection_string: "redis://${REDIS_HOST}:${REDIS_PORT}"
      password: "${REDIS_PASSWORD}"
      connection_pool_size: 20
      timeout_ms: 500
```

### Development Two-Tier Setup

```yaml
cache:
  enabled: true
  default_provider: "two-tier"
  providers:
    - name: "memory"
      type: "moka"
      capacity: 1000
    - name: "redis"
      type: "redis"
      connection_string: "redis://localhost:6379"
  two_tier:
    enabled: true
    fast_cache_provider: "memory"
    slow_cache_provider: "redis"
    fast_cache_ttl_seconds: 30
    slow_cache_ttl_seconds: 300
```

### Disabling Cache for Testing

```yaml
cache:
  enabled: false
```

## Reading Cache Configuration in Code

Here's how to access the cache configuration from your code:

```rust
use navius::core::config::CacheConfig;

fn initialize_cache(config: &CacheConfig) {
    if !config.enabled {
        println!("Cache is disabled");
        return;
    }
    
    println!("Using default provider: {}", config.default_provider);
    
    // Access provider-specific configuration
    if let Some(redis_config) = config.get_provider_config("redis") {
        println!("Redis connection: {}", redis_config.get_string("connection_string").unwrap());
    }
    
    // Access two-tier configuration
    if let Some(two_tier_config) = config.two_tier.as_ref() {
        if two_tier_config.enabled {
            println!("Two-tier cache is enabled");
            println!("Fast cache TTL: {}s", two_tier_config.fast_cache_ttl_seconds);
            println!("Slow cache TTL: {}s", two_tier_config.slow_cache_ttl_seconds);
        }
    }
}
```

## Dynamic Cache Configuration

Navius supports dynamic cache configuration changes through the configuration management system. When the configuration is updated, the cache system will automatically apply the relevant changes without requiring a server restart.

Changes that can be applied dynamically:
- Enabling/disabling the cache
- Changing TTL values
- Adjusting capacity limits
- Modifying eviction policies

Changes that require a restart:
- Adding/removing cache providers
- Changing connection strings
- Switching the default provider type 