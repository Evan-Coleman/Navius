# Navius Cache Documentation

Welcome to the navius-cache documentation! This crate provides a flexible, type-safe caching infrastructure for the Navius framework.

## Documentation

- [README](./README.md) - Main documentation with features, installation, and usage
- [API Documentation](../target/doc/navius_cache/index.html) - Generated API documentation (run `cargo doc --open`)

## Examples

- [Basic Usage](./examples/basic_usage.rs) - Basic cache operations with in-memory backend
- [Metrics Usage](./examples/metrics_usage.rs) - Using metrics to monitor cache operations
- [Repository Pattern](./examples/repository_pattern.rs) - Integrating cache with a repository pattern

## Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# Metrics usage example
cargo run --example metrics_usage --features metrics

# Repository pattern example
cargo run --example repository_pattern

# Main metrics example
cargo run --example metrics --features metrics
```

## Integration with Other Crates

The navius-cache crate is designed to work seamlessly with other Navius crates:

- **navius-core**: Shares common error handling and configuration patterns
- **navius-http**: Can be used to cache HTTP responses
- **navius-db**: Can be used to cache database query results
- **navius-auth**: Can be used to cache authentication tokens

## Related Crates

For Redis caching support, check out the [navius-cache-redis](../navius-cache-redis/README.md) crate.

## Feedback and Contributions

We welcome feedback and contributions to the navius-cache crate. Please submit issues and pull requests to the GitHub repository.

*Updated: May 30, 2025* 