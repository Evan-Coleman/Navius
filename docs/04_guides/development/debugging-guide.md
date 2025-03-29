---
title: "Debugging Guide for Navius Development"
description: "Comprehensive techniques and tools for debugging Navius applications"
category: "Guides"
tags: ["development", "debugging", "troubleshooting", "logging", "performance", "rust"]
last_updated: "April 7, 2025"
version: "1.0"
---

# Debugging Guide for Navius Development

This guide provides comprehensive instructions and best practices for debugging Navius applications. Effective debugging is essential for maintaining code quality and resolving issues efficiently.

## Table of Contents

- [Debugging Philosophy](#debugging-philosophy)
- [Common Debugging Scenarios](#common-debugging-scenarios)
- [Debugging Tools](#debugging-tools)
- [Logging and Tracing](#logging-and-tracing)
- [Rust-Specific Debugging Techniques](#rust-specific-debugging-techniques)
- [Database Debugging](#database-debugging)
- [API Debugging](#api-debugging)
- [Performance Debugging](#performance-debugging)
- [Advanced Debugging Scenarios](#advanced-debugging-scenarios)
- [Debugging in Production](#debugging-in-production)

## Debugging Philosophy

Effective debugging in Navius development follows these principles:

1. **Reproduce First** - Create a reliable reproduction case before attempting to fix an issue
2. **Isolate the Problem** - Narrow down the scope of the issue
3. **Data-Driven Approach** - Use facts, logs, and evidence rather than guesswork
4. **Systematic Investigation** - Follow a methodical process rather than random changes
5. **Root Cause Analysis** - Fix the underlying cause, not just the symptoms

## Common Debugging Scenarios

### Application Crashes

When your Navius application crashes:

1. **Check the Stack Trace** - Identify where the crash occurred
2. **Examine Error Messages** - Parse logs for error details
3. **Reproduce the Crash** - Create a minimal test case
4. **Check for Resource Issues** - Verify memory usage and system resources
5. **Review Recent Changes** - Consider what code changed recently

Example stack trace analysis:

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: DatabaseError { kind: ConnectionError, cause: Some("connection refused") }', src/services/user_service.rs:52:10
stack backtrace:
   0: std::panicking::begin_panic_handler
   1: std::panicking::panic_handler
   2: core::panicking::panic_fmt
   3: core::result::unwrap_failed
   4: navius::services::user_service::UserService::find_by_id
   5: navius::handlers::user_handlers::get_user
   6: navius::main
```

This indicates:
- The crash is in `user_service.rs` line 52
- It's unwrapping a database connection error
- The connection is being refused

### Runtime Errors

For non-crash errors (incorrect behavior):

1. **Identify the Expected vs. Actual Behavior**
2. **Use Logging to Track Flow**
3. **Create Unit Tests** to reproduce and verify the issue
4. **Use Debugger Breakpoints** at key decision points

### Build Errors

For build failures:

1. **Read Compiler Messages Carefully** - Rust provides detailed error messages
2. **Check Dependencies** - Verify Cargo.toml and dependency versions
3. **Use Tools** - Clippy can identify additional issues
4. **Clean and Rebuild** - `cargo clean && cargo build`

### Debugging Tests

For test failures:

1. **Run Single Test** - Focus on one test with `cargo test test_name`
2. **Use `--nocapture`** - See output with `cargo test -- --nocapture`
3. **Add Debugging Prints** - Temporarily add print statements
4. **Use Test-Specific Logs** - Enable debug logging during tests

## Debugging Tools

### IDE Debuggers

#### Visual Studio Code

1. Setup configuration in `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Navius Server",
      "cargo": {
        "args": ["build", "--bin=navius"],
        "filter": {
          "name": "navius",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug",
        "CONFIG_DIR": "./config",
        "RUN_ENV": "development"
      }
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Unit Tests",
      "cargo": {
        "args": ["test", "--no-run"],
        "filter": {
          "name": "navius",
          "kind": "lib"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

2. Set breakpoints by clicking in the gutter
3. Start debugging by pressing F5 or using the Debug menu
4. Use the Debug panel to:
   - Step through code (F10)
   - Step into functions (F11)
   - View variables and their values
   - Evaluate expressions in the Debug Console

#### JetBrains IDEs (CLion/IntelliJ with Rust plugin)

1. Create Run/Debug configurations for:
   - Main application
   - Specific test files
   - All tests

2. Debugging features to use:
   - Expression evaluation
   - Memory view
   - Smart step-into
   - Conditional breakpoints

### Command Line Debugging

For environments without IDE support, use:

1. **LLDB/GDB**:
   ```bash
   # Build with debug symbols
   cargo build
   
   # Start debugger
   lldb ./target/debug/navius
   
   # Set breakpoints
   breakpoint set --file user_service.rs --line 52
   
   # Run program
   run
   
   # After hitting breakpoint
   frame variable  # Show variables in current frame
   thread backtrace  # Show current stack
   expression user.id  # Evaluate expression
   ```

2. **cargo-lldb**:
   ```bash
   cargo install cargo-lldb
   cargo lldb --bin navius
   ```

### Specialized Debugging Tools

1. **Memory Analysis**:
   - Valgrind for memory leaks: `valgrind --leak-check=full ./target/debug/navius`
   - ASAN (Address Sanitizer): Build with `-Z sanitizer=address`

2. **Thread Analysis**:
   - Inspect thread states: `ps -T -p <PID>`
   - Thread contention: `perf record -g -p <PID>`

3. **Network Debugging**:
   - Wireshark for packet analysis
   - `tcpdump` for network traffic capture
   - `curl` for API request testing

## Logging and Tracing

### Structured Logging

Navius uses the `tracing` crate for structured logging:

```rust
use tracing::{debug, error, info, instrument, warn};

#[instrument(skip(password))]
pub async fn authenticate_user(username: &str, password: &str) -> Result<User, AuthError> {
    debug!("Attempting to authenticate user: {}", username);
    
    match user_repository.find_by_username(username).await {
        Ok(user) => {
            if verify_password(password, &user.password_hash) {
                info!("User authenticated successfully: {}", username);
                Ok(user)
            } else {
                warn!("Failed authentication attempt for user: {}", username);
                Err(AuthError::InvalidCredentials)
            }
        }
        Err(e) => {
            error!(error = ?e, "Database error during authentication");
            Err(AuthError::DatabaseError(e))
        }
    }
}
```

### Log Levels

Use appropriate log levels:

- **ERROR**: Application errors requiring immediate attention
- **WARN**: Unexpected situations that don't cause application failure
- **INFO**: Important events for operational insights
- **DEBUG**: Detailed information useful for debugging
- **TRACE**: Very detailed information, typically for pinpointing issues

### Configuring Logging

Set via environment variables:

```bash
# Set log level
export RUST_LOG=navius=debug,warp=info

# Log to file
export RUST_LOG_STYLE=always
export RUST_LOG_FILE=/var/log/navius.log
```

Or in code:

```rust
use tracing_subscriber::{self, fmt::format::FmtSpan, EnvFilter};

fn setup_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("navius=info,warp=warn"));
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_file(true)
        .with_line_number(true)
        .init();
}
```

### Log Analysis

For analyzing logs:

1. **Search with grep/ripgrep**:
   ```bash
   rg "error|exception" navius.log
   ```

2. **Context with before/after lines**:
   ```bash
   rg -A 5 -B 2 "DatabaseError" navius.log
   ```

3. **Filter by time period**:
   ```bash
   rg "2023-04-07T14:[0-5]" navius.log
   ```

4. **Count occurrences**:
   ```bash
   rg -c "AUTH_FAILED" navius.log
   ```

## Rust-Specific Debugging Techniques

### Debug Prints

Use `dbg!` macro for quick debugging:

```rust
// Instead of
let result = complex_calculation(x, y);
println!("Result: {:?}", result);

// Use dbg! to show file/line and expression
let result = dbg!(complex_calculation(x, y));
```

### Unwrap Alternatives

Replace `unwrap()` and `expect()` with better error handling:

```rust
// Instead of
let user = db.find_user(id).unwrap();

// Use more descriptive handling
let user = db.find_user(id)
    .map_err(|e| {
        error!("Failed to retrieve user {}: {:?}", id, e);
        e
    })?;
```

### Narrowing Down Rust Compiler Errors

For complex compile errors:

1. **Binary Search** - Comment out sections of code until error disappears
2. **Type Annotations** - Add explicit type annotations to clarify issues
3. **Minimal Example** - Create a minimal failing example
4. **Check Versions** - Verify dependency versions for compatibility

### Debugging Async Code

Async code can be challenging to debug:

1. **Instrument async functions**:
   ```rust
   #[instrument(skip(request))]
   async fn handle_request(request: Request) -> Response {
       // ...
   }
   ```

2. **Use `output_span_events`** to trace async execution:
   ```rust
   tracing_subscriber::fmt()
       .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
       .init();
   ```

3. **Inspect tasks**:
   ```rust
   tokio::spawn(async move {
       let span = tracing::info_span!("worker_task", id = %task_id);
       let _guard = span.enter();
       // task code...
   });
   ```

### Memory Analysis

For memory issues:

1. **Check for leaks** with `Drop` trait:
   ```rust
   impl Drop for MyResource {
       fn drop(&mut self) {
           debug!("MyResource being dropped: {:?}", self.id);
       }
   }
   ```

2. **Use weak references** where appropriate:
   ```rust
   use std::rc::{Rc, Weak};
   use std::cell::RefCell;
   
   struct Parent {
       children: Vec<Rc<RefCell<Child>>>,
   }
   
   struct Child {
       parent: Weak<RefCell<Parent>>,
   }
   ```

## Database Debugging

### Query Analysis

For slow or problematic database queries:

1. **Query Logging** - Enable PostgreSQL query logging:
   ```
   # In postgresql.conf
   log_min_duration_statement = 100  # Log queries taking > 100ms
   ```

2. **Query Explain** - Use EXPLAIN ANALYZE:
   ```sql
   EXPLAIN ANALYZE SELECT * FROM users WHERE email LIKE '%example.com';
   ```

3. **Check Indexes** - Verify appropriate indexes exist:
   ```sql
   SELECT indexname, indexdef FROM pg_indexes WHERE tablename = 'users';
   ```

### Connection Issues

For database connection problems:

1. **Connection Pool Diagnostics**:
   ```rust
   // Log connection pool status
   info!(
       "DB Pool: active={}, idle={}, size={}",
       pool.status().await.active,
       pool.status().await.idle,
       pool.status().await.size
   );
   ```

2. **Check Connection Parameters**:
   ```rust
   let conn_params = PgConnectOptions::new()
       .host(&config.db_host)
       .port(config.db_port)
       .username(&config.db_user)
       .password(&config.db_password)
       .database(&config.db_name);
   
   debug!("Connection parameters: {:?}", conn_params);
   ```

3. **Manual Connection Test**:
   ```bash
   PGPASSWORD=your_password psql -h hostname -U username -d database -c "\conninfo"
   ```

## API Debugging

### Request/Response Logging

For API debugging:

1. **Add request/response middleware**:
   ```rust
   async fn log_request_response(
       req: Request,
       next: Next,
   ) -> Result<impl IntoResponse, (StatusCode, String)> {
       let path = req.uri().path().to_string();
       let method = req.method().clone();
       
       let req_id = Uuid::new_v4();
       let start = std::time::Instant::now();
       
       info!(request_id = %req_id, %method, %path, "Request received");
       
       let response = next.run(req).await;
       
       let status = response.status();
       let duration = start.elapsed();
       
       info!(
           request_id = %req_id,
           %method,
           %path,
           status = %status.as_u16(),
           duration_ms = %duration.as_millis(),
           "Response sent"
       );
       
       Ok(response)
   }
   ```

2. **API Testing Tools**:
   - Use Postman or Insomnia for manual API testing
   - Create collections for common request scenarios
   - Save environments for different setups (dev, test, prod)

3. **Curl for quick tests**:
   ```bash
   curl -v -X POST http://localhost:3000/api/users \
     -H "Content-Type: application/json" \
     -d '{"username":"test", "password":"test123"}'
   ```

## Performance Debugging

### Identifying Performance Issues

1. **Profiling** with `flamegraph`:
   ```bash
   cargo install flamegraph
   CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --bin navius
   ```

2. **Benchmarking** with Criterion:
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   
   fn benchmark_user_service(c: &mut Criterion) {
       let service = UserService::new(/* dependencies */);
       
       c.bench_function("find_user_by_id", |b| {
           b.iter(|| service.find_by_id(black_box(1)))
       });
   }
   
   criterion_group!(benches, benchmark_user_service);
   criterion_main!(benches);
   ```

3. **Request Timing**:
   ```rust
   async fn handle_request() -> Response {
       let timer = std::time::Instant::now();
       
       // Handle request...
       
       let duration = timer.elapsed();
       info!("Request processed in {}ms", duration.as_millis());
       
       // Return response...
   }
   ```

### Common Performance Issues

1. **N+1 Query Problem**:
   - Symptom: Multiple sequential database queries
   - Solution: Use joins or batch fetching

2. **Missing Indexes**:
   - Symptom: Slow queries with table scans
   - Solution: Add appropriate indexes

3. **Blocking Operations in Async Context**:
   - Symptom: High latency, thread pool exhaustion
   - Solution: Move blocking operations to blocking task pool
   ```rust
   let result = tokio::task::spawn_blocking(move || {
       // CPU-intensive or blocking operation
       expensive_calculation()
   }).await?;
   ```

4. **Memory Leaks**:
   - Symptom: Growing memory usage over time
   - Solution: Check for unclosed resources, circular references

## Advanced Debugging Scenarios

### Race Conditions

For debugging concurrency issues:

1. **Add Tracing for Async Operations**:
   ```rust
   #[instrument(skip(data))]
   async fn process_data(id: u64, data: Vec<u8>) {
       info!("Starting processing");
       // Processing code...
       info!("Finished processing");
   }
   ```

2. **Use Atomic Operations**:
   ```rust
   use std::sync::atomic::{AtomicUsize, Ordering};
   
   static COUNTER: AtomicUsize = AtomicUsize::new(0);
   
   fn increment_counter() {
       let prev = COUNTER.fetch_add(1, Ordering::SeqCst);
       debug!("Counter incremented from {} to {}", prev, prev + 1);
   }
   ```

3. **Debugging Deadlocks**:
   - Add timeout to lock acquisitions
   - Log lock acquisition/release
   - Use deadlock detection in development

### Memory Corruption

For possible memory corruption:

1. **Use Address Sanitizer**:
   ```bash
   RUSTFLAGS="-Z sanitizer=address" cargo test
   ```

2. **Check Unsafe Code**:
   - Review all `unsafe` blocks
   - Verify pointer safety
   - Check lifetime correctness

3. **Foreign Function Interface Issues**:
   - Verify signature matches
   - Check data marshaling
   - Ensure proper resource cleanup

## Debugging in Production

### Safe Production Debugging

1. **Structured Logging**:
   - Use context-rich structured logs
   - Include correlation IDs for request tracing
   - Log adequate information without sensitive data

2. **Metrics and Monitoring**:
   - Track key performance indicators
   - Set up alerts for anomalies
   - Use distributed tracing for complex systems

3. **Feature Flags**:
   - Enable additional logging in production for specific issues
   ```rust
   if feature_flags.is_enabled("enhanced_auth_logging") {
       debug!("Enhanced auth logging: {:?}", auth_details);
   }
   ```

### Post-Mortem Analysis

For analyzing production issues after they occur:

1. **Log Aggregation**:
   - Collect logs centrally
   - Use tools like ELK Stack or Grafana Loki
   - Create dashboards for common issues

2. **Error Tracking**:
   - Integrate with error tracking services
   - Group similar errors
   - Track error rates and trends

3. **Core Dumps**:
   - Enable core dumps in production
   - Secure sensitive information
   - Analyze with `rust-gdb`

## Related Resources

- [Rust Debugging with LLDB/GDB](https://rustc-dev-guide.rust-lang.org/debugging.html)
- [Effective Error Handling in Rust](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Navius Logging Setup](../operations/logging-and-monitoring.md)
- [Testing Guide](./testing-guide.md)
- [IDE Setup for Debugging](./ide-setup.md)
- [PostgreSQL Performance Tuning](../performance/database-optimization.md)
