#!/usr/bin/env bash

# code-example-fixer.sh
# Script to automatically fix common issues in code examples extracted by the code-example-verifier.sh
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
VERIFICATION_DIR="target/code-verification"
EXAMPLES_DIR="$VERIFICATION_DIR/examples"
FIXED_DIR="$VERIFICATION_DIR/fixed"
COMMON_IMPORTS="
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::io::{self, Read, Write};
use navius::core::error::AppError;
use navius::app::Application;
use navius::core::config::Config;
use navius::core::services::ServiceRegistry;
use navius::app::api::Router;
use axum::{
    routing::{get, post, put, delete},
    response::IntoResponse,
    extract::{Path, Json, State},
    http::StatusCode,
};
"

# Create output directory
mkdir -p "$FIXED_DIR"

# Function to process a document's extracted examples
process_document() {
    local category="$1"
    local document="$2"
    local examples_dir="$EXAMPLES_DIR/$category/$document"
    local tracking_file="$examples_dir/examples.csv"
    local fixed_dir="$FIXED_DIR/$category/$document"
    local fixed_tracking="$fixed_dir/fixes.csv"
    
    if [[ ! -f "$tracking_file" ]]; then
        echo "No examples found for $category/$document"
        return 0
    fi
    
    mkdir -p "$fixed_dir"
    echo "Example,OriginalFile,FixedFile,IssuesFound,IssuesFixed,Status" > "$fixed_tracking"
    
    echo "Processing examples from $category/$document..."
    
    # Get failing examples from tracking file
    grep ",Fails$" "$tracking_file" 2>/dev/null | while IFS=, read -r example_id start_line end_line file issues status; do
        # Clean up issues string
        issues="${issues//\"}"
        
        # Get original example file
        local orig_file=$(echo "$file" | sed 's/"//g')
        local example_name=$(basename "$orig_file")
        local fixed_file="$fixed_dir/$example_name"
        local issues_fixed=""
        
        echo "Fixing example $example_id (${orig_file})..."
        
        # Copy the original file to start with
        cp "$orig_file" "$fixed_file"
        
        # Apply fixes based on detected issues
        if [[ "$issues" == *"Missing imports"* ]]; then
            echo "- Adding missing imports..."
            # Add common imports at the top of the file after the first line (comment)
            sed -i '1a\'$'\n'"$COMMON_IMPORTS" "$fixed_file"
            issues_fixed="${issues_fixed}Added imports;"
        fi
        
        if [[ "$issues" == *"Code fragment without declarations"* ]]; then
            echo "- Wrapping code in function..."
            # If the code doesn't have a function declaration, add a wrapper function
            if ! grep -q "fn " "$fixed_file"; then
                local content=$(sed '1,/DEFAULT_IMPORTS/d' "$fixed_file")
                # Create a new file with the wrapper
                echo "// Example $example_id from $document" > "${fixed_file}.tmp"
                echo "$COMMON_IMPORTS" >> "${fixed_file}.tmp"
                echo "
// Added function wrapper
fn example_wrapper() {
    // This is a wrapper function to make code fragments compilable
    #[allow(unused_variables, dead_code, unused_imports)]
    {
$content
    }
}

// Main function for standalone examples
#[cfg(test)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only added for standalone examples that need a main function
    Ok(())
}
" >> "${fixed_file}.tmp"
                mv "${fixed_file}.tmp" "$fixed_file"
                issues_fixed="${issues_fixed}Added function wrapper;"
            fi
        fi
        
        if [[ "$issues" == *"Uses unwrap or expect"* ]]; then
            echo "- Improving error handling..."
            # Replace unwrap with proper error handling
            sed -i 's/\.unwrap()/\.map_err(|e| AppError::internal_server_error(e.to_string()))?/g' "$fixed_file"
            sed -i 's/\.expect([^)]*)/\.map_err(|e| AppError::internal_server_error(e.to_string()))?/g' "$fixed_file"
            issues_fixed="${issues_fixed}Improved error handling;"
        fi
        
        if [[ "$issues" == *"Error handling issues"* ]]; then
            echo "- Adding Result return type..."
            # Add Result return type to functions that use Err but don't declare it
            sed -i 's/fn \([a-zA-Z0-9_]*\)(.*) {/fn \1\(...\) -> Result<(), Box<dyn Error>> {/g' "$fixed_file"
            issues_fixed="${issues_fixed}Added Result return type;"
        fi
        
        # Test if our fixes worked by trying to compile
        local new_status="Fails"
        if rustc --edition=2021 -o /dev/null "$fixed_file" 2>/dev/null; then
            new_status="Compiles"
            echo "✓ Fixed successfully!"
        else
            echo "✗ Still has issues after fixing"
        fi
        
        # Record the fix in tracking file
        echo "$example_id,\"$orig_file\",\"$fixed_file\",\"$issues\",\"$issues_fixed\",\"$new_status\"" >> "$fixed_tracking"
    done
    
    # Summary
    local total=$(grep -c "," "$fixed_tracking" 2>/dev/null || echo 0)
    if [[ $total -eq 0 ]]; then
        echo "No examples needed fixing for $category/$document"
        return 0
    fi
    
    local fixed=$(grep ",\"Compiles\"$" "$fixed_tracking" 2>/dev/null | wc -l | tr -d ' ')
    local still_failing=$(grep ",\"Fails\"$" "$fixed_tracking" 2>/dev/null | wc -l | tr -d ' ')
    
    echo "Summary for $category/$document:"
    echo "- Total examples processed: $((total-1))"
    echo "- Fixed successfully: $fixed"
    echo "- Still failing: $still_failing"
    echo "- Success rate: $(( fixed * 100 / (total-1) ))%"
    
    return 0
}

# Function to generate a markdown diff for a fixed example
generate_diff() {
    local category="$1"
    local document="$2"
    local example_id="$3"
    local fixed_dir="$FIXED_DIR/$category/$document"
    local fixed_tracking="$fixed_dir/fixes.csv"
    
    if [[ ! -f "$fixed_tracking" ]]; then
        echo "No fixes found for $category/$document"
        return 1
    fi
    
    # Get the example data
    local example_data=$(grep "^$example_id," "$fixed_tracking")
    if [[ -z "$example_data" ]]; then
        echo "Example $example_id not found in $category/$document"
        return 1
    fi
    
    IFS=, read -r _ orig_file fixed_file issues fixed_issues status <<< "$example_data"
    
    # Clean up the file paths
    orig_file=$(echo "$orig_file" | sed 's/"//g')
    fixed_file=$(echo "$fixed_file" | sed 's/"//g')
    
    # Generate a markdown diff
    local diff_file="$fixed_dir/example_${example_id}_diff.md"
    
    echo "# Code Example Fix for Example $example_id

## Document Path
\`$category/$document\`

## Issues Found
$issues

## Fixes Applied
$fixed_issues

## Status
**$status**

## Original Code
\`\`\`rust
$(cat "$orig_file")
\`\`\`

## Fixed Code
\`\`\`rust
$(cat "$fixed_file")
\`\`\`

## Key Changes
" > "$diff_file"

    # Add key changes section
    if [[ "$fixed_issues" == *"Added imports"* ]]; then
        echo "### Added Missing Imports
- Standard library imports were added
- Navius-specific imports were added
" >> "$diff_file"
    fi
    
    if [[ "$fixed_issues" == *"Added function wrapper"* ]]; then
        echo "### Added Function Wrapper
- Code fragment was wrapped in \`example_wrapper()\` function
- Added \`#[allow(unused_variables, dead_code, unused_imports)]\` to suppress warnings
- Added \`main()\` function for standalone execution
" >> "$diff_file"
    fi
    
    if [[ "$fixed_issues" == *"Improved error handling"* ]]; then
        echo "### Improved Error Handling
- Replaced \`.unwrap()\` with proper error handling
- Added \`map_err\` to convert errors to \`AppError\`
" >> "$diff_file"
    fi
    
    if [[ "$fixed_issues" == *"Added Result return type"* ]]; then
        echo "### Added Result Return Type
- Added \`-> Result<(), Box<dyn Error>>\` return type to functions
- Ensures proper error propagation
" >> "$diff_file"
    fi
    
    echo "Diff generated at $diff_file"
    return 0
}

# Function to generate a summary report
generate_summary() {
    local report_file="$FIXED_DIR/summary.md"
    
    echo "# Code Example Fixes Summary

## Overview

This report summarizes the automated fixes applied to failing code examples.

## Summary Statistics

| Category | Documents Processed | Examples Fixed | Still Failing | Success Rate |
|----------|---------------------|---------------|---------------|--------------|" > "$report_file"
    
    # Find all tracking files
    find "$FIXED_DIR" -name "fixes.csv" | while read -r tracking_file; do
        local category=$(basename "$(dirname "$(dirname "$tracking_file")")")
        local document=$(basename "$(dirname "$tracking_file")")
        
        # Get statistics
        local total=$(grep -c "," "$tracking_file" 2>/dev/null || echo 0)
        if [[ $total -le 1 ]]; then
            continue  # Skip if no examples or just header
        fi
        
        total=$((total-1))  # Subtract header
        local fixed=$(grep ",\"Compiles\"$" "$tracking_file" 2>/dev/null | wc -l | tr -d ' ')
        local still_failing=$((total - fixed))
        local success_rate=0
        if [[ $total -gt 0 ]]; then
            success_rate=$((fixed * 100 / total))
        fi
        
        echo "| $category | $document | $fixed | $still_failing | ${success_rate}% |" >> "$report_file"
    done
    
    echo "
## Common Issues and Fixes

| Issue Type | Count | Fix Applied |
|------------|-------|------------|" >> "$report_file"
    
    # Count issue types
    local missing_imports=$(grep -r "Added imports" "$FIXED_DIR" | wc -l | tr -d ' ')
    local added_wrapper=$(grep -r "Added function wrapper" "$FIXED_DIR" | wc -l | tr -d ' ')
    local improved_error=$(grep -r "Improved error handling" "$FIXED_DIR" | wc -l | tr -d ' ')
    local added_result=$(grep -r "Added Result return type" "$FIXED_DIR" | wc -l | tr -d ' ')
    
    echo "| Missing Imports | $missing_imports | Added standard and Navius-specific imports |" >> "$report_file"
    echo "| Fragment Without Function | $added_wrapper | Wrapped code in example_wrapper() function |" >> "$report_file"
    echo "| Unwrap/Expect Usage | $improved_error | Replaced with proper error handling |" >> "$report_file"
    echo "| Error Handling Issues | $added_result | Added Result return type to functions |" >> "$report_file"
    
    echo "
## Next Steps

1. **Review Fixes**: Examine the fixes for each example and confirm they maintain the original intent
2. **Manual Fixes**: Address examples that couldn't be fixed automatically
3. **Update Documentation**: Apply fixes to the original markdown files
4. **Re-run Verification**: Validate that the fixes resolve the issues
5. **Document Patterns**: Record common patterns and fixes for future reference

## Detailed Fix Reports

The following documents have fix reports available:
" >> "$report_file"
    
    # List all documents with fixes
    find "$FIXED_DIR" -name "fixes.csv" | sort | while read -r tracking_file; do
        local category=$(basename "$(dirname "$(dirname "$tracking_file")")")
        local document=$(basename "$(dirname "$tracking_file")")
        
        # Get statistics
        local total=$(grep -c "," "$tracking_file" 2>/dev/null || echo 0)
        if [[ $total -le 1 ]]; then
            continue  # Skip if no examples or just header
        fi
        
        echo "- $category/$document" >> "$report_file"
    done
    
    echo "Summary report generated at $report_file"
    return 0
}

# Function to apply fixes directly to markdown file
apply_fixes_to_markdown() {
    local doc_path="$1"
    local fixed_dir="$FIXED_DIR"
    
    if [[ ! -f "$doc_path" ]]; then
        echo "Document not found: $doc_path"
        return 1
    fi
    
    local filename=$(basename "$doc_path" .md)
    local dirname=$(dirname "$doc_path")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    local tracking_file="$EXAMPLES_DIR/$category/$filename/examples.csv"
    local fixes_file="$fixed_dir/$category/$filename/fixes.csv"
    
    if [[ ! -f "$tracking_file" || ! -f "$fixes_file" ]]; then
        echo "No fixes available for $doc_path"
        return 1
    fi
    
    # Create a backup of the original file
    local backup_file="${doc_path}.bak"
    cp "$doc_path" "$backup_file"
    
    echo "Applying fixes to $doc_path..."
    
    # Read the fixes file and apply each fixed example
    tail -n +2 "$fixes_file" | while IFS=, read -r example_id orig_file fixed_file issues fixed_issues status; do
        # Only apply fixes that actually compile
        if [[ "$status" != *"Compiles"* ]]; then
            echo "- Skipping example $example_id (still failing)"
            continue
        fi
        
        echo "- Applying fix for example $example_id..."
        
        # Read line numbers from the tracking file
        local line_data=$(grep "^$example_id," "$tracking_file")
        IFS=, read -r _ start_line end_line _ _ _ <<< "$line_data"
        
        # Clean up the file paths
        fixed_file=$(echo "$fixed_file" | sed 's/"//g')
        
        # Read the fixed code (skip the first line comment and imports)
        local fixed_code=$(grep -v "^// " "$fixed_file" | grep -v "^use " | sed 's/^/  /')
        
        # Replace the code in the markdown file
        # We need to be careful to:
        # 1. Keep the ```rust line
        # 2. Replace only inside the code block
        # 3. Keep the closing ```
        
        # This is a complex operation, so we'll use a temporary file
        local temp_file="${doc_path}.tmp"
        
        # Copy up to the start of the code block
        head -n "$start_line" "$doc_path" > "$temp_file"
        
        # Add the ```rust line
        echo '```rust' >> "$temp_file"
        
        # Add the fixed code
        echo "$fixed_code" >> "$temp_file"
        
        # Add the closing ```
        echo '```' >> "$temp_file"
        
        # Add the rest of the file after the end of the code block
        local line_count=$(wc -l < "$doc_path")
        tail -n "$((line_count - end_line))" "$doc_path" >> "$temp_file"
        
        # Replace the original file
        mv "$temp_file" "$doc_path"
    done
    
    echo "Fixes applied to $doc_path. Backup saved as $backup_file"
    return 0
}

# Main execution
echo "===== Code Example Fixer Tool ====="
echo "This tool automatically fixes common issues in Rust code examples"
echo "Part of the Phase 2 Completion Plan implementation"
echo ""

# Check if any arguments were provided
if [[ $# -eq 0 ]]; then
    # No arguments - print help
    echo "Usage:"
    echo "  $0 [options]"
    echo ""
    echo "Options:"
    echo "  --all                Process all documents with failing examples"
    echo "  --document <path>    Process a specific document"
    echo "  --category <name>    Process all documents in a category"
    echo "  --apply <path>       Apply fixes directly to the markdown file"
    echo "  --diff <cat/doc/id>  Generate a markdown diff for a specific example"
    echo "  --help               Display this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --all                     # Process all documents"
    echo "  $0 --category 04_guides      # Process all guides"
    echo "  $0 --document 04_guides/features/api-integration.md  # Process specific document"
    echo "  $0 --diff 04_guides/features/api-integration/1       # Generate diff for example 1"
    echo "  $0 --apply 04_guides/features/api-integration.md     # Apply fixes to markdown"
    exit 0
fi

# Process arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)
            echo "Processing all documents with failing examples..."
            
            # Find all directories with examples
            find "$EXAMPLES_DIR" -type d -mindepth 2 | while read -r dir; do
                if [[ -f "$dir/examples.csv" ]]; then
                    category=$(basename "$(dirname "$dir")")
                    document=$(basename "$dir")
                    
                    # Check if there are any failing examples
                    if grep -q ",Fails$" "$dir/examples.csv" 2>/dev/null; then
                        process_document "$category" "$document"
                    fi
                fi
            done
            
            # Generate summary
            generate_summary
            ;;
        
        --category)
            category="$2"
            echo "Processing all documents in category $category..."
            
            # Find all directories with examples in the specified category
            find "$EXAMPLES_DIR/$category" -type d -mindepth 1 | while read -r dir; do
                if [[ -f "$dir/examples.csv" ]]; then
                    document=$(basename "$dir")
                    
                    # Check if there are any failing examples
                    if grep -q ",Fails$" "$dir/examples.csv" 2>/dev/null; then
                        process_document "$category" "$document"
                    fi
                fi
            done
            
            # Generate summary
            generate_summary
            shift
            ;;
        
        --document)
            doc_path="$2"
            filename=$(basename "$doc_path" .md)
            dirname=$(dirname "$doc_path")
            category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
            
            if [[ -z "$category" ]]; then
                category="misc"
            fi
            
            echo "Processing document $doc_path..."
            process_document "$category" "$filename"
            
            # Generate diff for all examples
            if [[ -f "$FIXED_DIR/$category/$filename/fixes.csv" ]]; then
                tail -n +2 "$FIXED_DIR/$category/$filename/fixes.csv" | cut -d, -f1 | while read -r example_id; do
                    generate_diff "$category" "$filename" "$example_id"
                done
            fi
            
            shift
            ;;
        
        --diff)
            diff_path="$2"
            # Parse the path into category/document/example_id
            category=$(echo "$diff_path" | cut -d/ -f1)
            document=$(echo "$diff_path" | cut -d/ -f2)
            example_id=$(echo "$diff_path" | cut -d/ -f3)
            
            echo "Generating diff for $category/$document example $example_id..."
            generate_diff "$category" "$document" "$example_id"
            shift
            ;;
        
        --apply)
            doc_path="$2"
            echo "Applying fixes to markdown file $doc_path..."
            apply_fixes_to_markdown "$doc_path"
            shift
            ;;
        
        --help)
            echo "Usage:"
            echo "  $0 [options]"
            echo ""
            echo "Options:"
            echo "  --all                Process all documents with failing examples"
            echo "  --document <path>    Process a specific document"
            echo "  --category <name>    Process all documents in a category"
            echo "  --apply <path>       Apply fixes directly to the markdown file"
            echo "  --diff <cat/doc/id>  Generate a markdown diff for a specific example"
            echo "  --help               Display this help message"
            exit 0
            ;;
        
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
    shift
done

echo ""
echo "Code example fixing complete!"
echo "Results are stored in $FIXED_DIR"
echo ""
echo "Next steps:"
echo "1. Review the fixes in the summary report"
echo "2. Apply fixes to markdown files with --apply"
echo "3. Re-run the code-example-verifier.sh to validate fixes"
echo "4. Address examples that couldn't be fixed automatically" 