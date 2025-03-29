---
title: Documentation Scripts Fix Instructions
description: Technical instructions for fixing and enhancing documentation validation and improvement scripts
category: instructions
tags:
  - documentation
  - tools
  - scripts
  - maintenance
  - shell
related:
  - 31_documentation-script-fixes.md
  - 30_documentation-reorganization-instructions.md
  - ../05_reference/standards/documentation-standards.md
last_updated: March 27, 2025
version: 1.0
status: not started
---

# Documentation Scripts Fix Instructions

## Overview

This document provides detailed technical instructions for fixing the documentation validation and improvement scripts located in `.devtools/scripts/doc-overhaul/`. It serves as a companion to the [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md).

## Script Inventory

The following scripts need to be fixed:

| Script Name | Purpose | Current Status |
|-------------|---------|----------------|
| `generate_report.sh` | Generates quality report for documentation | ❌ Not working |
| `comprehensive_test.sh` | Runs all documentation tests | ❌ Not working |
| `fix_frontmatter.sh` | Validates and fixes frontmatter in markdown files | ❌ Not working |
| `fix_links.sh` | Finds and fixes broken internal links | ❌ Not working |
| `add_sections.sh` | Adds missing sections to documents | ❌ Not working |
| `improve_docs.sh` | Main script for documentation improvement workflow | ❌ Not working |

## Development Environment Setup

Before starting the fix process, set up your development environment:

1. **Install Dependencies**:
   ```bash
   # Install markdownlint
   npm install -g markdownlint-cli

   # Install other required tools
   brew install shellcheck jq yq
   ```

2. **Clone Test Repository**:
   ```bash
   # Create a test branch
   git checkout -b doc-script-fixes
   ```

3. **Create Test Documentation Set**:
   ```bash
   # Create a test directory
   mkdir -p .devtools/test/docs
   
   # Copy a subset of documentation for testing
   cp -r docs/new/01_getting_started .devtools/test/docs/
   ```

## Diagnostic Procedure

For each script, follow this diagnostic procedure:

1. **Run with debugging enabled**:
   ```bash
   bash -x .devtools/scripts/doc-overhaul/SCRIPT_NAME.sh
   ```

2. **Capture error output**:
   ```bash
   bash -x .devtools/scripts/doc-overhaul/SCRIPT_NAME.sh 2> error_log.txt
   ```

3. **Analyze common patterns**:
   - Look for shell compatibility issues (bash vs zsh)
   - Check for array declaration and usage problems
   - Identify command substitution issues
   - Note path resolution problems

4. **Document specific issues in a structured format**:
   ```markdown
   ## Script: SCRIPT_NAME.sh
   
   ### Issues Found:
   1. Line XX: [DESCRIPTION OF ISSUE]
   2. Line YY: [DESCRIPTION OF ISSUE]
   
   ### Root Causes:
   - [ROOT CAUSE 1]
   - [ROOT CAUSE 2]
   
   ### Proposed Fixes:
   - [FIX 1]
   - [FIX 2]
   ```

## Fix Implementation Strategy

### 1. Fix Common Utilities First

Several scripts share common utility functions. These should be fixed first:

1. **Create a shell-agnostic utility library**:
   ```bash
   # Create a new utilities file
   touch .devtools/scripts/doc-overhaul/utils.sh
   ```

2. **Implement cross-shell compatible functions**:
   ```bash
   #!/bin/bash
   # utils.sh - Cross-shell compatible utility functions
   
   # Detect shell environment
   detect_shell() {
     if [ -n "$ZSH_VERSION" ]; then
       echo "zsh"
     elif [ -n "$BASH_VERSION" ]; then
       echo "bash"
     else
       echo "unknown"
     fi
   }
   
   # Safe array declaration and use
   # Usage: iterate_array "item1 item2 item3" callback_function
   iterate_array() {
     local items="$1"
     local callback="$2"
     
     for item in $items; do
       $callback "$item"
     done
   }
   
   # Path normalization
   normalize_path() {
     local path="$1"
     echo "$(cd "$(dirname "$path")" && pwd)/$(basename "$path")"
   }
   
   # Safe command execution with error handling
   safe_exec() {
     local cmd="$1"
     local error_msg="$2"
     
     if ! eval "$cmd"; then
       echo "Error: $error_msg" >&2
       return 1
     fi
     return 0
   }
   ```

3. **Update scripts to use the utility library**:
   ```bash
   # Add to the top of each script
   source "$(dirname "$0")/utils.sh"
   ```

### 2. Fix Individual Scripts

For each script, follow these steps:

#### generate_report.sh

1. **Fix markdownlint detection**:
   ```bash
   # Replace direct command check with function
   check_markdownlint() {
     if command -v markdownlint >/dev/null 2>&1; then
       return 0
     elif command -v markdownlint-cli >/dev/null 2>&1; then
       return 0
     else
       return 1
     fi
   }
   ```

2. **Fix expression syntax errors**:
   ```bash
   # Replace
   if [ $count -eq 0 ]; then
   
   # With
   if [ "$count" -eq 0 ]; then
   ```

3. **Improve variable expansion**:
   ```bash
   # Replace
   for file in $files; do
   
   # With
   IFS=$'\n'
   for file in $files; do
   unset IFS
   ```

#### comprehensive_test.sh

1. **Fix declare command compatibility**:
   ```bash
   # Replace
   declare -A results
   
   # With
   if [ "$(detect_shell)" = "zsh" ]; then
     typeset -A results
   else
     declare -A results
   fi
   ```

2. **Fix CSV output generation**:
   ```bash
   # Create a shell-agnostic CSV generation function
   generate_csv() {
     local output_file="$1"
     local header="$2"
     local data="$3"
     
     echo "$header" > "$output_file"
     echo "$data" >> "$output_file"
   }
   ```

#### fix_frontmatter.sh

1. **Fix file count reporting**:
   ```bash
   # Replace direct globbing with a function
   count_files() {
     local pattern="$1"
     local count=0
     
     if [ "$(detect_shell)" = "zsh" ]; then
       count=$(ls -1 $pattern 2>/dev/null | wc -l)
     else
       count=$(find . -name "$pattern" -type f | wc -l)
     fi
     
     echo "$count"
   }
   ```

2. **Improve path handling**:
   ```bash
   # Use the normalize_path function from utils.sh
   docs_dir=$(normalize_path "${docs_dir:-./docs}")
   ```

#### fix_links.sh

1. **Update link pattern detection**:
   ```bash
   # Create a robust link pattern function
   find_markdown_links() {
     local file="$1"
     grep -o '\[.*\](.*\.md[^)]*)'  "$file" || true
   }
   ```

2. **Improve link fixing algorithm**:
   ```bash
   # Create a path resolver function
   resolve_relative_path() {
     local source_file="$1"
     local target_link="$2"
     local source_dir
     
     source_dir="$(dirname "$source_file")"
     python3 -c "import os.path; print(os.path.normpath(os.path.join('$source_dir', '$target_link')))"
   }
   ```

## Testing Procedure

After implementing fixes, test each script thoroughly:

1. **Unit Testing**:
   ```bash
   # Create a test runner
   touch .devtools/scripts/doc-overhaul/test_runner.sh
   
   # Implement test cases
   for script in .devtools/scripts/doc-overhaul/*.sh; do
     if [[ "$script" != *test* ]]; then
       echo "Testing $script..."
       bash "$script" -t
     fi
   done
   ```

2. **Integration Testing**:
   ```bash
   # Test the main workflow
   bash .devtools/scripts/doc-overhaul/improve_docs.sh --test
   ```

3. **Cross-Environment Testing**:
   ```bash
   # Test in bash
   bash .devtools/scripts/doc-overhaul/comprehensive_test.sh
   
   # Test in zsh
   zsh .devtools/scripts/doc-overhaul/comprehensive_test.sh
   ```

## Documentation Updates

Update the script documentation with the following:

1. **Usage Instructions**:
   ```markdown
   ## Usage
   
   ```bash
   # Generate a documentation quality report
   .devtools/scripts/doc-overhaul/generate_report.sh [OPTIONS]
   
   # Options:
   #   --help, -h     Show this help message
   #   --verbose, -v  Show verbose output
   #   --csv          Output results in CSV format
   ```
   ```

2. **Common Errors and Solutions**:
   ```markdown
   ## Troubleshooting
   
   ### Error: markdownlint not found
   
   **Solution**: Install markdownlint-cli using npm: `npm install -g markdownlint-cli`
   
   ### Error: Invalid option to declare
   
   **Solution**: This script requires bash v4.2+. Try running with: `bash .devtools/scripts/doc-overhaul/script.sh`
   ```

3. **Examples**:
   ```markdown
   ## Examples
   
   ```bash
   # Generate a full documentation report
   .devtools/scripts/doc-overhaul/generate_report.sh --verbose
   
   # Fix frontmatter in all markdown files
   .devtools/scripts/doc-overhaul/fix_frontmatter.sh docs/
   
   # Fix internal links in the documentation
   .devtools/scripts/doc-overhaul/fix_links.sh docs/
   ```
   ```

## Directory Structure Support

To support the new numbered directory structure:

1. **Update path handling in all scripts**:
   ```bash
   # Add path normalization function
   normalize_numbered_path() {
     local path="$1"
     # Extract section number pattern (##_) from path
     local section_pattern=$(echo "$path" | grep -o '[0-9]\+_[^/]*' || echo "")
     echo "$section_pattern"
   }
   ```

2. **Update templates**:
   ```bash
   # Update section templates to match new structure
   update_templates() {
     local template_dir=".devtools/scripts/doc-overhaul/templates"
     # Update template paths
     sed -i.bak 's|docs/|docs/new/|g' "$template_dir"/*.md
     # Update link formats
     sed -i.bak 's|(\.\./\([^)]*\))|(\.\./[0-9]*_\1)|g' "$template_dir"/*.md
   }
   ```

## CI/CD Integration

Create GitHub Actions workflow:

1. **Create workflow file**:
   ```yaml
   # .github/workflows/doc-quality.yml
   name: Documentation Quality Check
   
   on:
     push:
       paths:
         - 'docs/**'
         - '.devtools/scripts/doc-overhaul/**'
     pull_request:
       paths:
         - 'docs/**'
   
   jobs:
     doc-quality:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v2
         
         - name: Set up Node.js
           uses: actions/setup-node@v2
           with:
             node-version: '14'
             
         - name: Install dependencies
           run: |
             npm install -g markdownlint-cli
             
         - name: Run documentation quality checks
           run: |
             bash .devtools/scripts/doc-overhaul/comprehensive_test.sh --ci
             
         - name: Upload quality report
           uses: actions/upload-artifact@v2
           with:
             name: doc-quality-report
             path: doc-quality-report.json
   ```

## Success Verification

To verify that the script fixes are successful:

1. **Functional Testing**:
   - All scripts run without errors
   - Scripts produce expected output
   - Scripts work with the new directory structure

2. **Cross-Platform Testing**:
   - Scripts work on macOS (zsh)
   - Scripts work on Linux (bash)
   - Scripts handle different environments correctly

3. **Documentation Coverage**:
   - All scripts have clear usage instructions
   - Common errors and solutions are documented
   - Examples are provided

## Next Steps

After successfully fixing the scripts:

1. **Create a PR with your changes**:
   ```bash
   git add .devtools/scripts/doc-overhaul/
   git commit -m "Fix documentation scripts for cross-shell compatibility"
   git push -u origin doc-script-fixes
   ```

2. **Document improvements in the changelog**:
   ```markdown
   ## [Unreleased]
   
   ### Fixed
   - Documentation scripts now work in both bash and zsh environments
   - Fixed path handling in documentation scripts
   - Improved error reporting in all documentation tools
   ```

3. **Update the documentation roadmap**:
   ```bash
   # Update the roadmap with script fix completion
   sed -i.bak 's/status: not started/status: completed/g' docs/new/98_roadmaps/31_documentation-script-fixes.md
   ```

## Related Documents

- [Documentation Scripts Fix Roadmap](31_documentation-script-fixes.md)
- [Documentation Reorganization Instructions](30_documentation-reorganization-instructions.md)
- [Documentation Standards](../05_reference/standards/documentation-standards.md) 