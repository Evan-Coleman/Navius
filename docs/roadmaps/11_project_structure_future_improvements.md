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

- [ ] Evaluate the purpose of `src/bin` and determine if it should be relocated or restructured
- [ ] Create additional module diagrams for newly organized directories
- [ ] Update IDE configurations based on the final structure

## Documentation Improvements

- [ ] Update all code examples in documentation to reflect the new structure
- [ ] Create a developer cheatsheet for the new structure
- [ ] Add more detailed explanations to the module dependencies document
- [ ] Expand the onboarding guide with additional examples

## Build Process Improvements

- [ ] Optimize the build process for faster compilation
- [ ] Add additional linting rules to enforce the new structure
- [ ] Enhance the structure verification script to provide more detailed recommendations

## Next Steps

These improvements should be prioritized and added to the team's backlog for future sprints. The low priority items should be considered for the next sprint to complete the transition to the new structure.

To track progress on these improvements, create tickets in the issue tracker and reference this document. As items are completed, they can be checked off in this document to maintain a clear overview of the remaining work. 