#!/bin/bash

# run-tests.sh - Tests the functionality of documentation tools
#
# This script runs a series of tests to verify that the documentation
# tools are working correctly.

# Script directory - used for relative path resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BASE_DIR="/Users/goblin/dev/git/navius"
VERBOSE=false
TEST_DATA_DIR="${SCRIPT_DIR}/test-data"
ALL_TESTS=true
TEST_FILTER=""

# Process command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose)
      VERBOSE=true
      shift
      ;;
    --test)
      ALL_TESTS=false
      TEST_FILTER="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --verbose      Show detailed test output"
      echo "  --test TEST    Run only the specified test"
      echo "  --help         Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Function to log messages
log() {
  if [[ "${VERBOSE}" == "true" ]]; then
    echo "$1"
  fi
}

# Function to set up test environment
setup_test_environment() {
  # Ensure the test-data directory exists
  if [[ ! -d "${TEST_DATA_DIR}" ]]; then
    log "Creating test data directory: ${TEST_DATA_DIR}"
    mkdir -p "${TEST_DATA_DIR}"
  fi
  
  # Create subdirectories
  mkdir -p "${TEST_DATA_DIR}/valid"
  mkdir -p "${TEST_DATA_DIR}/broken-links"
  mkdir -p "${TEST_DATA_DIR}/missing-frontmatter"
  mkdir -p "${TEST_DATA_DIR}/duplicate-sections"
  
  # Create a valid markdown file
  cat > "${TEST_DATA_DIR}/valid/test-document.md" << 'EOF'
---
title: "Test Document"
description: "This is a valid test document"
created_at: "2023-03-28"
updated_at: "2023-03-28"
tags: ["test", "valid"]
---

# Test Document

## Introduction

This is a test document with valid frontmatter and structure.

## Examples

Here is a code example:

```javascript
console.log("Hello, world!");
```

## Reference

For more information, see the [API Reference](../reference.md).

## Conclusion

This document is complete and valid.
EOF
  
  # Create a markdown file with broken links
  cat > "${TEST_DATA_DIR}/broken-links/broken-links.md" << 'EOF'
---
title: "Document with Broken Links"
description: "This document contains broken links for testing"
created_at: "2023-03-28"
updated_at: "2023-03-28"
tags: ["test", "broken-links"]
---

# Document with Broken Links

## Introduction

This document contains broken links for testing link fixing tools.

## Links

Here are some broken links:

- [Non-existent Document](./non-existent.md)
- [Missing File](../missing/file.md)
- [Incorrect Case](./TEST-document.md)
- [Missing Extension](../valid/test-document)

## Conclusion

This document is used for testing link fixing tools.
EOF
  
  # Create a markdown file with missing frontmatter
  cat > "${TEST_DATA_DIR}/missing-frontmatter/no-frontmatter.md" << 'EOF'
# Document Without Frontmatter

This document is missing frontmatter and should be fixed by the frontmatter fixing tool.

## Content

This is the content of the document.

## Conclusion

The end of the document.
EOF
  
  # Create a markdown file with duplicate sections
  cat > "${TEST_DATA_DIR}/duplicate-sections/duplicate-sections.md" << 'EOF'
---
title: "Document with Duplicate Sections"
description: "This document contains duplicate sections for testing"
created_at: "2023-03-28"
updated_at: "2023-03-28"
tags: ["test", "duplicate-sections"]
---

# Document with Duplicate Sections

## Introduction

This document contains duplicate sections for testing section fixing tools.

## Content

This is some content.

## Introduction

This is a duplicate introduction section that should be removed.

## Content

This is duplicate content that should be removed.

## Conclusion

This is the conclusion.
EOF
}

# Function to clean up test environment
cleanup_test_environment() {
  if [[ -d "${TEST_DATA_DIR}" && "${VERBOSE}" != "true" ]]; then
    log "Cleaning up test data directory: ${TEST_DATA_DIR}"
    rm -rf "${TEST_DATA_DIR}"
  else
    log "Keeping test data directory for inspection: ${TEST_DATA_DIR}"
  fi
}

# Function to run a test
run_test() {
  local test_name="$1"
  local command="$2"
  local expected_status="$3"
  
  if [[ "${ALL_TESTS}" == "false" && "${TEST_FILTER}" != "${test_name}" ]]; then
    return
  fi
  
  echo "Running test: ${test_name}"
  log "Command: ${command}"
  
  # Run the command and capture its output and status
  local output
  output=$(eval "${command}" 2>&1)
  local status=$?
  
  # Check if the status matches the expected status
  if [[ "${status}" -eq "${expected_status}" ]]; then
    echo "✅ PASS: ${test_name}"
  else
    echo "❌ FAIL: ${test_name} (expected status: ${expected_status}, got: ${status})"
    echo "Output:"
    echo "${output}"
  fi
  
  # Print output if verbose
  if [[ "${VERBOSE}" == "true" ]]; then
    echo "Output:"
    echo "${output}"
  fi
  
  echo ""
}

# Function to run tests for the setup-environment.sh script
test_setup_environment() {
  run_test "setup-environment" "${SCRIPT_DIR}/setup-environment.sh" 0
}

# Function to run tests for the fix-links.sh script
test_fix_links() {
  # Test with a valid file (should pass)
  run_test "fix-links-valid" "${SCRIPT_DIR}/fix-links.sh --dir ${TEST_DATA_DIR}/valid --dry-run" 0
  
  # Test with a file containing broken links (should identify the broken links)
  run_test "fix-links-broken" "${SCRIPT_DIR}/fix-links.sh --dir ${TEST_DATA_DIR}/broken-links --dry-run" 0
}

# Function to run tests for the simple-batch-validate.sh script
test_simple_batch_validate() {
  # Test with a valid directory (should pass)
  run_test "simple-batch-validate-valid" "${SCRIPT_DIR}/simple-batch-validate.sh ${TEST_DATA_DIR}/valid ${TEST_DATA_DIR}/valid-report.md" 0
  
  # Test with a directory containing issues (should identify issues)
  run_test "simple-batch-validate-issues" "${SCRIPT_DIR}/simple-batch-validate.sh ${TEST_DATA_DIR} ${TEST_DATA_DIR}/issues-report.md" 0
}

# Function to run tests for the run-daily-fixes.sh script
test_run_daily_fixes() {
  # Test with dry-run mode (should simulate running but not make changes)
  run_test "run-daily-fixes-dry-run" "${SCRIPT_DIR}/run-daily-fixes.sh --day saturday --dry-run" 0
}

# Function to run tests for the analyze-fix-logs.sh script
test_analyze_fix_logs() {
  # Create a test log file
  mkdir -p "${SCRIPT_DIR}/logs"
  local log_file="${SCRIPT_DIR}/logs/fix-links-20230328.log"
  
  cat > "${log_file}" << 'EOF'
Processing directory: /Users/goblin/dev/git/navius/11newdocs11/01_getting_started
BROKEN LINK: ./installation.md
FIXED LINK: ./installation.md -> ../01_getting_started/installation.md
BROKEN LINK: ../reference/api.md
UNFIXABLE LINK: ../reference/api.md
BROKEN LINK: ./configuration.md
FIXED LINK: ./configuration.md -> ../01_getting_started/configuration.md
EOF
  
  # Test the analyze-fix-logs.sh script
  run_test "analyze-fix-logs" "${SCRIPT_DIR}/analyze-fix-logs.sh" 0
}

# Main execution
echo "Starting documentation tool tests..."
setup_test_environment

# Run tests
test_setup_environment
test_fix_links
test_simple_batch_validate
test_run_daily_fixes
test_analyze_fix_logs

cleanup_test_environment
echo "All tests completed."

exit 0 