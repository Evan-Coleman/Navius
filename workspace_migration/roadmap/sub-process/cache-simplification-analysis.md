---
title: "Cache Operations Analysis"
description: "Analysis of the current cache interface to identify core operations"
category: analysis
tags:
  - cache
  - simplification
  - analysis
last_updated: April 4, 2024
version: 1.0
---

# Cache Operations Analysis

## Overview
This document analyzes the current cache interface to identify which operations are essential for most applications and which operations can be moved to extension traits. This analysis informs the cache simplification initiative.

## Current CacheOperations Trait
The current `CacheOperations` trait in `navius-cache` includes approximately 42 methods across various categories:

1. **Basic Key-Value Operations** - 10 methods
   - get, get_many
   - set, set_many
   - delete, delete_many
   - exists
   - increment
   - expire
   - clear

2. **List Operations** - 10 methods
   - list_push_right, list_push_right_many
   - list_push_left, list_push_left_many
   - list_pop_right, list_pop_left
   - list_range
   - list_length
   - list_remove
   - list_trim, list_set

3. **Hash Map Operations** - 12 methods
   - hash_get, hash_get_many
   - hash_set, hash_set_many
   - hash_exists
   - hash_delete
   - hash_get_all
   - hash_keys, hash_values
   - hash_increment
   - hash_length

4. **Set Operations** - 10+ methods
   - set_add, set_contains
   - set_remove
   - set_members, set_length
   - set_intersection, set_union, set_difference
   - set_intersection_store, set_union_store, set_difference_store
   - set_random_members

5. **Sorted Set Operations** - 16+ methods
   - zset_add, zset_remove
   - zset_score, zset_increment_score
   - zset_range, zset_range_with_scores
   - zset_range_by_score, zset_range_by_score_with_scores
   - zset_rank, zset_rev_rank
   - zset_length, zset_count
   - zset_rev_range_by_score, zset_rev_range_by_score_with_scores
   - zset_remove_range_by_rank, zset_remove_range_by_score
   - zset_intersection_store, zset_union_store

6. **Health Checks**
   - health_check

## Usage Analysis
Based on a review of the codebase and common caching patterns, we can estimate the usage frequency of these operations:

| Operation Category | Usage Frequency | Notes |
|-------------------|-----------------|-------|
| Basic Key-Value   | ~90%           | Core functionality used by almost all applications |
| List Operations   | ~30%           | Used for queues, event streams, recent items |
| Hash Map Operations| ~20%          | Used for structured data, aggregations |
| Set Operations    | ~10%           | Used for unique collections, intersections |
| Sorted Set Operations | ~5%        | Used for leaderboards, time-ordered data |

## Essential Operations Analysis
Analyzing which operations are essential for most applications, we identify:

### Must-Have Operations (BasicCache trait)
1. **get** - Retrieve a single cached item
2. **set** - Store a single item in the cache with optional TTL
3. **delete** - Remove an item from the cache
4. **exists** - Check if a key exists in the cache
5. **expire** - Set expiration for a key
6. **clear** - Clear the entire cache
7. **health_check** - Check cache connectivity/health

### Commonly Used Extensions (ListOperations trait)
1. **list_push_right** - Add to the end of a list
2. **list_pop_left** - Remove from the beginning of a list (queue pattern)
3. **list_length** - Get the length of a list

### Medium Priority Operations (HashOperations trait)
1. **hash_get** - Get a field from a hash
2. **hash_set** - Set a field in a hash
3. **hash_exists** - Check if a field exists in a hash

### Lower Priority Operations (SetOperations trait)
1. **set_add** - Add member to a set
2. **set_contains** - Check if member is in a set
3. **set_remove** - Remove from set

### Specialized Operations (SortedSetOperations trait)
1. **zset_add** - Add to a sorted set with score
2. **zset_range** - Get range from a sorted set
3. **zset_score** - Get score of an element

## Batch Operations Considerations
Batch operations like `get_many` and `set_many` provide performance benefits but add complexity. Options:

1. **Include in BasicCache** - Makes the core interface larger but covers important performance use cases
2. **Create BatchOperations trait** - Separates batch operations into their own extension trait
3. **Implement on top of core operations** - Provide default implementations that call core methods in a loop

After analysis, we recommend keeping `get_many` and `set_many` in the core BasicCache trait as they provide significant performance benefits for common operations.

## Numeric Operations
The `increment` operation is common enough to include in BasicCache, as counters are a frequent use case.

## Implementation Complexity Analysis
Each operation has varying implementation complexity across different backends:

| Operation | Memory Cache | Redis | DynamoDB | File-based |
|-----------|-------------|-------|----------|------------|
| Basic Key-Value | Easy | Easy | Easy | Easy |
| Batch Operations | Medium | Easy | Medium | Hard |
| List Operations | Medium | Easy | Hard | Very Hard |
| Hash Operations | Medium | Easy | Medium | Hard |
| Set Operations | Medium | Easy | Hard | Very Hard |
| Sorted Set Operations | Hard | Easy | Very Hard | Very Hard |

This analysis confirms that Redis naturally supports all operations, while other backends struggle with more complex data structures.

## Recommendations for BasicCache Trait

Based on this analysis, we recommend the following methods for the BasicCache trait:

```rust
#[async_trait]
pub trait BasicCache: Send + Sync + 'static {
    /// Get a value from the cache
    async fn get<K, V>(&self, key: K) -> CacheResult<Option<V>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static;

    /// Get multiple values from the cache
    async fn get_many<K, V>(&self, keys: Vec<K>) -> CacheResult<Vec<Option<V>>>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: DeserializeOwned + Send + 'static;

    /// Set a value in the cache with optional TTL
    async fn set<K, V>(&self, key: K, value: &V, ttl: Option<Duration>) -> CacheResult<()>
    where
        K: CacheKey + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Set multiple values in the cache with optional TTL
    async fn set_many<K, V>(&self, entries: Vec<(K, V)>, ttl: Option<Duration>) -> CacheResult<()>
    where
        K: CacheKey + Eq + std::hash::Hash + std::fmt::Debug + 'static,
        V: Serialize + Send + Sync + Clone + 'static;

    /// Delete a key from the cache
    async fn delete<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Delete multiple keys from the cache
    async fn delete_many<K>(&self, keys: Vec<K>) -> CacheResult<usize>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Check if a key exists in the cache
    async fn exists<K>(&self, key: K) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Increment a counter in the cache
    async fn increment<K>(&self, key: K, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Set an expiration time for a key
    async fn expire<K>(&self, key: K, ttl: Duration) -> CacheResult<bool>
    where
        K: CacheKey + std::fmt::Debug + 'static;

    /// Clear the entire cache
    async fn clear(&self) -> CacheResult<()>;

    /// Get the health status of the cache
    async fn health_check(&self) -> CacheResult<()>;
}
```

## Proposed Extension Traits
The remaining operations would be moved to extension traits:

1. **ListOperations** - List-based operations
2. **HashOperations** - Hash map operations
3. **SetOperations** - Set operations
4. **SortedSetOperations** - Sorted set operations

## Backward Compatibility Considerations

For backward compatibility, we can:

1. Create a `LegacyCacheOperations` trait that inherits from all traits
2. Implement adapter patterns to convert between interfaces
3. Provide a bridge implementation that delegates to a BasicCache + extensions

## Next Steps

1. Review this analysis with the team
2. Finalize the BasicCache trait design
3. Create initial extension trait designs
4. Design adapter patterns for migration
5. Update the redis-rs plugin implementation to use this approach

*Updated by: Development Team*  
*April 4, 2024* 