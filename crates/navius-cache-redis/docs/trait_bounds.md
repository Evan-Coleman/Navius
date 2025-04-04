# Understanding Trait Bounds in Redis Cache Implementation

This document explains the trait bounds used in the Redis cache implementation and why they're required.

## Common Trait Bounds

### For Key Types (K)

Most methods use the following bounds for key types:

```rust
K: CacheKey + std::fmt::Debug + 'static
```

- **`CacheKey`**: Ensures the type can be used as a cache key (typically convertible to a string)
- **`std::fmt::Debug`**: Enables better error reporting by allowing keys to be printed in debug logs
- **`'static`**: Ensures the type can live for the entire program duration, which is necessary for async operations

### For Value Types (V)

#### For methods that store values:

```rust
V: Serialize + Send + Sync + Clone + 'static
```

- **`Serialize`**: Allows the value to be serialized for storage in Redis
- **`Send + Sync`**: Ensures the type can be safely shared and sent between threads in an async context
- **`Clone`**: Allows the value to be cloned when needed (e.g., for retries or multiple operations)
- **`'static`**: Ensures the type can live for the entire program duration, which is necessary for async operations

#### For methods that retrieve values:

```rust
V: DeserializeOwned + Send + 'static
```

- **`DeserializeOwned`**: Allows the value to be deserialized from Redis data without borrowing
- **`Send`**: Ensures the type can be safely sent between threads in an async context
- **`'static`**: Ensures the type can live for the entire program duration, which is necessary for async operations

### For Field Types (F) in Hash Operations

```rust
F: CacheKey + Serialize + Clone + std::fmt::Debug + 'static
```

- **`CacheKey`**: Ensures the type can be used as a hash field key
- **`Serialize`**: Allows the field to be serialized for storage in Redis
- **`Clone`**: Allows the field to be cloned when needed
- **`std::fmt::Debug`**: Enables better error reporting
- **`'static`**: Ensures the type can live for the entire program duration

## Special Cases

### Sorted Set (ZSet) Operations

Some zset operations combine or modify the common trait bounds:

1. **zset_add** and **zset_remove**:
   ```rust
   K: CacheKey + 'static
   V: Serialize + Send + Sync + 'static
   ```

2. **zset_score** and **zset_rank**:
   ```rust
   K: CacheKey + 'static
   V: Serialize + Send + Sync + 'static
   ```

3. **zset_range** and **zset_range_by_score**:
   ```rust
   K: CacheKey + 'static
   V: DeserializeOwned + Send + 'static
   ```

## Why These Bounds Are Necessary

1. **Asynchronous Context**: Redis operations run asynchronously, requiring types to be sendable between threads (`Send`) and in some cases shareable among threads (`Sync`).

2. **Serialization**: Redis stores all data as strings or binary data, requiring serialization and deserialization.

3. **Safe Error Handling**: Debug bounds allow for better error messages and debugging.

4. **Ownership**: The `'static` lifetime ensures values aren't borrowed during async operations, which could lead to use-after-free errors.

5. **Redis-Specific Requirements**: Some operations, like hash operations, have additional requirements for their field types.

## Examples

### Simple Example

```rust
use navius_cache::{Cache, CacheOperations};
use navius_cache_redis::{new, RedisCacheConfig};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct User {
    id: u64,
    username: String,
}

async fn store_and_retrieve_user() {
    let config = RedisCacheConfig::new("redis://localhost:6379");
    let cache = new(config).await.unwrap();
    
    let user = User { id: 1, username: "alice".to_string() };
    
    // Store user in cache
    cache.set("user:1", &user, None).await.unwrap();
    
    // Retrieve user from cache
    let retrieved_user: Option<User> = cache.get("user:1").await.unwrap();
    
    assert_eq!(retrieved_user.unwrap().username, "alice");
}
```

### Sorted Set Example

```rust
use navius_cache::{Cache, CacheOperations};
use navius_cache_redis::{new, RedisCacheConfig};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct Player {
    id: u64,
    name: String,
}

async fn leaderboard_example() {
    let config = RedisCacheConfig::new("redis://localhost:6379");
    let cache = new(config).await.unwrap();
    
    // Add players to leaderboard
    let players = vec![
        (100.0, Player { id: 1, name: "Alice".to_string() }),
        (200.0, Player { id: 2, name: "Bob".to_string() }),
        (150.0, Player { id: 3, name: "Charlie".to_string() }),
    ];
    
    cache.zset_add("leaderboard", players).await.unwrap();
    
    // Get top players
    let top_players: Vec<Player> = cache.zset_range("leaderboard", 0, 1).await.unwrap();
    assert_eq!(top_players.len(), 2);
    
    // Get players by score range
    let mid_tier: Vec<Player> = cache.zset_range_by_score("leaderboard", 120.0, 180.0).await.unwrap();
    assert_eq!(mid_tier.len(), 1);
    assert_eq!(mid_tier[0].name, "Charlie");
    
    // Count players in score range
    let count = cache.zset_count("leaderboard", 0.0, 150.0).await.unwrap();
    assert_eq!(count, 2); // Alice and Charlie
}
```

## Troubleshooting

### Common Errors

1. **"The trait bound `T: Send` is not satisfied"**:
   - Ensure your type implements `Send` (most types do)
   - If using a custom type containing raw pointers or non-Send types, consider redesigning

2. **"The trait bound `T: Sync` is not satisfied"**:
   - Ensure your type implements `Sync` (most immutable data is Sync)
   - If using interior mutability like `RefCell`, consider using `Mutex` instead

3. **"The trait bound `T: Clone` is not satisfied"**:
   - Add `#[derive(Clone)]` to your type
   - Or implement Clone manually

4. **"The trait bound `T: Serialize` is not satisfied"**:
   - Add `#[derive(Serialize)]` to your type and import `serde::Serialize`
   - Add `serde` and `serde_derive` to your dependencies

5. **"The trait bound `T: DeserializeOwned` is not satisfied"**:
   - Add `#[derive(Deserialize)]` to your type and import `serde::Deserialize`
   - Ensure your type doesn't contain borrowed references 