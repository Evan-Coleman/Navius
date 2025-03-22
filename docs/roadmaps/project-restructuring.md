# Project Restructuring Roadmap

**Status:** In Progress  
**Updated At:** March 22, 2025
**Priority:** High  

## Overview

This roadmap outlines a plan to iteratively restructure the Navius project to improve organization, maintainability, and developer experience. The main goals are:

1. Simplify the root directory to make the README more visible
2. Organize code following clean architecture principles
3. Standardize file locations and naming conventions
4. Improve discoverability for new developers
5. Support the core transition initiative

## Steps

### 1. Consolidate Top-Level Directories ✅
- [x] Create .devtools directory
- [x] Move development tools and configurations
- [x] Update path references
- [x] Verify directory structure

### 2. Standardize Source Code Organization ✅
- [x] Move services module to core and create user-facing scaffold
- [x] Move remaining modules (models, repository, api, utils)
- [x] Update import paths throughout the codebase
- [x] Verify all tests pass after migration

### 3. Improve Documentation Structure ✅
- [x] Create a structured documentation hierarchy
- [x] Ensure consistent formatting across all documentation
- [x] Update links and references throughout documentation
- [x] Add a documentation index

### 4. Standardize Testing Structure ✅
- [x] Consolidate test directories
- [x] Establish consistent test naming conventions
- [x] Update testing roadmap to reflect new structure
- [x] Ensure test coverage metrics work with the new structure

### 5. Optimize Build Configuration ✅
- [x] Standardize environment configuration files
- [x] Optimize build scripts for different environments
- [x] Update deployment scripts to work with new structure
- [x] Ensure CI/CD pipelines are compatible with new structure

### 6. Clean Up Generated Code Management ✅
- [x] Establish clear guidelines for generated code
- [x] Update generation scripts to follow the new structure
- [x] Document the code generation process
- [x] Add appropriate gitignore patterns for generated files

### 7. Create Developer Tooling for Navigation ⏳
- [x] Create project maps or diagrams
- [x] Update IDE configuration files
- [x] Create helper scripts for common tasks
- [x] Update onboarding documentation for new developers
- [ ] Consolidate project structure documentation

## Detailed Work Plan

### Step 1: Consolidate Top-Level Directories

1. Create a `.devtools` directory and move:
   - `scripts/` → `.devtools/scripts/`
   - `coverage/` → `.devtools/coverage/`
   - Development-only shell scripts → `.devtools/scripts/`

2. Organize CI/CD files:
   - Keep `.gitlab-ci.yml` at the root
   - Move `.gitlab/` → `.devtools/gitlab/`
   - Move `.github/` → `.devtools/github/`

3. Update references in documentation and scripts:
   - Update paths in CI/CD configuration
   - Update deployment scripts
   - Update development documentation

### Step 2: Standardize Source Code Organization

1. Complete core transition:
   - Move remaining modules from `/src` to `/src/core` following the core transition pattern
   - Update module exports in `lib.rs`
   - Update import paths throughout the codebase

2. Standardize module structure:
   - Ensure each module has a well-defined `mod.rs` or appropriate structure
   - Follow consistent naming conventions
   - Establish clear boundaries between modules

3. Update references:
   - Fix import paths throughout the codebase
   - Update documentation references
   - Update test imports

### Step 3: Improve Documentation Structure

1. Create documentation hierarchy:
   - `docs/guides/` - User guides and tutorials
   - `docs/reference/` - API and technical reference
   - `docs/architecture/` - Architecture documentation
   - `docs/roadmaps/` - Development roadmaps
   - `docs/contributing/` - Contribution guidelines

2. Standardize documentation format:
   - Ensure consistent Markdown formatting
   - Add metadata headers to all documentation files
   - Create templates for different types of documentation

3. Create documentation index:
   - Add a main index file
   - Add section-specific index files
   - Ensure cross-linking between documentation files

### Step 4: Standardize Testing Structure

1. Consolidate test directories:
   - Merge `test/` and `tests/` into a single `tests/` directory
   - Organize tests by module type (unit, integration, e2e)

2. Standardize test naming:
   - Adopt consistent test file naming conventions
   - Ensure test module structure matches source code structure

3. Update testing infrastructure:
   - Update test runner configuration
   - Ensure coverage tools work with new structure
   - Update test documentation

### Step 5: Optimize Build Configuration

1. Standardize environment configuration:
   - Consolidate environment-specific configuration
   - Create clear separation between default and environment-specific settings

2. Optimize build scripts:
   - Update build scripts for different environments
   - Ensure consistent behavior across environments
   - Optimize for CI/CD integration

### Step 6: Clean Up Generated Code Management

**Status:** Completed

Improve handling of generated code:

- [x] Establish clear guidelines for generated code
- [x] Update generation scripts to follow the new structure
- [x] Document the code generation process
- [x] Add appropriate gitignore patterns for generated files

### 7. Create Developer Tooling for Navigation

**Status:** In Progress

In this step, we have created tools to help developers navigate the restructured codebase:

- [x] Create project maps or diagrams
  - Created module dependencies diagram showing core relationships
  - Added request flow sequence diagram
  - Added clean architecture visualization
  - Added core module structure diagram

- [x] Update IDE configuration files
  - Added VS Code launch configurations for debugging
  - Added tasks.json with common development tasks
  - Added settings.json with optimized settings for Rust development
  - Added recommended extensions

- [x] Create helper scripts for common tasks
  - Created navigate.sh script for finding code components
  - Created verify-structure.sh script to validate project structure

- [x] Update onboarding documentation for new developers
  - Added comprehensive onboarding guide
  - Documented development workflow
  - Provided instructions for setting up the development environment
  - Explained the architecture and key components

- [ ] Consolidate project structure documentation
  - Created recommendation document for consolidation
  - Identified redundant documentation
  - Added task to roadmap

## Success Criteria

The restructuring will be considered successful when:

1. The root directory contains only essential files and directories
2. All code follows a consistent organizational pattern
3. Documentation is well-structured and easy to navigate
4. Tests are organized in a logical and consistent manner
5. Build and deployment processes work seamlessly with the new structure
6. New developers can quickly understand and navigate the codebase
7. The README is prominently visible and contains clear getting-started instructions

## Impact and Benefits

- **Improved Developer Experience**: Clearer organization makes it easier to find and modify code
- **Better Onboarding**: New developers can more quickly understand the project structure
- **Enhanced Maintainability**: Consistent patterns make maintenance easier
- **Reduced Cognitive Load**: Simplified structure reduces the mental effort needed to navigate the codebase
- **Better GitHub Presentation**: Simplified root directory makes the README more prominent
- **Support for Core Transition**: Aligns with the ongoing core transition initiative 