---
title: "IDE Setup for Navius Development"
description: "Documentation about IDE Setup for Navius Development"
category: contributing
tags:
  - development
last_updated: March 23, 2025
version: 1.0
---
# IDE Setup for Navius Development

This guide provides instructions for setting up your Integrated Development Environment (IDE) for working with the Navius codebase.

## Recommended IDE Configurations

The Navius project includes recommended IDE configurations in the `.devtools/ide` directory. These configurations provide consistent settings, useful tasks, debug configurations, and recommended extensions.

## Visual Studio Code

### Quick Setup

1. Copy the configuration files to your local `.vscode` directory:

```bash
mkdir -p .vscode
cp .devtools/ide/vscode/* .vscode/
```

2. Restart VS Code or reload the window.

3. Install the recommended extensions when prompted.

### Key Features

The VS Code configuration includes:

- **Launch Configurations**: Debug configurations for running the server and tests
- **Tasks**: Common tasks like build, test, and formatting
- **Settings**: Optimized settings for Rust development
- **Extensions**: Recommended extensions for Rust development

### Debugging

To debug the Navius server:

1. Press `F5` or use the Run and Debug panel to select "Debug Navius Server"
2. Set breakpoints in your code
3. Use the debug console to inspect variables

To debug tests:

1. Select "Debug Unit Tests" to run all tests with the debugger
2. Use "Debug Specific Test" to debug a single test

## JetBrains IDEs (CLion/IntelliJ IDEA)

For JetBrains IDEs, follow these steps:

1. Install the Rust plugin
2. Open the Navius project
3. Configure the Rust toolchain:
   - Settings/Preferences → Languages & Frameworks → Rust
   - Set the toolchain location

### Debugging in JetBrains IDEs

1. Create a Run/Debug Configuration for the Navius binary
2. Set the working directory to the project root
3. Add necessary environment variables:
   - `RUST_LOG=debug`
   - `CONFIG_DIR=./config`
   - `RUN_ENV=development`

## Common Issues and Troubleshooting

### Rust Analyzer Issues

If you experience issues with Rust Analyzer:

1. Check if the Rust Analyzer extension is installed and enabled
2. Reload the window (`Ctrl+Shift+P` → "Reload Window")
3. Run "Rust Analyzer: Restart Server" from the command palette

### Build Issues

If you encounter build issues in the IDE:

1. Run `cargo clean` from the terminal
2. Ensure your Rust toolchain is up to date with `rustup update`
3. Try building from the command line with `cargo build`

### Debugging Issues

If debugging doesn't work properly:

1. Ensure you have the appropriate debugger installed (LLDB for macOS/Linux, Microsoft C++ tools for Windows)
2. Check that your launch configuration has the correct paths and environment variables
3. Try running the application from the command line first to verify it builds and runs correctly

## Additional Resources

- [Rust Analyzer Manual](https://rust-analyzer.github.io/manual.html)
- [VS Code Rust Debugging](https://code.visualstudio.com/docs/languages/rust)
- [CLion Rust Guide](https://www.jetbrains.com/help/clion/rust-support.html) 
