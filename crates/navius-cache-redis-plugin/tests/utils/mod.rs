pub mod mock_redis;
/// Testing utilities for the Redis cache plugin
///
/// This module provides utilities to make testing Redis caching functionality easier.
pub mod test_redis;

// Re-export common testing components for more convenient imports
pub use mock_redis::MockRedis;
pub use test_redis::{RedisTestContext, TestRedisServer};
