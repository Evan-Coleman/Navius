#!/bin/bash

# Documentation Validation Script
# This script analyzes documentation without making any changes

set -e

# Set the docs directory
DOCS_DIR="docs"

# Set the reports directory
REPORTS_DIR="target/reports/docs_validation"

# Create reports directory if it doesn't exist
mkdir -p "$REPORTS_DIR"

# Initialize counters
total_files=0
files_with_issues=0
files_missing_frontmatter=0
files_with_broken_links=0
total_broken_links=0
files_with_relative_links=0
total_relative_links=0
files_missing_related=0
first_issue_file=""

# Get all markdown files in the docs directory
ALL_FILES=$(find "$DOCS_DIR" -type f -name "*.md" | sort)

# Check for verbose flag
VERBOSE=false

for arg in "$@"; do
    if [ "$arg" = "-v" ] || [ "$arg" = "--verbose" ]; then
        VERBOSE=true
    fi
done

TODAY_DATE=$(date "+%B %d, %Y")
REPORT_FILE="$REPORTS_DIR/doc_validation_report.md"

# Create report header
echo "# Documentation Validation Report - ${TODAY_DATE}" > $REPORT_FILE
echo "" >> $REPORT_FILE
echo "## Summary" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Create temporary files
MISSING_FRONTMATTER=$(mktemp)
MISSING_RELATED=$(mktemp)
BROKEN_LINKS=$(mktemp)
RELATIVE_LINKS=$(mktemp)

# Check if a file has frontmatter
has_frontmatter() {
    local file="$1"
    head -n 20 "$file" | grep -q "^---" && head -n 20 "$file" | grep -q "^title:"
    return $?
}

# Check if a file has an overview section
has_overview() {
    local file="$1"
    # Check for ## Overview or # Overview
    grep -q "^## Overview\|^# Overview" "$file"
    return $?
}

# Check if a file has a 'Related Documents' section
has_related_documents() {
    local file="$1"
    grep -q "Related Documents" "$file"
    return $?
}

# Extract links from a file
extract_links() {
    local file="$1"
    grep -o -E '\[[^]]+\]\([^)]+\)' "$file" | awk -F'[()]' '{print $2}'
}

# Extract relative links from a file
extract_relative_links() {
    local file="$1"
    local links=$(extract_links "$file")
    local relative_links=()
    
    for link in $links; do
        # Skip external links
        if [[ "$link" =~ ^(http|https|ftp):// ]]; then
            continue
        fi
        
        # Skip correctly formatted absolute paths that exist
        if [[ "$link" == /docs/* ]] && link_exists "$link"; then
            continue
        fi
        
        # Check for relative links
        if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
            relative_links+=("$link")
        fi
    done
    
    # Return the relative links
    if [ ${#relative_links[@]} -gt 0 ]; then
        printf '%s\n' "${relative_links[@]}"
    fi
}

# Check if a linked file exists
link_exists() {
    local link="$1"
    
    # Handle absolute paths (starting with /docs/)
    if [[ "$link" == /docs/* ]]; then
        local path_after_docs="${link#/docs}"
        if [[ -f "docs${path_after_docs}" ]]; then
            return 0  # Link exists
        else
            return 1  # Link doesn't exist
        fi
    fi
    
    # Handle relative paths
    local dir=$(dirname "$2")
    local target_path
    
    # Handle ../ paths
    if [[ "$link" =~ ^\.\./ ]]; then
        target_path=$(realpath --relative-to="$(pwd)" "$dir/$link" 2>/dev/null)
        if [[ -f "$target_path" ]]; then
            return 0  # Link exists
        fi
        return 1  # Link doesn't exist
    fi
    
    # Handle ./ paths
    if [[ "$link" =~ ^\./ ]]; then
        target_path="$dir/${link:2}"
        if [[ -f "$target_path" ]]; then
            return 0  # Link exists
        fi
        return 1  # Link doesn't exist
    fi
    
    # Handle paths with no ./ or ../
    target_path="$dir/$link"
    if [[ -f "$target_path" ]]; then
        return 0  # Link exists
    fi
    
    return 1  # Link doesn't exist
}

echo "Scanning markdown files in $DOCS_DIR..."

# Process each markdown file
for file in $ALL_FILES; do
    total_files=$((total_files + 1))
    file_issues=0
    
    # Check for frontmatter
    if ! has_frontmatter "$file"; then
        files_missing_frontmatter=$((files_missing_frontmatter + 1))
        if [ -z "$first_issue_file" ]; then
            first_issue_file="$file"
        fi
        file_issues=1
        echo "$file" >> $MISSING_FRONTMATTER
        
        if [ "$VERBOSE" = true ]; then
            echo "❌ Missing frontmatter: $file"
        fi
    fi
    
    # Check for 'Related Documents' section
    if ! has_related_documents "$file"; then
        # Skip README files for Related Documents check
        if [[ $(basename "$file") != "README.md" ]]; then
            files_missing_related=$((files_missing_related + 1))
            if [ -z "$first_issue_file" ]; then
                first_issue_file="$file"
            fi
            file_issues=1
            echo "$file" >> $MISSING_RELATED
            
            if [ "$VERBOSE" = true ]; then
                echo "❌ Missing 'Related Documents' section: $file"
            fi
        fi
    fi
    
    # Check for broken links
    broken_links=$(extract_links "$file")
    file_broken_links=0
    if [[ -n "$broken_links" ]]; then
        for link in $broken_links; do
            # Skip external links
            if [[ "$link" =~ ^(http|https|ftp):// ]]; then
                continue
            fi
            
            # Skip links that are already correctly formatted with /docs/ and exist
            if [[ "$link" == /docs/* ]] && link_exists "$link"; then
                continue
            fi
            
            if ! link_exists "$link" "$file"; then
                if [ $file_broken_links -eq 0 ]; then
                    files_with_broken_links=$((files_with_broken_links + 1))
                    if [ -z "$first_issue_file" ]; then
                        first_issue_file="$file"
                    fi
                fi
                
                total_broken_links=$((total_broken_links + 1))
                file_broken_links=$((file_broken_links + 1))
                file_issues=1
                echo "$file|$link" >> $BROKEN_LINKS
                
                if [ "$VERBOSE" = true ]; then
                    echo "❌ Broken link in $file: $link"
                fi
            fi
        done
    fi
    
    # Check for relative links
    relative_links=$(extract_relative_links "$file")
    file_relative_links=0
    if [[ -n "$relative_links" ]]; then
        while IFS= read -r link; do
            if [ $file_relative_links -eq 0 ]; then
                files_with_relative_links=$((files_with_relative_links + 1))
                if [ -z "$first_issue_file" ]; then
                    first_issue_file="$file"
                fi
            fi
            
            total_relative_links=$((total_relative_links + 1))
            file_relative_links=$((file_relative_links + 1))
            file_issues=1
            echo "$file|$link" >> $RELATIVE_LINKS
            
            if [ "$VERBOSE" = true ]; then
                echo "⚠️ Relative link in $file: $link"
            fi
        done <<< "$relative_links"
    fi
    
    if [ $file_issues -gt 0 ]; then
        files_with_issues=$((files_with_issues + 1))
    elif [ "$VERBOSE" = true ]; then
        echo "✅ $file"
    fi
done

# Update summary
echo "- Total markdown files: $total_files" >> $REPORT_FILE
echo "- Files missing frontmatter: $files_missing_frontmatter" >> $REPORT_FILE
echo "- Files with broken links: $files_with_broken_links" >> $REPORT_FILE
echo "- Total broken links: $total_broken_links" >> $REPORT_FILE
echo "- Files with relative links: $files_with_relative_links" >> $REPORT_FILE
echo "- Total relative links: $total_relative_links" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Add detailed information
if [[ $files_missing_frontmatter -gt 0 ]]; then
    echo "## Files Missing Frontmatter" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    while IFS= read -r file; do
        echo "- \`$file\`" >> $REPORT_FILE
    done < $MISSING_FRONTMATTER
    echo "" >> $REPORT_FILE
fi

if [[ $files_missing_related -gt 0 ]]; then
    echo "## Files Missing 'Related Documents' Section" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    while IFS= read -r file; do
        echo "- \`$file\`" >> $REPORT_FILE
    done < $MISSING_RELATED
    echo "" >> $REPORT_FILE
fi

if [[ $files_with_broken_links -gt 0 ]]; then
    echo "## Broken Links" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    echo "| File | Broken Link |" >> $REPORT_FILE
    echo "|------|-------------|" >> $REPORT_FILE
    while IFS="|" read -r file link; do
        echo "| \`$file\` | \`$link\` |" >> $REPORT_FILE
    done < $BROKEN_LINKS
    echo "" >> $REPORT_FILE
fi

if [[ $relative_links -gt 0 ]]; then
    echo "## Relative Links (should use absolute paths)" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    echo "These links should be updated to use absolute paths from the project root (starting with /docs/)." >> $REPORT_FILE
    echo "" >> $REPORT_FILE
    echo "| File | Relative Link |" >> $REPORT_FILE
    echo "|------|---------------|" >> $REPORT_FILE
    while IFS="|" read -r file link; do
        echo "| \`$file\` | \`$link\` |" >> $REPORT_FILE
    done < $RELATIVE_LINKS
    echo "" >> $REPORT_FILE
fi

# Clean up
rm $MISSING_FRONTMATTER $MISSING_RELATED $BROKEN_LINKS $RELATIVE_LINKS

echo "Validation completed!"
echo "See $REPORT_FILE for the detailed report."
echo ""
echo "Summary:"
echo "- Total markdown files: $total_files"
echo "- Files missing frontmatter: $files_missing_frontmatter"
echo "- Files with broken links: $files_with_broken_links"
echo "- Total broken links: $total_broken_links"
echo "- Files with relative links: $files_with_relative_links"
echo "- Total relative links: $total_relative_links"

# Print the summary report
if [ "$VERBOSE" = true ]; then
    echo
    echo "------------------------------------------------------------------"
    echo "Validation Report Summary"
    echo "------------------------------------------------------------------"
    echo "Total files scanned: $total_files"
    echo "Files with issues: $files_with_issues"
    echo 
    echo "## Breakdown of Issues:"
    echo "- Files missing frontmatter: $files_missing_frontmatter"
    echo "- Files with broken links: $files_with_broken_links"
    echo "   - Total broken links found: $total_broken_links"
    echo "- Files with relative links: $files_with_relative_links"
    echo "   - Total relative links found: $total_relative_links"
    echo "- Files missing 'Related Documents' section: $files_missing_related"
    echo "------------------------------------------------------------------"
    
    if [ $files_with_issues -gt 0 ]; then
        echo
        echo "To fix these issues, run the following for each file with issues:"
        echo "./scripts/doc-overhaul/improve_docs.sh <filename>"
        echo
        echo "For example:"
        echo "./scripts/doc-overhaul/improve_docs.sh $first_issue_file"
    fi
fi

# Print a summary line for non-verbose output
if [ "$VERBOSE" = false ] && [ $files_with_issues -gt 0 ]; then
    echo
    echo "✖ Found $files_with_issues files with issues (out of $total_files scanned)."
    echo
    echo "Run with -v for detailed report or run the following to fix issues:"
    echo "./scripts/doc-overhaul/improve_docs.sh <filename>"
    
    if [ -n "$first_issue_file" ]; then
        echo
        echo "For example: ./scripts/doc-overhaul/improve_docs.sh $first_issue_file"
    fi
fi

if [ $files_with_issues -eq 0 ]; then
    echo
    echo "✅ All documentation files passed validation!"
fi 