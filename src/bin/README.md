# Utility Binaries Directory

This directory is intended for additional binary targets (executables) that complement the main Navius application.

## Purpose

The `src/bin` directory follows Rust's standard convention for organizing multiple binary targets in a Cargo project. While the main application binary is defined in `src/main.rs`, this directory allows for additional executables that:

- Provide utility or maintenance functionality
- Serve as specialized entry points for different use cases
- Offer tooling related to the main application

## Adding a New Binary

To add a new utility binary, create a Rust file directly in this directory:

```
src/bin/my_utility.rs
```

This will automatically create a binary target named `my_utility` that can be built with:

```bash
cargo build --bin my_utility
```

And run with:

```bash
cargo run --bin my_utility
```

## Examples of Potential Utility Binaries

This directory could include binaries such as:

1. **Migration Tools**
   ```
   src/bin/migrate.rs - Database migration utility
   ```

2. **Data Processing Scripts**
   ```
   src/bin/import_data.rs - Import data from external sources
   src/bin/export_data.rs - Export data to various formats
   ```

3. **Admin Utilities**
   ```
   src/bin/admin_tools.rs - Administrative functions for maintenance
   ```

4. **Benchmarking or Testing Tools**
   ```
   src/bin/benchmark.rs - Performance testing utilities
   ```

## Design Guidelines

When creating utility binaries:

1. Keep binaries focused on a single responsibility
2. Reuse code from the main application through the library interface
3. Maintain consistent error handling and logging patterns
4. Include proper documentation and usage examples
5. Consider adding tests in `tests/bin/`

## Example Binary Structure

```rust
use navius::core::{config, database, error};
use std::process;

fn main() {
    // Initialize logging
    if let Err(e) = setup_logging() {
        eprintln!("Failed to set up logging: {}", e);
        process::exit(1);
    }

    // Parse command line arguments
    let args = parse_args();

    // Run the actual utility code
    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn setup_logging() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging similar to main application
    // ...
    Ok(())
}

fn parse_args() -> Args {
    // Parse command line arguments
    // ...
    Args {}
}

struct Args {
    // Command line arguments
}

fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // Utility functionality
    // ...
    Ok(())
}
``` 