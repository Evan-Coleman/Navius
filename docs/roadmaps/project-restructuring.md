# Project Restructuring Roadmap

**Status:** Not Started  
**Updated At:** March 22, 2025
**Priority:** High  

## Overview

This roadmap outlines a plan to iteratively restructure the Navius project to improve organization, maintainability, and developer experience. The main goals are:

1. Simplify the root directory to make the README more visible
2. Organize code following clean architecture principles
3. Standardize file locations and naming conventions
4. Improve discoverability for new developers
5. Support the core transition initiative

## Step 1: Consolidate Top-Level Directories

**Status:** Not Started

Consolidate non-essential top-level directories to reduce clutter:

- [ ] Create a `.devtools` directory and move development-related files there
- [ ] Consolidate script files
- [ ] Organize CI/CD configuration files
- [ ] Update documentation to reflect new structure

## Step 2: Standardize Source Code Organization

**Status:** Not Started

Complete the core transition initiative and standardize the source code structure:

- [ ] Finish moving functionality to `/src/core` following the core transition pattern
- [ ] Establish clear naming conventions for all modules
- [ ] Reorganize remaining `/src` directories to follow a consistent pattern
- [ ] Update import paths throughout the codebase

## Step 3: Improve Documentation Structure

**Status:** Not Started

Reorganize documentation to improve discoverability:

- [ ] Create a structured documentation hierarchy
- [ ] Ensure consistent formatting across all documentation
- [ ] Update links and references throughout documentation
- [ ] Add a documentation index

## Step 4: Standardize Testing Structure

**Status:** Not Started

Improve test organization and consistency:

- [ ] Consolidate test directories
- [ ] Establish consistent test naming conventions
- [ ] Update testing roadmap to reflect new structure
- [ ] Ensure test coverage metrics work with the new structure

## Step 5: Optimize Build Configuration

**Status:** Not Started

Improve build process and configuration:

- [ ] Standardize environment configuration files
- [ ] Optimize build scripts for different environments
- [ ] Update deployment scripts to work with new structure
- [ ] Ensure CI/CD pipelines are compatible with new structure

## Step 6: Clean Up Generated Code Management

**Status:** Not Started

Improve handling of generated code:

- [ ] Establish clear guidelines for generated code
- [ ] Update generation scripts to follow the new structure
- [ ] Document the code generation process
- [ ] Add appropriate gitignore patterns for generated files

## Step 7: Create Developer Tooling for Navigation

**Status:** Not Started

Improve tooling to help developers navigate the restructured codebase:

- [ ] Create project maps or diagrams
- [ ] Update IDE configuration files
- [ ] Create helper scripts for common tasks
- [ ] Update onboarding documentation for new developers

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

1. Establish generated code guidelines:
   - Define when code generation is appropriate
   - Document code generation process
   - Set standards for using generated code

2. Update generation scripts:
   - Ensure scripts work with new structure
   - Update paths and configuration

3. Gitignore patterns:
   - Add appropriate patterns to exclude generated files
   - Document exceptions when generated files should be committed

### Step 7: Create Developer Tooling for Navigation

1. Project maps:
   - Create visual representations of project structure
   - Document module dependencies
   - Update module boundary documentation

2. IDE configuration:
   - Update VSCode/IntelliJ configuration
   - Add helpful IDE tasks
   - Include recommended extensions

3. Helper scripts:
   - Create navigation scripts
   - Add project structure verification
   - Create code generation helpers

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