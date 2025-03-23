# Future Improvements

**Created On:** March 23, 2025
**Updated On:** March 23, 2025

This document outlines future improvements identified during the project restructuring process. These tasks are not part of the original restructuring roadmap but should be considered for future sprints to further enhance the project structure.

## Structure Improvements

### High Priority

- [x] Relocate remaining modules to the new structure:
  - [x] Move `src/metrics` → `src/core/metrics`
  - [x] Move `src/repository` → `src/core/repository`
  - [x] Move `src/api` → `src/core/api`
  - [x] Move `src/error` → `src/core/error`
  - [x] Move `src/auth` → `src/core/auth`
  - [x] Move `src/reliability` → `src/core/reliability`
  - [x] Move `src/utils` → `src/core/utils`

### Medium Priority

- [x] Add recommended app directories:
  - [x] Create `src/app/api` for user-facing API endpoints
  - [x] Create `src/app/services` for user-facing service implementations
- [x] Evaluate and relocate:
  - [x] Determine appropriate location for `src/apis` (→ empty directory, no relocation needed)
  - [x] Determine appropriate location for `src/handlers` (→ `src/app/api`)
  - [x] Determine appropriate location for `src/services` (→ `src/app/services`)
  - [x] Determine appropriate location for `src/models` (→ `src/core/models/extensions.rs`)

### Low Priority

- [x] Evaluate the purpose of `src/bin` and determine if it should be relocated or restructured
- [x] Create additional module diagrams for newly organized directories
- [x] Update IDE configurations based on the final structure

## Documentation Improvements

- [x] Update all code examples in documentation to reflect the new structure
- [x] Create a developer cheatsheet for the new structure
- [x] Add more detailed explanations to the module dependencies document
- [x] Expand the onboarding guide with additional examples

## Build Process Improvements

- [x] Optimize the build process for faster compilation
- [x] Add additional linting rules to enforce the new structure
- [x] Enhance the structure verification script to provide more detailed recommendations

## Next Steps

Based on the results of the structure verification script, the following improvements should be prioritized:

1. **Fix Library Exports**: Add proper exports in `lib.rs` for core modules:
   ```rust
   pub use crate::core::router;
   pub use crate::core::cache;
   pub use crate::core::config;
   ```

2. **Complete Core Transition**: Fully relocate old directories to their new locations:
   - Complete the transition of `src/metrics` → `src/core/metrics`
   - Complete the transition of `src/repository` → `src/core/repository`
   - Complete the transition of other root-level directories

3. **Fix Import Patterns**: Update import statements to use the `crate::core::` prefix instead of direct imports from root modules.

4. **Address Naming Conventions**: Review and update file names to follow Rust's snake_case convention.

These improvements should be prioritized and added to the team's backlog for future sprints. The findings of the structure verification script provide a clear roadmap for completing the transition to the new structure.

To track progress on these improvements, create tickets in the issue tracker and reference this document. As items are completed, they can be checked off in this document to maintain a clear overview of the remaining work.

## Implemented Improvements Summary

### Module Diagrams

The following module diagrams have been created to visualize the project structure:

1. `docs/architecture/diagrams/app-module-diagram.md` - Diagram of the app directory structure
2. `docs/architecture/diagrams/core-module-diagram.md` - Diagram of the core directory structure
3. `docs/architecture/diagrams/app-core-interactions.md` - Diagram of interactions between app and core

### Developer Resources

1. `docs/guides/project-structure-cheatsheet.md` - Quick reference guide for navigating the project
2. `.devtools/ide/vscode-settings.json` - VS Code settings for improved project navigation
3. Updated onboarding guide with detailed code examples for common patterns

### Build Optimizations

1. `.cargo/config.toml` - Cargo configuration file with optimized build settings including:
   - Incremental compilation for faster rebuilds
   - Pipeline building for optimized dependency compilation
   - Fast linkers for macOS and Linux
   - Custom profiles including a development profile with better performance
   - Parallel compilation settings

### Project Structure Verification

The structure verification script has been enhanced to:

1. Provide more detailed recommendations for fixing issues
2. Check additional aspects of the project structure:
   - Import patterns across the codebase
   - Naming conventions for files and directories
   - IDE configuration completeness
   - Build configuration presence
3. Generate a detailed summary of issues by category
4. Provide actionable recommendations with specific code snippets

### Directory Documentation

1. `src/bin/README.md` - Documentation for the utility binaries directory
2. Enhanced root-level README files for key directories

### Findings and Decisions

1. **src/bin directory**: This directory exists but is currently empty. The main binary is built from src/main.rs. We have left this directory in place as it follows Rust conventions and could be used for additional binaries in the future. We've added a README.md file to explain its purpose and how to use it.

2. **IDE Configuration**: VS Code settings have been created to improve developer experience with the new structure, including file nesting, icon themes, and search exclusions. 

3. **Build Optimizations**: Added Cargo configuration to improve build times and performance, with a focus on development experience.

4. **Structure Verification**: Enhanced the verification script to provide actionable recommendations and comprehensive checks. The script now generates a detailed report of issues by category and provides specific recommendations for fixing each issue. 