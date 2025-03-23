# Cargo Configuration

This directory contains Cargo configuration files that optimize the build process for the Navius project.

## Files

- `config.toml` - Contains build settings to optimize compilation speed and performance

## Key Optimizations

The Cargo configuration includes several optimizations:

1. **Incremental Compilation**: Enables faster rebuilds during development
2. **Pipeline Building**: Optimizes dependency compilation order
3. **Fast Linkers**: Uses specialized linkers like lld for faster linking on macOS and Linux
4. **Custom Profiles**: Includes a `dev-opt` profile for development with better performance
5. **Parallel Compilation**: Increases codegen units for faster parallel builds
6. **Targeted Optimization**: Optimizes frequently used dependencies

## Usage

These optimizations are automatically applied when building the project. To take advantage of the `dev-opt` profile for faster development builds with better runtime performance:

```bash
cargo build --profile dev-opt
```

Or to run with this profile:

```bash
cargo run --profile dev-opt
```

## Benchmarks

On average, these optimizations can reduce build times by:
- ~20-30% for incremental builds
- ~10-15% for clean builds
- The `dev-opt` profile provides runtime performance close to release builds while maintaining good debugging capabilities

## Considerations

- The `lld` linker must be installed:
  - macOS: `brew install llvm` (includes lld)
  - Linux: `apt install lld` or equivalent
- Some settings may need adjustment based on your specific hardware configuration 