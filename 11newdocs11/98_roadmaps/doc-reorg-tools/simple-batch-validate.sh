#!/bin/bash
#
# simple-batch-validate.sh
#
# Simple batch document validator for the documentation reorganization project
#
# Usage:
#   ./simple-batch-validate.sh <directory> [output_file] [--verbose]
#

SCRIPT_DIR="$(dirname "$0")"
BASE_DIR="/Users/goblin/dev/git/navius"
VERBOSE=false

# Check if required arguments are provided
if [ $# -lt 1 ]; then
    echo "Usage: ./simple-batch-validate.sh <directory> [output_file] [--verbose]"
    exit 1
fi

TARGET_DIR="$1"
OUTPUT_FILE="${2:-${SCRIPT_DIR}/reports/validation-report.md}"

# Process additional flags
if [[ "$*" == *"--verbose"* ]]; then
    VERBOSE=true
fi

# Create reports directory if it doesn't exist
mkdir -p "${SCRIPT_DIR}/reports"

# Log message helper function
log_message() {
    local message="$1"
    if [ "$VERBOSE" = true ]; then
        echo "$message"
    fi
}

log_message "Starting validation on directory: $TARGET_DIR"
log_message "Output will be written to: $OUTPUT_FILE"

# Function to count files in a directory
count_files() {
    local dir="$1"
    local extension="$2"
    find "$dir" -type f -name "*.$extension" | wc -l
}

# Function to check for frontmatter issues
check_frontmatter() {
    local file="$1"
    local issues=0
    
    # Check if file has frontmatter
    if ! grep -q "^---" "$file"; then
        echo "- Missing frontmatter"
        issues=$((issues + 1))
    else
        # Check for essential frontmatter fields
        if ! grep -q "^title:" "$file"; then
            echo "- Missing title in frontmatter"
            issues=$((issues + 1))
        fi
        
        if ! grep -q "^description:" "$file"; then
            echo "- Missing description in frontmatter"
            issues=$((issues + 1))
        fi
        
        if ! grep -q "^last_updated:" "$file"; then
            echo "- Missing last_updated in frontmatter"
            issues=$((issues + 1))
        fi
    fi
    
    return $issues
}

# Function to check for required sections
check_sections() {
    local file="$1"
    local issues=0
    
    # Check for essential sections
    if ! grep -q "^## Overview" "$file"; then
        echo "- Missing Overview section"
        issues=$((issues + 1))
    fi
    
    # Only some documents need these sections
    local filename=$(basename "$file")
    if [[ "$filename" != "README.md" ]]; then
        if [[ "$TARGET_DIR" == *"01_getting_started"* ]] || 
           [[ "$TARGET_DIR" == *"02_examples"* ]] || 
           [[ "$TARGET_DIR" == *"04_guides"* ]]; then
            if ! grep -q "^## Installation" "$file"; then
                echo "- Missing Installation section"
                issues=$((issues + 1))
            fi
            
            if ! grep -q "^## Configuration" "$file"; then
                echo "- Missing Configuration section"
                issues=$((issues + 1))
            fi
        fi
    fi
    
    return $issues
}

# Function to check for broken links
check_links() {
    local file="$1"
    local issues=0
    local links
    
    # Extract markdown links
    links=$(grep -o '\[[^]]*\]([^)]*)' "$file" | grep -o '([^)]*)' | sed 's/^(//' | sed 's/)$//')
    
    for link in $links; do
        # Skip external links and anchor links
        if [[ "$link" == http* ]] || [[ "$link" == "#"* ]]; then
            continue
        fi
        
        # Check if the link exists
        resolved_path="${link}"
        
        # If it's a relative path, try to resolve it relative to the file
        if [[ "${link:0:1}" != "/" ]]; then
            file_dir=$(dirname "$file")
            resolved_path="$file_dir/$link"
        fi
        
        # Normalize path to handle ../ references
        normalized_path=$(realpath -m --relative-to="$BASE_DIR" "$resolved_path")
        full_path="${BASE_DIR}/${normalized_path}"
        
        if [ ! -e "$full_path" ]; then
            echo "- Broken link: $link"
            issues=$((issues + 1))
        fi
    done
    
    return $issues
}

# Function to check for duplicate sections
check_duplicate_sections() {
    local file="$1"
    local issues=0
    
    # Get all section headers
    local headers=$(grep -E "^#{2,4} " "$file" | sort)
    local prev_header=""
    
    for header in $headers; do
        if [ "$header" == "$prev_header" ]; then
            echo "- Duplicate section: $header"
            issues=$((issues + 1))
        fi
        prev_header="$header"
    done
    
    return $issues
}

# Generate the validation report
generate_report() {
    local dir_name=$(basename "$TARGET_DIR")
    local md_files=$(find "$TARGET_DIR" -type f -name "*.md" | sort)
    local total_files=$(echo "$md_files" | wc -l)
    local files_with_issues=0
    local total_issues=0
    local frontmatter_issues=0
    local section_issues=0
    local link_issues=0
    local duplicate_issues=0
    
    # Create the report header
    cat > "$OUTPUT_FILE" << EOF
---
title: Validation Report for ${dir_name}
description: Summary of validation checks on markdown documentation
category: Documentation
tags: [validation, report, quality-check]
last_updated: $(date "+%B %d, %Y")
version: 1.0
---

# Validation Report: ${dir_name}

## Overview

This report summarizes the results of validation checks performed on markdown documentation files in the \`${dir_name}\` directory. The checks include frontmatter validation, required sections, link validation, and duplicate section detection.

## Summary

| Metric | Count |
|--------|-------|
| Total files | ${total_files} |
| Files with issues | 0 |
| Total issues | 0 |
| Frontmatter issues | 0 |
| Missing section issues | 0 |
| Broken link issues | 0 |
| Duplicate section issues | 0 |

## Files with Issues

EOF

    local issue_details=""
    
    # Process each file
    for file in $md_files; do
        local filename=$(basename "$file")
        local file_issues=0
        local file_report=""
        
        log_message "Checking file: $filename"
        
        # Check frontmatter
        file_report+="### $filename\n\n"
        local frontmatter_result=$(check_frontmatter "$file")
        local frontmatter_count=$?
        if [ $frontmatter_count -gt 0 ]; then
            file_report+="#### Frontmatter Issues\n\n$frontmatter_result\n\n"
            file_issues=$((file_issues + frontmatter_count))
            frontmatter_issues=$((frontmatter_issues + frontmatter_count))
        fi
        
        # Check sections
        local sections_result=$(check_sections "$file")
        local sections_count=$?
        if [ $sections_count -gt 0 ]; then
            file_report+="#### Section Issues\n\n$sections_result\n\n"
            file_issues=$((file_issues + sections_count))
            section_issues=$((section_issues + sections_count))
        fi
        
        # Check links
        local links_result=$(check_links "$file")
        local links_count=$?
        if [ $links_count -gt 0 ]; then
            file_report+="#### Link Issues\n\n$links_result\n\n"
            file_issues=$((file_issues + links_count))
            link_issues=$((link_issues + links_count))
        fi
        
        # Check duplicate sections
        local duplicate_result=$(check_duplicate_sections "$file")
        local duplicate_count=$?
        if [ $duplicate_count -gt 0 ]; then
            file_report+="#### Duplicate Section Issues\n\n$duplicate_result\n\n"
            file_issues=$((file_issues + duplicate_count))
            duplicate_issues=$((duplicate_issues + duplicate_count))
        fi
        
        # Add to report if issues were found
        if [ $file_issues -gt 0 ]; then
            issue_details+="$file_report"
            files_with_issues=$((files_with_issues + 1))
            total_issues=$((total_issues + file_issues))
            log_message "Found $file_issues issues in $filename"
        else
            log_message "No issues found in $filename"
        fi
    done
    
    # Update summary with actual counts
    sed -i.bak "s/Files with issues | 0/Files with issues | ${files_with_issues}/" "$OUTPUT_FILE"
    sed -i.bak "s/Total issues | 0/Total issues | ${total_issues}/" "$OUTPUT_FILE"
    sed -i.bak "s/Frontmatter issues | 0/Frontmatter issues | ${frontmatter_issues}/" "$OUTPUT_FILE"
    sed -i.bak "s/Missing section issues | 0/Missing section issues | ${section_issues}/" "$OUTPUT_FILE"
    sed -i.bak "s/Broken link issues | 0/Broken link issues | ${link_issues}/" "$OUTPUT_FILE"
    sed -i.bak "s/Duplicate section issues | 0/Duplicate section issues | ${duplicate_issues}/" "$OUTPUT_FILE"
    rm "${OUTPUT_FILE}.bak"
    
    # Add issue details or No Issues message
    if [ $files_with_issues -gt 0 ]; then
        echo -e "$issue_details" >> "$OUTPUT_FILE"
    else
        echo -e "\nNo issues found in any files.\n" >> "$OUTPUT_FILE"
    fi
    
    # Add recommendations section
    cat >> "$OUTPUT_FILE" << EOF

## Recommendations

Based on the validation results, here are recommendations for improving documentation quality:

${frontmatter_issues -gt 0 ? "- Fix frontmatter issues by ensuring all files have title, description, and last_updated fields" : ""}
${section_issues -gt 0 ? "- Add missing sections to documents that require them" : ""}
${link_issues -gt 0 ? "- Fix broken links to ensure all references point to valid resources" : ""}
${duplicate_issues -gt 0 ? "- Remove duplicate sections to improve document structure and readability" : ""}
${total_issues -eq 0 ? "- Documentation appears to be in good condition! Continue maintaining this standard." : ""}

## Next Steps

1. Address identified issues in priority order: broken links > missing sections > frontmatter issues > duplicate sections
2. Run validation checks regularly to ensure documentation quality is maintained
3. Update this report after making changes to verify improvements
EOF

    log_message "Validation report generated: $OUTPUT_FILE"
    log_message "Total files: $total_files"
    log_message "Files with issues: $files_with_issues"
    log_message "Total issues: $total_issues"
}

# Run the report generation
generate_report

exit 0 