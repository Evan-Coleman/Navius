use async_trait::async_trait;
use redis::AsyncCommands;
use std::time::Duration;

use navius_cache::{
    invalidation::{CacheInvalidator, CacheInvalidatorError, CacheInvalidatorResult},
    operations::Cache,
};

use crate::{
    config::RedisCacheConfig,
    connection::RedisConnectionManager,
    error::{RedisCacheError, RedisCacheResult},
    operations::RedisCache,
};

/// Redis cache invalidator implementation
#[derive(Clone)]
pub struct RedisInvalidator {
    /// Redis cache
    cache: RedisCache,
}

impl RedisInvalidator {
    /// Create a new Redis invalidator
    pub async fn new(config: RedisCacheConfig) -> RedisCacheResult<Self> {
        let cache = RedisCache::new(config).await?;

        Ok(Self { cache })
    }

    /// Get the underlying cache
    pub fn cache(&self) -> &RedisCache {
        &self.cache
    }

    /// Get the connection manager
    pub fn connection_manager(&self) -> &RedisConnectionManager {
        self.cache.connection_manager()
    }

    /// Create an invalidation tag key
    fn tag_key(&self, tag: &str) -> String {
        format!("tag:{}", tag)
    }

    /// Create a key-to-tags mapping key
    fn key_tags_mapping(&self, key: &str) -> String {
        format!("key_tags:{}", key)
    }
}

#[async_trait]
impl CacheInvalidator for RedisInvalidator {
    async fn tag(&self, key: &str, tags: &[&str]) -> CacheInvalidatorResult<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let key_mapping = self.key_tags_mapping(key);

        // First, store the tags associated with this key
        self.connection_manager()
            .execute_command(&key_mapping, "SADD", |mut conn| {
                let mut cmd = redis::cmd("SADD");
                cmd.arg(&key_mapping);

                for tag in tags {
                    cmd.arg(tag);
                }

                cmd.query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!(
                    "Failed to store key-to-tags mapping: {}",
                    e
                ))
            })?;

        // Then, for each tag, store the key
        for tag in tags {
            let tag_key = self.tag_key(tag);

            self.connection_manager()
                .execute_command(&tag_key, "SADD", |mut conn| {
                    redis::cmd("SADD").arg(&tag_key).arg(key).query(&mut conn)
                })
                .await
                .map_err(|e| {
                    CacheInvalidatorError::Provider(format!(
                        "Failed to store tag-to-keys mapping: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    async fn untag(&self, key: &str, tags: &[&str]) -> CacheInvalidatorResult<()> {
        if tags.is_empty() {
            return Ok(());
        }

        let key_mapping = self.key_tags_mapping(key);

        // Remove the specified tags from the key's mapping
        self.connection_manager()
            .execute_command(&key_mapping, "SREM", |mut conn| {
                let mut cmd = redis::cmd("SREM");
                cmd.arg(&key_mapping);

                for tag in tags {
                    cmd.arg(tag);
                }

                cmd.query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!(
                    "Failed to remove tags from key mapping: {}",
                    e
                ))
            })?;

        // For each tag, remove the key
        for tag in tags {
            let tag_key = self.tag_key(tag);

            self.connection_manager()
                .execute_command(&tag_key, "SREM", |mut conn| {
                    redis::cmd("SREM").arg(&tag_key).arg(key).query(&mut conn)
                })
                .await
                .map_err(|e| {
                    CacheInvalidatorError::Provider(format!(
                        "Failed to remove key from tag mapping: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    async fn invalidate_tags(&self, tags: &[&str]) -> CacheInvalidatorResult<u64> {
        if tags.is_empty() {
            return Ok(0);
        }

        let mut total_invalidated = 0;

        // For each tag, get all keys and delete them
        for tag in tags {
            let tag_key = self.tag_key(tag);

            // Get all keys for this tag
            let keys: Vec<String> = self
                .connection_manager()
                .execute_command(&tag_key, "SMEMBERS", |mut conn| {
                    redis::cmd("SMEMBERS").arg(&tag_key).query(&mut conn)
                })
                .await
                .map_err(|e| {
                    CacheInvalidatorError::Provider(format!("Failed to get keys for tag: {}", e))
                })?;

            // Delete all the keys
            for key in &keys {
                if let Ok(true) = self.cache.delete(key).await {
                    total_invalidated += 1;
                }

                // Also remove the key-tags mapping
                let key_mapping = self.key_tags_mapping(key);
                let _: Result<(), _> = self
                    .connection_manager()
                    .execute_command(&key_mapping, "DEL", |mut conn| {
                        redis::cmd("DEL").arg(&key_mapping).query(&mut conn)
                    })
                    .await;
            }

            // Remove the tag itself
            let _: Result<(), _> = self
                .connection_manager()
                .execute_command(&tag_key, "DEL", |mut conn| {
                    redis::cmd("DEL").arg(&tag_key).query(&mut conn)
                })
                .await;
        }

        Ok(total_invalidated)
    }

    async fn get_tags(&self, key: &str) -> CacheInvalidatorResult<Vec<String>> {
        let key_mapping = self.key_tags_mapping(key);

        let tags: Vec<String> = self
            .connection_manager()
            .execute_command(&key_mapping, "SMEMBERS", |mut conn| {
                redis::cmd("SMEMBERS").arg(&key_mapping).query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!("Failed to get tags for key: {}", e))
            })?;

        Ok(tags)
    }

    async fn get_keys_by_tag(&self, tag: &str) -> CacheInvalidatorResult<Vec<String>> {
        let tag_key = self.tag_key(tag);

        let keys: Vec<String> = self
            .connection_manager()
            .execute_command(&tag_key, "SMEMBERS", |mut conn| {
                redis::cmd("SMEMBERS").arg(&tag_key).query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!("Failed to get keys for tag: {}", e))
            })?;

        Ok(keys)
    }

    async fn invalidate_pattern(&self, pattern: &str) -> CacheInvalidatorResult<u64> {
        let prefixed_pattern = self.connection_manager().prefixed_key(pattern);

        // Find all keys matching the pattern
        let keys: Vec<String> = self
            .connection_manager()
            .execute_command(&prefixed_pattern, "KEYS", |mut conn| {
                redis::cmd("KEYS").arg(&prefixed_pattern).query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!("Failed to find keys by pattern: {}", e))
            })?;

        if keys.is_empty() {
            return Ok(0);
        }

        let mut invalidated = 0;

        // Delete all the keys
        let result: i64 = self
            .connection_manager()
            .execute_command("multiple", "DEL", |mut conn| {
                redis::cmd("DEL").arg(&keys).query(&mut conn)
            })
            .await
            .map_err(|e| {
                CacheInvalidatorError::Provider(format!("Failed to delete keys: {}", e))
            })?;

        invalidated = result as u64;

        // Also clean up tag mappings for these keys
        for key in keys {
            // Extract the original key without prefix
            let unprefixed_key = if let Some(stripped) =
                key.strip_prefix(&self.connection_manager().config().key_prefix)
            {
                stripped.to_string()
            } else {
                continue;
            };

            // Get tags for this key
            if let Ok(tags) = self.get_tags(&unprefixed_key).await {
                // For each tag, remove this key
                for tag in tags {
                    let tag_key = self.tag_key(&tag);

                    let _: Result<(), _> = self
                        .connection_manager()
                        .execute_command(&tag_key, "SREM", |mut conn| {
                            redis::cmd("SREM")
                                .arg(&tag_key)
                                .arg(&unprefixed_key)
                                .query(&mut conn)
                        })
                        .await;
                }
            }

            // Remove the key-tags mapping
            let key_mapping = self.key_tags_mapping(&unprefixed_key);
            let _: Result<(), _> = self
                .connection_manager()
                .execute_command(&key_mapping, "DEL", |mut conn| {
                    redis::cmd("DEL").arg(&key_mapping).query(&mut conn)
                })
                .await;
        }

        Ok(invalidated)
    }

    async fn clear(&self) -> CacheInvalidatorResult<()> {
        // Clear all cache data
        self.cache.clear().await.map_err(|e| {
            CacheInvalidatorError::Provider(format!("Failed to clear cache: {}", e))
        })?;

        // Also clear all tag mappings
        self.invalidate_pattern("tag:*").await?;
        self.invalidate_pattern("key_tags:*").await?;

        Ok(())
    }

    async fn health_check(&self) -> CacheInvalidatorResult<bool> {
        self.cache
            .health_check()
            .await
            .map_err(|e| CacheInvalidatorError::Provider(format!("Health check failed: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navius_cache::invalidation::CacheInvalidator;
    use std::time::Duration;

    #[tokio::test]
    async fn test_tag_invalidation() {
        let config = RedisCacheConfig::new(
            "redis://localhost:6379".to_string(),
            "test:".to_string(),
            Duration::from_secs(3600),
        );

        // Skip test if Redis is not available
        let invalidator = RedisInvalidator::new(config).await;
        if invalidator.is_err() {
            println!("Skipping test_tag_invalidation - Redis not available");
            return;
        }

        let invalidator = invalidator.unwrap();
        let cache = invalidator.cache();

        // Clear any existing data
        let _ = invalidator.clear().await;

        // Set up test data
        let key1 = "user:1";
        let key2 = "user:2";
        let key3 = "product:1";

        cache.set(key1, &"User One", None).await.unwrap();
        cache.set(key2, &"User Two", None).await.unwrap();
        cache.set(key3, &"Product One", None).await.unwrap();

        // Tag the keys
        invalidator.tag(key1, &["user", "active"]).await.unwrap();
        invalidator.tag(key2, &["user", "inactive"]).await.unwrap();
        invalidator.tag(key3, &["product"]).await.unwrap();

        // Check that the tags were stored correctly
        let key1_tags = invalidator.get_tags(key1).await.unwrap();
        assert!(key1_tags.contains(&"user".to_string()));
        assert!(key1_tags.contains(&"active".to_string()));

        // Get keys by tag
        let user_keys = invalidator.get_keys_by_tag("user").await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&key1.to_string()));
        assert!(user_keys.contains(&key2.to_string()));

        // Invalidate by tag
        let invalidated = invalidator.invalidate_tags(&["user"]).await.unwrap();
        assert_eq!(invalidated, 2);

        // Verify keys are gone
        assert!(cache.get::<String>(key1).await.unwrap().is_none());
        assert!(cache.get::<String>(key2).await.unwrap().is_none());

        // But product should still be there
        assert!(cache.get::<String>(key3).await.unwrap().is_some());

        // Invalidate by pattern
        let invalidated = invalidator.invalidate_pattern("product:*").await.unwrap();
        assert_eq!(invalidated, 1);

        // Verify product is gone
        assert!(cache.get::<String>(key3).await.unwrap().is_none());
    }
}
