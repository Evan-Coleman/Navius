# Navius Redis Cache

A high-performance Redis cache implementation with comprehensive metrics, connection pooling, and Lua scripting support.

## Features

- Robust connection pooling with health monitoring
- Comprehensive metrics for all operations
- Serialization/deserialization of complex objects
- Support for atomic operations via Lua scripting
- Pipeline support for batch operations
- Collection operations (lists, sets, hashes)
- Prometheus metrics integration

## Usage

### Basic Usage

```rust
use std::sync::Arc;
use std::time::Duration;
use navius_cache::{Cache, CacheError};
use navius_cache_redis::{RedisCache, RedisConnectionManager};

async fn example() -> Result<(), CacheError> {
    // Create a connection manager
    let redis_url = "redis://127.0.0.1:6379/";
    let connection_manager = Arc::new(RedisConnectionManager::new(redis_url)?);
    
    // Create a cache instance
    let cache = RedisCache::new(connection_manager);
    
    // Basic operations
    cache.set("my_key", "my_value", Some(Duration::from_secs(60))).await?;
    let value: String = cache.get("my_key").await?;
    
    // Delete a key
    cache.delete("my_key").await?;
    
    Ok(())
}
```

### Lua Scripting Support

Enable Lua scripting for atomic operations:

```rust
// Create a cache with Lua scripting enabled
let cache = RedisCache::new(connection_manager.clone())
    .with_lua_scripting();

// Initialize common scripts
cache.initialize_common_scripts().await?;

// Use atomic operations
let result = cache.set_if_not_exists("lock_key", "owner_id", Some(Duration::from_secs(10))).await?;
if result {
    // Lock acquired
    // ... do work ...
    
    // Release lock
    cache.delete("lock_key").await?;
}

// Increment a counter atomically
let new_value: i64 = cache.atomic_increment("counter", 1, None).await?;

// Check and increment (useful for rate limiting)
let can_proceed = cache.check_and_increment_counter(
    "rate_limit:user:123", 
    5,  // max allowed
    Some(Duration::from_secs(60))
).await?;

// Register and use custom scripts
let script_name = "my_custom_script";
let script_content = r#"
    return redis.call('SET', KEYS[1], ARGV[1])
"#;

cache.register_script(script_name, script_content).await?;
let result: String = cache.execute_script(
    script_name,
    vec!["key1".to_string()],
    vec!["value1".to_string()]
).await?;
```

## Metrics

The Navius Redis Cache provides comprehensive metrics for monitoring cache performance and health. These metrics are exposed via Prometheus and can be easily integrated with your monitoring infrastructure.

### Metrics Categories

1. **Operation Metrics**
   - `navius_redis_cache_operation_duration_seconds`: Histogram of operation durations
   - `navius_redis_cache_operation_errors_total`: Counter of operation errors
   - `navius_redis_cache_operation_total`: Counter of operations executed

2. **Connection Pool Metrics**
   - `navius_redis_cache_pool_size`: Current size of the connection pool
   - `navius_redis_cache_connection_acquisition_time_seconds`: Time to acquire a connection
   - `navius_redis_cache_connection_errors_total`: Counter of connection errors
   - `navius_redis_cache_connection_health`: Health status of the Redis connection (0-1)

3. **Lua Script Metrics**
   - `navius_redis_cache_script_execution_duration_seconds`: Histogram of script execution times
   - `navius_redis_cache_script_errors_total`: Counter of script execution errors

### Metrics Implementation Details

The metrics system in navius-cache-redis uses several technical approaches to provide comprehensive monitoring:

#### TimedOperation Utility

For automatic metric collection, we use a `TimedOperation` struct that handles:
- Recording operation start time
- Tracking success or failure
- Categorizing errors by type
- Recording timing in appropriate histograms

Example usage:
```rust
let timer = TimedOperation::new(metrics::names::GET);
let result = perform_operation();
timer.record(&result);
```

#### Health Tracking

Connection health is tracked using:
- A three-state model: `Healthy`, `Degraded`, or `Unhealthy`
- Detailed reason recording for degraded/unhealthy states
- Gauges that reflect current health on a 0-1 scale

#### Pool Statistics

Connection pool metrics track:
- Total connections created/closed
- Current active/idle connections
- Acquisition successes/failures/timeouts
- Connection acquisition timing

### Viewing Metrics

To view metrics in real-time, run the metrics example:

```bash
cargo run --example metrics_example
```

This will start a metrics server on port 9091. You can view the metrics at:
```
http://localhost:9091/metrics
```

### Grafana Dashboard

For visualization, import the provided Grafana dashboard JSON:

```
docs/dashboards/redis_cache_metrics.json
```

The dashboard includes:
- Operation rate and latency panels
- Error rate tracking
- Connection pool utilization
- Script execution performance
- Collection operation metrics

### Integration with Monitoring Systems

The metrics implementation is designed to work with:

1. **Prometheus**: Direct export of metrics in Prometheus format
   ```rust
   // To set up a Prometheus metrics endpoint
   use metrics_exporter_prometheus::PrometheusBuilder;
   
   let builder = PrometheusBuilder::new();
   let handle = builder.install_recorder().expect("Failed to install recorder");
   ```

2. **Custom Monitoring**: You can implement your own recorder
   ```rust
   // Register your custom metrics recorder
   metrics::set_boxed_recorder(Box::new(MyCustomRecorder::new()))?;
   ```

3. **Structured Logging**: Metrics data can be included in logs
   ```rust
   // Log current connection pool stats
   let stats = connection_manager.get_stats().await;
   tracing::info!(
       pool_size = stats.current_active_connections + stats.current_idle_connections,
       active = stats.current_active_connections,
       idle = stats.current_idle_connections,
       "Connection pool stats"
   );
   ```

### Testing Metrics

The crate includes comprehensive tests for metrics:
- Verification of metric recording for all operations
- Validation of connection pool metrics
- Tests for error metrics recording

See `tests/metrics_test.rs` for examples of how to test metrics.

## Examples

- `examples/basic_usage.rs`: Demonstrates basic cache operations
- `examples/metrics_example.rs`: Shows metrics collection and visualization
- `examples/lua_scripting.rs`: Demonstrates Lua scripting capabilities

## Running Tests

```bash
# Install redis-server if not already installed
# On MacOS: brew install redis
# On Ubuntu: apt-get install redis-server

# Start Redis server
redis-server --port 6379

# Run tests
cargo test
```

## Performance Considerations

- Use connection pooling appropriately for your workload
- Consider using pipeline operations for batch processing
- Lua scripts provide atomic operations without network round-trips
- Monitor metrics to identify bottlenecks and optimize accordingly

## Performance Benchmarking

The Redis Cache implementation includes a comprehensive benchmarking suite to evaluate performance across various operations and scenarios. This enables you to understand the performance characteristics of the cache and make informed decisions about configuration and usage patterns.

### Running Benchmarks

To run the benchmarks:

```bash
# Run all benchmarks with default settings
cargo bench --bench redis_benchmark

# Run a specific benchmark group
cargo bench --bench redis_benchmark -- "Basic Operations"

# Run a specific benchmark
cargo bench --bench redis_benchmark -- "Pipeline operations"

# Output detailed results for visualization
cargo bench --bench redis_benchmark -- --verbose > benchmark_results.txt
```

### Visualizing Benchmark Results

The package includes a benchmark visualizer to help interpret results:

```bash
# Run the visualizer on the benchmark results
cargo run --example benchmark_visualizer -- benchmark_results.txt
```

This will generate bar charts and performance comparisons to help you understand the relative performance of different operations.

### Benchmark Categories

The benchmark suite covers the following categories:

1. **Basic Operations**
   - GET/SET operations for strings
   - SET with expiration
   - DELETE operations
   - EXISTS checks

2. **Serialization Performance**
   - Serialization of small objects
   - Serialization of collections
   - Deserialization performance

3. **Data Structure Operations**
   - List operations (LPUSH, RPUSH, LPOP, LRANGE)
   - Hash operations (HSET, HGET, HGETALL)
   - Set operations (SADD, SISMEMBER, SMEMBERS, SINTER, SUNION)
   - Sorted Set operations (ZADD, ZRANGE, ZRANGEBYSCORE)

4. **Lua Scripting Performance**
   - Simple Lua script execution
   - Complex script operations

5. **Pipelining**
   - Pipeline vs. individual operations comparison
   - Scaling with operation count

6. **Connection Pool Performance**
   - Concurrent operation handling
   - Pool size impact on throughput

### Performance Optimization Guidelines

Based on benchmark findings, here are some guidelines for optimizing Redis Cache performance:

1. **Use pipelining for bulk operations**: The benchmarks demonstrate that pipelining can provide significant performance improvements (often 5-10x) when executing multiple Redis commands in sequence.

2. **Optimize connection pool size**: Benchmark your specific workload to determine the optimal connection pool size. Too few connections can limit throughput, while too many might waste resources.

3. **Consider serialization overhead**: For complex objects, serialization can become a bottleneck. Use compact serialization formats and consider caching frequently accessed objects.

4. **Leverage Lua scripts**: For operations that require multiple commands, Lua scripts can reduce network roundtrips and provide atomic execution.

5. **Balance TTL settings**: Setting expiration times adds some overhead. Only use TTL when necessary, and consider appropriate values based on your application's needs.

### Interpreting Benchmark Results

The benchmark results provide several key metrics:

- **Average Time**: The mean execution time per operation
- **Throughput**: Operations per second the cache can handle
- **Min/Max Times**: Range of performance variation

For most applications, the throughput (operations per second) is the most important metric to optimize for.

### Customizing Benchmarks

You can customize the benchmarks for your specific environment by modifying the Redis connection parameters in the `create_test_cache()` function in the benchmark code.

## License

Apache 2.0 