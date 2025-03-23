# IDE Configuration

This directory contains recommended IDE configurations to improve the development experience for the Navius project.

## Visual Studio Code

### Setup Instructions

1. Copy the files from the `vscode` directory to your `.vscode` directory in the project root:

```bash
mkdir -p .vscode
cp .devtools/ide/vscode/* .vscode/
```

2. Restart VS Code or reload the window.

3. When prompted, install the recommended extensions.

### Key Features

- **Rust Analyzer Configuration**: Optimized settings for Rust development
- **Code Formatting**: Automatic code formatting on save
- **Launch Configurations**: Pre-configured debug configurations for:
  - Running the Navius server
  - Running all unit tests
  - Running a specific test
- **File Associations**: Proper association of Rust test files
- **Search Exclusions**: Excludes generated code and build artifacts from search results
- **Recommended Extensions**: Curated list of useful extensions for Rust development
- **Project-specific Theming**: Custom colors to quickly identify you're working in the Navius project
- **File Nesting**: Improved file organization with intuitive nesting of related files
- **Enhanced Navigation**: Custom tree rendering and icon theme for better project structure visibility
- **Todo Tracking**: Configured Todo Tree extension with custom highlighting for different types of tasks
- **Spelling Checker**: Pre-configured with domain-specific terms for the project

### Enhanced Productivity Features

The settings include specific configurations for the new project structure:

- **Module Organization**: Icon themes that distinguish between `app` and `core` directories
- **Code Navigation**: File nesting patterns that group related files (e.g., mod.rs with its children)
- **Visual Cues**: Custom colors for different types of code markers (TODO, FIXME, REFACTOR, etc.)
- **Code Quality**: Integration with Clippy for enhanced linting

## IntelliJ IDEA / CLion Setup

For JetBrains IDEs (when available), you can:

1. Import the Rust project using the built-in Rust plugin
2. Configure the debugger to use the native Rust debugging support
3. Use the provided run configurations

## JetBrains Fleet

Support for Fleet will be added in the future as it becomes more widely adopted.

## Common IDE Features

Regardless of your IDE, make sure to configure:

1. **Rust Analyzer**: For code intelligence and navigation
2. **Clippy Integration**: For enhanced linting
3. **Test Runner Integration**: For easily running tests
4. **Debugger**: For debugging the application and tests

## Troubleshooting

If you encounter any issues with the IDE configurations:

1. Make sure you have the latest version of your IDE
2. Verify that Rust tools are properly installed
3. Check that the paths in configurations match your project structure
4. Consider posting an issue if the problem persists 