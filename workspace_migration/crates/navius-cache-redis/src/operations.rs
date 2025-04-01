impl<K> RedisStringOperations<K> for RedisCache
where
    K: CacheKey + std::fmt::Debug + 'static,
{
    // ... existing implementation ...
}

impl<K> RedisListOperations<K> for RedisCache 
where
    K: CacheKey + std::fmt::Debug + 'static,
{
    // ... existing implementation ...
}

impl<K, F> RedisHash<K, F> for RedisCache
where
    K: CacheKey + std::fmt::Debug + 'static,
    F: CacheKey + std::fmt::Debug + 'static,
{
    #[instrument(skip(self), level = "debug")]
    async fn hexists(&self, key: &K, field: &F) -> Result<bool, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        let field_str = self.key_to_string(field)?;
        
        let exists = self.with_connection(|mut conn| async move {
            let result = redis::cmd("HEXISTS")
                .arg(&prefixed_key)
                .arg(&field_str)
                .query_async(&mut conn)
                .await?;
            Ok(result)
        }).await?;
        
        Ok(exists)
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hset_multiple<V: CacheKey + Serialize>(
        &self,
        key: &K,
        fields: &[(F, V)],
    ) -> Result<(), RedisCacheError> {
        // ... existing code ...
    }

    #[instrument(skip(self), level = "debug")]
    async fn hgetall<V: DeserializeOwned>(&self, key: &K) -> Result<HashMap<String, V>, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let result: HashMap<String, String> = self.with_connection(|mut conn| async move {
            Ok(redis::cmd("HGETALL")
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await?)
        }).await?;
        
        // ... existing code ...
    }

    #[instrument(skip(self), level = "debug")]
    async fn hkeys(&self, key: &K) -> Result<Vec<String>, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let keys: Vec<String> = self.with_connection(|mut conn| async move {
            Ok(redis::cmd("HKEYS")
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await?)
        }).await?;
        
        Ok(keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hvals<V: DeserializeOwned>(&self, key: &K) -> Result<Vec<V>, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let values: Vec<String> = self.with_connection(|mut conn| async move {
            Ok(redis::cmd("HVALS")
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await?)
        }).await?;
        
        // ... existing code ...
    }

    #[instrument(skip(self), level = "debug")]
    async fn hincrby(&self, key: &K, field: &F, amount: i64) -> Result<i64, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        let field_str = self.key_to_string(field)?;
        
        let result = self.with_connection(|mut conn| async move {
            Ok(redis::cmd("HINCRBY")
                .arg(&prefixed_key)
                .arg(&field_str)
                .arg(amount)
                .query_async(&mut conn)
                .await?)
        }).await?;
        
        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hlen(&self, key: &K) -> Result<usize, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let len = self.with_connection(|mut conn| async move {
            Ok(redis::cmd("HLEN").arg(&prefixed_key).query_async(&mut conn).await?)
        }).await?;
        
        Ok(len)
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hdel(&self, key: &K, fields: &[F]) -> Result<usize, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        // Build command with all field names
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(&prefixed_key);
        
        for field in fields {
            let field_str = self.key_to_string(field)?;
            cmd.arg(field_str);
        }
        
        // Execute with proper await
        self.with_connection(|mut conn| async move {
            let result = cmd.query_async(&mut conn).await?;
            Ok(result)
        }).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_exists<K, F>(&self, key: K, field: F) -> CacheResult<bool>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: bool = self
            .connection_manager
            .execute_command(&prefixed_key, "HEXISTS", |mut conn| {
                redis::cmd("HEXISTS")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self, fields), level = "debug")]
    async fn hash_delete<K, F>(&self, key: K, fields: Vec<F>) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        if fields.is_empty() {
            return Ok(0);
        }

        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Convert fields to strings
        let field_strings: Vec<String> = fields.into_iter().map(|f| f.to_string()).collect();

        // Use HDEL to delete multiple fields at once
        let mut cmd = redis::cmd("HDEL");
        cmd.arg(&prefixed_key);
        for field in &field_strings {
            cmd.arg(field);
        }

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HDEL", |mut conn| cmd.query(&mut conn))
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_get_all<K, V>(&self, key: K) -> CacheResult<Vec<(String, V)>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        // Get all fields and values as flattened array of [field1, val1, field2, val2, ...]
        let result: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HGETALL", |mut conn| {
                redis::cmd("HGETALL")
                    .arg(&prefixed_key)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Process results into pairs
        let mut pairs = Vec::new();
        let mut iter = result.into_iter();

        while let (Some(field_bytes), Some(value_bytes)) = (iter.next(), iter.next()) {
            let field = String::from_utf8(field_bytes)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            match self.deserialize(&value_bytes).await {
                Ok(value) => pairs.push((field, value)),
                Err(e) => return Err(e),
            }
        }

        Ok(pairs)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_keys<K>(&self, key: K) -> CacheResult<Vec<String>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let keys: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HKEYS", |mut conn| {
                redis::cmd("HKEYS")
                    .arg(&prefixed_key)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Convert bytes to strings
        let string_keys = keys
            .into_iter()
            .map(|k| {
                String::from_utf8(k).map_err(|e| CacheError::SerializationError(e.to_string()))
            })
            .collect::<Result<Vec<String>, CacheError>>()?;

        Ok(string_keys)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_values<K, V>(&self, key: K) -> CacheResult<Vec<V>>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        V: DeserializeOwned + 'static,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let values_bytes: Vec<Vec<u8>> = self
            .connection_manager
            .execute_command(&prefixed_key, "HVALS", |mut conn| {
                redis::cmd("HVALS")
                    .arg(&prefixed_key)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        // Deserialize each value
        let mut values = Vec::with_capacity(values_bytes.len());
        for value_bytes in values_bytes {
            match self.deserialize(&value_bytes).await {
                Ok(value) => values.push(value),
                Err(e) => return Err(e),
            }
        }

        Ok(values)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_increment<K, F>(&self, key: K, field: F, amount: i64) -> CacheResult<i64>
    where
        K: CacheKey + 'static + std::fmt::Debug,
        F: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let field_str = field.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let result: i64 = self
            .connection_manager
            .execute_command(&prefixed_key, "HINCRBY", |mut conn| {
                redis::cmd("HINCRBY")
                    .arg(&prefixed_key)
                    .arg(&field_str)
                    .arg(amount)
                    .query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn hash_length<K>(&self, key: K) -> CacheResult<usize>
    where
        K: CacheKey + 'static + std::fmt::Debug,
    {
        let key_str = key.to_string();
        let prefixed_key = self.connection_manager.prefixed_key(&key_str);

        let count: usize = self
            .connection_manager
            .execute_command(&prefixed_key, "HLEN", |mut conn| {
                redis::cmd("HLEN").arg(&prefixed_key).query(&mut conn)
            })
            .await
            .map_err(|e| CacheError::from(e))?;

        Ok(count)
    }
}

// Fix the same pattern for other Redis operations like set operations
impl<K> RedisSet<K> for RedisCache
where
    K: CacheKey + std::fmt::Debug + 'static,
{
    #[instrument(skip(self, values), level = "debug")]
    async fn srem<V: CacheKey + Serialize + std::fmt::Debug>(&self, key: &K, values: &[V]) -> Result<usize, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        // Build command with all serialized values
        let mut cmd = redis::cmd("SREM");
        cmd.arg(&prefixed_key);
        
        for value in values {
            let serialized = self.serialize(value)?;
            cmd.arg(serialized);
        }
        
        self.with_connection(|mut conn| async move {
            let result = cmd.query_async(&mut conn).await?;
            Ok(result)
        }).await
    }

    #[instrument(skip(self), level = "debug")]
    async fn sismember<V: CacheKey + Serialize + std::fmt::Debug>(&self, key: &K, value: &V) -> Result<bool, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        let serialized = self.serialize(value)?;
        
        let is_member = self.with_connection(|mut conn| async move {
            let result = redis::cmd("SISMEMBER")
                .arg(&prefixed_key)
                .arg(serialized)
                .query_async(&mut conn)
                .await?;
            Ok(result)
        }).await?;
        
        Ok(is_member)
    }

    #[instrument(skip(self), level = "debug")]
    async fn smembers<V: DeserializeOwned + std::fmt::Debug>(&self, key: &K) -> Result<HashSet<V>, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let values: Vec<String> = self.with_connection(|mut conn| async move {
            let result = redis::cmd("SMEMBERS")
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await?;
            Ok(result)
        }).await?;
        
        // Deserialize values
        let mut result = HashSet::new();
        for value in values {
            match self.deserialize::<V>(&value) {
                Ok(v) => { result.insert(v); },
                Err(e) => return Err(e),
            }
        }
        
        Ok(result)
    }

    #[instrument(skip(self), level = "debug")]
    async fn scard(&self, key: &K) -> Result<usize, RedisCacheError> {
        let prefixed_key = self.prefix_key(key);
        
        let count = self.with_connection(|mut conn| async move {
            let result = redis::cmd("SCARD")
                .arg(&prefixed_key)
                .query_async(&mut conn)
                .await?;
            Ok(result)
        }).await?;
        
        Ok(count)
    }

    // ... existing code for other set operations ...
}

impl<K> RedisSortedSet<K> for RedisCache
where
    K: CacheKey + std::fmt::Debug + 'static,
{
    // ... existing implementation with proper awaits ...
}

// Make sure all individual methods also add awaits to all query_async calls
// and wrap in Ok() where needed 