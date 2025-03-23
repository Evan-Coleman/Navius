#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

echo "Project Structure Improvement Script"
echo "===================================="
echo ""

# Part 1: Fix Import Patterns
echo "Fixing Import Patterns..."
echo "------------------------"

# Find all Rust files in the src directory
find_rust_files() {
    find src -name "*.rs" -type f | grep -v "target/" 
}

# Function to fix imports in a single file
fix_imports() {
    local file="$1"
    echo "Processing $file"
    
    # Get the correct import patterns
    if [[ "$file" == src/core/* ]]; then
        # For core files, imports from other core modules should use crate::core::
        sed -i '' -E 's/use crate::(metrics|repository|api|error|auth|reliability|utils|models|services|config|cache|database|handlers)/use crate::core::\1/g' "$file"
    elif [[ "$file" == src/app/* ]]; then
        # For app files, imports should generally use crate::app:: or crate::core::
        # We won't automatically change these to avoid breaking things
        grep -E "use crate::(metrics|repository|api|error|auth|reliability|utils|models|services|config|cache|database|handlers)" "$file" | grep -v "use crate::(core|app)::" > /dev/null && echo "  Warning: $file may have direct imports from root modules" || true
    fi
}

# Check all Rust files for incorrect import patterns
for file in $(find_rust_files); do
    fix_imports "$file"
done

echo ""
echo "Import pattern fixes completed."
echo ""

# Part 2: Check File Naming Conventions
echo "Checking File Naming Conventions..."
echo "----------------------------------"

# Find files that don't follow snake_case convention
check_file_names() {
    echo "Looking for files that might not follow snake_case convention..."
    find src -type f -name "*.rs" | grep -E "[A-Z]" || echo "No files found with uppercase characters in name."
    
    # Look for CamelCase directories
    echo ""
    echo "Looking for directories that might not follow snake_case convention..."
    find src -type d | grep -E "[A-Z]" || echo "No directories found with uppercase characters in name."
}

check_file_names

echo ""
echo "Script completed."
echo ""
echo "Next steps:"
echo "1. Review any import warnings and fix manually if needed"
echo "2. Rename any files that don't follow snake_case convention"
echo "3. Run tests to ensure nothing was broken" 