use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use navius_cache_redis::{
    RedisCache, RedisCacheConfig, RedisLuaManager, initialize_common_scripts,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestUser {
    id: i32,
    name: String,
    email: String,
    created_at: u64,
    is_active: bool,
    data: Vec<String>,
}

impl TestUser {
    fn new(id: i32) -> Self {
        Self {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            is_active: true,
            data: vec![
                "Data 1".to_string(),
                "Data 2".to_string(),
                "Data 3".to_string(),
            ],
        }
    }
}

fn create_test_cache() -> (RedisCache, Runtime) {
    let rt = Runtime::new().unwrap();

    // Configure Redis connection
    let config = RedisCacheConfig::builder()
        .with_url("redis://localhost:6379")
        .with_connection_timeout(Duration::from_secs(5))
        .with_pool_size(10)
        .build();

    // Create cache
    let cache = rt.block_on(async {
        let cache = RedisCache::new(config).await.unwrap();

        // Flush the database before benchmarking
        let mut conn = cache.connection().await.unwrap();
        let _: () = redis::cmd("FLUSHDB").query_async(&mut conn).await.unwrap();

        cache
    });

    (cache, rt)
}

fn benchmark_basic_operations(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Basic Operations");

    // SET operation - string
    group.bench_function("SET string", |b| {
        let key = "bench_set_string";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache.set(key, value, None).await.unwrap();
            })
        })
    });

    // GET operation - string
    group.bench_function("GET string", |b| {
        let key = "bench_get_string";
        let value = "benchmark_value";

        rt.block_on(async {
            cache.set(key, value, None).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _: String = cache.get(key).await.unwrap();
            })
        })
    });

    // SET operation - with expiration
    group.bench_function("SET with expiration", |b| {
        let key = "bench_set_ttl";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache
                    .set(key, value, Some(Duration::from_secs(300)))
                    .await
                    .unwrap();
            })
        })
    });

    // DELETE operation
    group.bench_function("DELETE", |b| {
        let key = "bench_delete";
        let value = "benchmark_value";

        b.iter_with_setup(
            || {
                rt.block_on(async {
                    cache.set(key, value, None).await.unwrap();
                });
            },
            |_| {
                rt.block_on(async {
                    cache.delete(key).await.unwrap();
                })
            },
        )
    });

    // EXISTS operation
    group.bench_function("EXISTS", |b| {
        let key = "bench_exists";
        let value = "benchmark_value";

        rt.block_on(async {
            cache.set(key, value, None).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _ = cache.exists(key).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Serialization");

    // SET operation - small object
    group.bench_function("SET small object", |b| {
        let key = "bench_set_small_obj";
        let value = TestUser::new(1);

        b.iter(|| {
            rt.block_on(async {
                cache.set(key, &value, None).await.unwrap();
            })
        })
    });

    // GET operation - small object
    group.bench_function("GET small object", |b| {
        let key = "bench_get_small_obj";
        let value = TestUser::new(2);

        rt.block_on(async {
            cache.set(key, &value, None).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _: TestUser = cache.get(key).await.unwrap();
            })
        })
    });

    // SET operation - collection of objects
    group.bench_function("SET object collection", |b| {
        let key = "bench_set_collection";
        let value: Vec<TestUser> = (0..10).map(TestUser::new).collect();

        b.iter(|| {
            rt.block_on(async {
                cache.set(key, &value, None).await.unwrap();
            })
        })
    });

    // GET operation - collection of objects
    group.bench_function("GET object collection", |b| {
        let key = "bench_get_collection";
        let value: Vec<TestUser> = (0..10).map(TestUser::new).collect();

        rt.block_on(async {
            cache.set(key, &value, None).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _: Vec<TestUser> = cache.get(key).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_list_operations(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("List Operations");

    // LPUSH operation
    group.bench_function("LPUSH", |b| {
        let key = "bench_lpush";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache.lpush(key, value).await.unwrap();
            })
        })
    });

    // RPUSH operation
    group.bench_function("RPUSH", |b| {
        let key = "bench_rpush";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache.rpush(key, value).await.unwrap();
            })
        })
    });

    // LPOP operation
    group.bench_function("LPOP", |b| {
        let key = "bench_lpop";

        b.iter_with_setup(
            || {
                rt.block_on(async {
                    cache.lpush(key, "benchmark_value").await.unwrap();
                });
            },
            |_| {
                rt.block_on(async {
                    let _: String = cache.lpop(key).await.unwrap();
                })
            },
        )
    });

    // LRANGE operation
    group.bench_function("LRANGE", |b| {
        let key = "bench_lrange";

        rt.block_on(async {
            // Clear the list first
            let _ = cache.delete(key).await;

            // Push 100 items to the list
            for i in 0..100 {
                cache.rpush(key, format!("value_{}", i)).await.unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.lrange(key, 0, 50).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_hash_operations(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Hash Operations");

    // HSET operation
    group.bench_function("HSET", |b| {
        let key = "bench_hset";
        let field = "field";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache.hset(key, field, value).await.unwrap();
            })
        })
    });

    // HGET operation
    group.bench_function("HGET", |b| {
        let key = "bench_hget";
        let field = "field";
        let value = "benchmark_value";

        rt.block_on(async {
            cache.hset(key, field, value).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _: String = cache.hget(key, field).await.unwrap();
            })
        })
    });

    // HGETALL operation
    group.bench_function("HGETALL", |b| {
        let key = "bench_hgetall";

        rt.block_on(async {
            // Clear the hash first
            let _ = cache.delete(key).await;

            // Set 10 fields in the hash
            for i in 0..10 {
                cache
                    .hset(key, format!("field_{}", i), format!("value_{}", i))
                    .await
                    .unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let _: std::collections::HashMap<String, String> =
                    cache.hgetall(key).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_set_operations(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Set Operations");

    // SADD operation
    group.bench_function("SADD", |b| {
        let key = "bench_sadd";
        let value = "benchmark_value";

        b.iter(|| {
            rt.block_on(async {
                cache.sadd(key, value).await.unwrap();
            })
        })
    });

    // SISMEMBER operation
    group.bench_function("SISMEMBER", |b| {
        let key = "bench_sismember";
        let value = "benchmark_value";

        rt.block_on(async {
            cache.sadd(key, value).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let _ = cache.sismember(key, value).await.unwrap();
            })
        })
    });

    // SMEMBERS operation
    group.bench_function("SMEMBERS", |b| {
        let key = "bench_smembers";

        rt.block_on(async {
            // Clear the set first
            let _ = cache.delete(key).await;

            // Add 100 members to the set
            for i in 0..100 {
                cache.sadd(key, format!("value_{}", i)).await.unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.smembers(key).await.unwrap();
            })
        })
    });

    // Set operations with large sets
    let set1 = "bench_set1";
    let set2 = "bench_set2";

    rt.block_on(async {
        // Clear the sets first
        let _ = cache.delete(set1).await;
        let _ = cache.delete(set2).await;

        // Add 1000 members to each set with 500 common members
        for i in 0..1000 {
            cache.sadd(set1, format!("value_{}", i)).await.unwrap();
        }

        for i in 500..1500 {
            cache.sadd(set2, format!("value_{}", i)).await.unwrap();
        }
    });

    // SINTER operation
    group.bench_function("SINTER", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.sinter(&[set1, set2]).await.unwrap();
            })
        })
    });

    // SUNION operation
    group.bench_function("SUNION", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.sunion(&[set1, set2]).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_sorted_set_operations(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Sorted Set Operations");

    // ZADD operation
    group.bench_function("ZADD", |b| {
        let key = "bench_zadd";
        let value = "benchmark_value";
        let score = 1.0;

        b.iter(|| {
            rt.block_on(async {
                cache.zadd(key, value, score).await.unwrap();
            })
        })
    });

    // ZRANGE operation
    group.bench_function("ZRANGE", |b| {
        let key = "bench_zrange";

        rt.block_on(async {
            // Clear the sorted set first
            let _ = cache.delete(key).await;

            // Add 100 members to the sorted set
            for i in 0..100 {
                cache
                    .zadd(key, format!("value_{}", i), i as f64)
                    .await
                    .unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.zrange(key, 0, 50).await.unwrap();
            })
        })
    });

    // ZRANGEBYSCORE operation
    group.bench_function("ZRANGEBYSCORE", |b| {
        let key = "bench_zrangebyscore";

        rt.block_on(async {
            // Clear the sorted set first
            let _ = cache.delete(key).await;

            // Add 100 members to the sorted set
            for i in 0..100 {
                cache
                    .zadd(key, format!("value_{}", i), i as f64)
                    .await
                    .unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let _: Vec<String> = cache.zrangebyscore(key, 25.0, 75.0).await.unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_lua_scripting(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let lua_manager = rt.block_on(async {
        let manager = RedisLuaManager::new(cache.clone_inner()).await.unwrap();
        initialize_common_scripts(&manager).await.unwrap();
        manager
    });

    let mut group = c.benchmark_group("Lua Scripting");

    // Simple Lua script
    let simple_script = r#"
    redis.call('SET', KEYS[1], ARGV[1])
    return redis.call('GET', KEYS[1])
    "#;

    let simple_script_hash =
        rt.block_on(async { lua_manager.load_script(simple_script).await.unwrap() });

    group.bench_function("Simple Lua script", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: String = lua_manager
                    .execute_script(
                        &simple_script_hash,
                        &["bench_lua_simple"],
                        &["benchmark_value"],
                    )
                    .await
                    .unwrap();
            })
        })
    });

    // Complex Lua script (increment and expire)
    group.bench_function("Increment and expire script", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _: i64 = lua_manager
                    .execute_script(
                        "increment_and_expire",
                        &["bench_lua_increment"],
                        &["1", "300"],
                    )
                    .await
                    .unwrap();
            })
        })
    });

    group.finish();
}

fn benchmark_pipelining(c: &mut Criterion) {
    let (cache, rt) = create_test_cache();

    let mut group = c.benchmark_group("Pipelining");

    for &num_operations in &[1, 5, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("Pipeline operations", num_operations),
            &num_operations,
            |b, &num_ops| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut pipeline = cache.pipeline();

                        for i in 0..num_ops {
                            pipeline.set(
                                format!("bench_pipeline_{}", i),
                                format!("value_{}", i),
                                None,
                            );
                        }

                        pipeline.execute().await.unwrap();
                    })
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Individual operations", num_operations),
            &num_operations,
            |b, &num_ops| {
                b.iter(|| {
                    rt.block_on(async {
                        for i in 0..num_ops {
                            cache
                                .set(
                                    &format!("bench_individual_{}", i),
                                    &format!("value_{}", i),
                                    None,
                                )
                                .await
                                .unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

fn benchmark_connection_pool(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("Connection Pool");

    for &pool_size in &[1, 5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::new("Concurrent operations", pool_size),
            &pool_size,
            |b, &size| {
                // Create a cache with the specified pool size
                let config = RedisCacheConfig::builder()
                    .with_url("redis://localhost:6379")
                    .with_connection_timeout(Duration::from_secs(5))
                    .with_pool_size(size)
                    .build();

                let cache = rt.block_on(async { RedisCache::new(config).await.unwrap() });

                b.iter(|| {
                    rt.block_on(async {
                        let cache_arc = Arc::new(cache.clone());

                        let mut handles = vec![];
                        for i in 0..size {
                            let cache_clone = cache_arc.clone();
                            handles.push(tokio::spawn(async move {
                                cache_clone
                                    .set(
                                        &format!("bench_concurrent_{}", i),
                                        &format!("value_{}", i),
                                        None,
                                    )
                                    .await
                                    .unwrap();

                                let _: String = cache_clone
                                    .get(&format!("bench_concurrent_{}", i))
                                    .await
                                    .unwrap();
                            }));
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10));
    targets =
        benchmark_basic_operations,
        benchmark_serialization,
        benchmark_list_operations,
        benchmark_hash_operations,
        benchmark_set_operations,
        benchmark_sorted_set_operations,
        benchmark_lua_scripting,
        benchmark_pipelining,
        benchmark_connection_pool
);
criterion_main!(benches);
