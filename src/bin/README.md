# Navius Binary Targets

This directory is reserved for additional binary targets that extend or complement the main Navius application.

## Purpose

While the main application entry point is defined in `src/main.rs`, this directory follows Rust conventions for organizing multiple binary targets in a single package.

## Adding New Binaries

To add a new binary target:

1. Create a new file in this directory (e.g., `src/bin/your_tool.rs`)
2. Implement a `main` function as the entry point
3. The binary will be compiled with the same name as the file

Example:

```rust
// src/bin/migrate.rs
use navius::core::database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running database migrations...");
    // Implementation
    Ok(())
}
```

This will create a binary named `migrate` that can be executed separately from the main application.

## When to Use Binary Targets

Consider adding a binary target when you need:

- Utility tools for administration or maintenance
- Background workers or scheduled tasks
- Data migration scripts
- Development tools

## Building Binary Targets

Build all binaries (including the main application):

```bash
cargo build
```

Build a specific binary target:

```bash
cargo build --bin binary_name
```

Run a specific binary target:

```bash
cargo run --bin binary_name
``` 