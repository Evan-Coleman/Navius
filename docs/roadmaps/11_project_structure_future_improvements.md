# Future Improvements

**Created On:** March 23, 2025
**Updated On:** March 24, 2025

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
- [ ] Expand the onboarding guide with additional examples

## Build Process Improvements

- [ ] Optimize the build process for faster compilation
- [ ] Add additional linting rules to enforce the new structure
- [ ] Enhance the structure verification script to provide more detailed recommendations

## Next Steps

These improvements should be prioritized and added to the team's backlog for future sprints. The remaining documentation and build process improvements should be considered for the next sprint to complete the transition to the new structure.

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

### Findings and Decisions

1. **src/bin directory**: This directory exists but is currently empty. The main binary is built from src/main.rs. We have left this directory in place as it follows Rust conventions and could be used for additional binaries in the future.

2. **IDE Configuration**: VS Code settings have been created to improve developer experience with the new structure, including file nesting, icon themes, and search exclusions. 