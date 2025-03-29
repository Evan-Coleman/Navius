---
title: Project Restructuring Guide
description: Prompts and instructions for executing the project restructuring roadmap
category: roadmaps
tags:
  - restructuring
  - project-organization
  - implementation
related:
  - ../completed/project-restructuring.md
  - ../completed/project-restructuring-summary.md
last_updated: March 27, 2025
version: 1.0
---

# Project Restructuring Guide

This guide provides prompts and instructions for executing the project restructuring roadmap. Each step in the roadmap can be completed by issuing specific prompts to the AI assistant.

## General Instructions

1. Work through the steps in the roadmap sequentially
2. After completing each step, verify that everything still works
3. Run tests after each major change
4. Commit changes after completing each step
5. Update the roadmap to mark steps as completed

## Step 1: Consolidate Top-Level Directories

### Initial Analysis Prompt

```
Analyze the current top-level directories and identify candidates for consolidation into the .devtools directory. 
Show me which directories and files should be moved and which should remain at the top level.
```

### Creation of .devtools Directory Prompt

```
Create a .devtools directory and implement the consolidation plan for the following directories:
1. scripts/
2. coverage/
3. .github/
4. .gitlab/ (keeping .gitlab-ci.yml at the root)
Create any necessary README files in the new locations to explain the purpose of each directory.
```

### Update References Prompt

```
Find and update all references to the moved directories in:
1. CI/CD configuration files
2. Shell scripts
3. Documentation
4. Build configurations
Ensure all paths are correct after the reorganization.
```

## Step 2: Standardize Source Code Organization

### Core Transition Analysis Prompt

```
Analyze the remaining directories in the src/ folder that haven't been moved to src/core yet.
For each directory, determine if it should be moved to core following the core transition pattern.
```

### Core Transition Implementation Prompt

```
For the [specific module] directory, implement the core transition:
1. Move the directory to src/core/
2. Update the mod.rs file in src/core/
3. Create the user-facing scaffolding in the original location
4. Update import paths throughout the codebase
5. Run tests to ensure everything still works
```

### Module Structure Standardization Prompt

```
For the [specific module] directory, standardize the module structure:
1. Ensure it has a proper mod.rs file with clear exports
2. Organize files according to our conventions
3. Ensure naming is consistent
4. Update any imports that might be affected
```

## Step 3: Improve Documentation Structure

### Documentation Analysis Prompt

```
Analyze the current documentation in the docs/ directory:
1. Identify categories of documentation (guides, reference, architecture, etc.)
2. Suggest a hierarchical structure
3. Identify inconsistencies in formatting or style
```

### Documentation Restructuring Prompt

```
Implement the documentation restructuring:
1. Create the suggested directory structure
2. Move files to appropriate directories
3. Update cross-references
4. Create index files for each section
5. Ensure formatting is consistent
```

### Documentation Update Prompt

```
Update the following documentation files to reflect the new project structure:
1. README.md
2. CONTRIBUTING.md
3. docs/DEVELOPMENT.md
4. Installation guides
Update any path references and ensure they match the new structure.
```

## Step 4: Standardize Testing Structure

### Test Consolidation Prompt

```
Consolidate the test/ and tests/ directories:
1. Analyze the current test organization
2. Create a plan for merging them into a single tests/ directory
3. Suggest a structure that mirrors the source code organization
```

### Test Structure Implementation Prompt

```
Implement the test consolidation plan:
1. Move tests to the new structure
2. Update imports and paths
3. Ensure test naming follows the conventions
4. Update test runner configuration
```

### Test Coverage Update Prompt

```
Update the test coverage configuration:
1. Update coverage tool configuration
2. Ensure coverage reports reflect the new structure
3. Update testing roadmap documentation
```

## Step 5: Optimize Build Configuration

### Build Configuration Analysis Prompt

```
Analyze the current build configuration:
1. Identify environment-specific configurations
2. Examine build scripts
3. Review deployment process
4. Suggest standardization approaches
```

### Build Configuration Implementation Prompt

```
Implement the build configuration standardization:
1. Update environment configuration files
2. Optimize build scripts
3. Ensure CI/CD compatibility
4. Test builds in different environments
```

## Step 6: Clean Up Generated Code Management

### Generated Code Analysis Prompt

```
Analyze how generated code is currently managed:
1. Identify generation scripts
2. Examine where generated code is stored
3. Review .gitignore patterns
4. Suggest improvements to the process
```

### Generated Code Implementation Prompt

```
Implement the generated code management improvements:
1. Update generation scripts
2. Standardize generated code location
3. Update .gitignore patterns
4. Document the generation process
```

## Step 7: Create Developer Tooling for Navigation

### Navigation Tools Design Prompt

```
Design developer navigation tools:
1. Suggest project maps or diagrams
2. Recommend IDE configurations
3. Propose helper scripts
4. Draft improved onboarding documentation
```

### Navigation Tools Implementation Prompt

```
Implement the developer navigation tools:
1. Create the suggested project maps
2. Add IDE configuration files
3. Develop navigation helper scripts
4. Update onboarding documentation
```

## Progress Tracking Prompt

After completing each major step:

```
Update the project-restructuring.md roadmap file to mark the following items as completed:
1. [Specific step]
2. [Specific substep]
Also update the "Updated At" date to today's date.
```

## Final Verification Prompt

After completing all steps:

```
Perform a final verification of the project restructuring:
1. Check that all steps in the roadmap are completed
2. Verify the root directory is now cleaner
3. Ensure all tests pass
4. Verify documentation is accurate
5. Check that build and deployment processes work
6. Create a summary of the changes made
```

## Best Practices for Every Step

1. Always run tests after making changes
2. Make incremental changes and verify each step
3. Keep track of any issues encountered
4. Update documentation as you go
5. Communicate progress clearly
6. If a change doesn't work as expected, be prepared to roll back and try a different approach 

## Related Documents
- [Project Structure Roadmap](../../completed/11_project_structure_future_improvements.md) - Future improvements
- [Documentation Overhaul](../../12_document_overhaul.md) - Documentation plans

