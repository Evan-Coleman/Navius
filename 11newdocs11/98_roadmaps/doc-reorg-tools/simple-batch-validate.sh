#!/bin/bash
#
# simple-batch-validate.sh
#
# Simple batch document validator for the documentation reorganization project
#
# Usage:
#   ./simple-batch-validate.sh <directory> [output_report]
#

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <directory> [output_report]"
    exit 1
fi

DIR="$1"
REPORT="${2:-validation-report-$(date +%Y%m%d).md}"
SCRIPT_DIR="$(dirname "$0")"
VALIDATOR="$SCRIPT_DIR/simple-validate.sh"

if [[ ! -d "$DIR" ]]; then
    echo "Directory not found: $DIR"
    exit 1
fi

if [[ ! -x "$VALIDATOR" ]]; then
    echo "Validator script not found or not executable: $VALIDATOR"
    exit 1
fi

# Initialize counters
TOTAL_DOCS=0
DOCS_WITH_ISSUES=0
TOTAL_CODE_BLOCKS=0
TOTAL_LINKS=0
TOTAL_BROKEN_LINKS=0
MISSING_FRONTMATTER=0
STRUCTURE_ISSUES=0

# Create reports directory if it doesn't exist
REPORTS_DIR="$SCRIPT_DIR/reports"
mkdir -p "$REPORTS_DIR"

# Create an empty file to store detailed results
DETAILED_RESULTS_FILE="$REPORTS_DIR/detailed_results.txt"
echo "" > "$DETAILED_RESULTS_FILE"

# Process each markdown file
while read -r file; do
    echo "Validating: $file"
    FILE_NAME=$(basename "$file")
    
    # Create a file to store the validation results
    RESULTS_FILE="$REPORTS_DIR/${FILE_NAME%.md}_validation.txt"
    
    # Run validation and save output
    $VALIDATOR "$file" > "$RESULTS_FILE"
    
    # Extract simple metrics for summary (avoiding complex parsing)
    # Just count occurrences of key indicators
    
    # Count code blocks
    CODE_BLOCKS_COUNT=$(grep -c "Found .* code blocks" "$RESULTS_FILE" || echo "0")
    if [[ -n "$CODE_BLOCKS_COUNT" && "$CODE_BLOCKS_COUNT" != "0" ]]; then
        # Extract just the number if found
        CODE_BLOCKS=$(grep "Found .* code blocks" "$RESULTS_FILE" | head -1 | sed -E 's/Found ([0-9]+) code blocks.*/\1/' || echo "0")
        TOTAL_CODE_BLOCKS=$((TOTAL_CODE_BLOCKS + CODE_BLOCKS))
    fi
    
    # Count links
    LINKS_COUNT=$(grep -c "Found .* internal links" "$RESULTS_FILE" || echo "0")
    if [[ -n "$LINKS_COUNT" && "$LINKS_COUNT" != "0" ]]; then
        # Extract just the number if found
        LINKS=$(grep "Found .* internal links" "$RESULTS_FILE" | head -1 | sed -E 's/Found ([0-9]+) internal links/\1/' || echo "0")
        TOTAL_LINKS=$((TOTAL_LINKS + LINKS))
    fi
    
    # Check for broken links indicator
    if grep -q "broken link" "$RESULTS_FILE"; then
        BROKEN_COUNT=$(grep -c "broken link" "$RESULTS_FILE" || echo "0")
        TOTAL_BROKEN_LINKS=$((TOTAL_BROKEN_LINKS + BROKEN_COUNT))
        DOCS_WITH_ISSUES=$((DOCS_WITH_ISSUES + 1))
    fi
    
    # Check for frontmatter issues
    if grep -q "❌.*frontmatter" "$RESULTS_FILE" || grep -q "❌.*Required field" "$RESULTS_FILE"; then
        MISSING_FRONTMATTER=$((MISSING_FRONTMATTER + 1))
        DOCS_WITH_ISSUES=$((DOCS_WITH_ISSUES + 1))
    fi
    
    # Check for structure issues
    if grep -q "❌.*heading" "$RESULTS_FILE" || grep -q "❌.*section" "$RESULTS_FILE"; then
        STRUCTURE_ISSUES=$((STRUCTURE_ISSUES + 1))
        DOCS_WITH_ISSUES=$((DOCS_WITH_ISSUES + 1))
    fi
    
    # Add file name to detailed results
    echo "### Document: $FILE_NAME" >> "$DETAILED_RESULTS_FILE"
    echo "" >> "$DETAILED_RESULTS_FILE"
    echo "See detailed results in: $RESULTS_FILE" >> "$DETAILED_RESULTS_FILE"
    echo "" >> "$DETAILED_RESULTS_FILE"
    
    # Increment total document count
    TOTAL_DOCS=$((TOTAL_DOCS + 1))
done < <(find "$DIR" -type f -name "*.md")

# Generate recommendations
RECOMMENDATIONS=""
if [[ $TOTAL_BROKEN_LINKS -gt 0 ]]; then
    RECOMMENDATIONS+="1. Fix the $TOTAL_BROKEN_LINKS broken links identified in the report.\n"
fi
if [[ $MISSING_FRONTMATTER -gt 0 ]]; then
    RECOMMENDATIONS+="2. Add or correct frontmatter in the $MISSING_FRONTMATTER documents with missing or incomplete frontmatter.\n"
fi
if [[ $STRUCTURE_ISSUES -gt 0 ]]; then
    RECOMMENDATIONS+="3. Fix structure issues in the $STRUCTURE_ISSUES documents that are missing required sections.\n"
fi
if [[ -z "$RECOMMENDATIONS" ]]; then
    RECOMMENDATIONS="1. All documents passed validation. No recommendations at this time.\n"
fi

# Generate the report
cat > "$REPORT" << EOL
---
title: Document Validation Report
description: Validation report for documentation reorganization
category: Documentation
last_updated: $(date +"%B %d, %Y")
---

# Document Validation Report

## Overview

This report contains validation results for documentation files validated as part of the documentation reorganization project. The validation was performed using the \`simple-validate.sh\` script on $(date).

## Validation Summary

| Metric | Count |
|--------|-------|
| Documents Validated | $TOTAL_DOCS |
| Documents with Issues | $DOCS_WITH_ISSUES |
| Total Code Blocks | $TOTAL_CODE_BLOCKS |
| Total Internal Links | $TOTAL_LINKS |
| Broken Links | $TOTAL_BROKEN_LINKS |
| Documents Missing Frontmatter | $MISSING_FRONTMATTER |
| Documents with Structure Issues | $STRUCTURE_ISSUES |

## Detailed Results

This section contains a list of validated documents. Detailed validation results for each document are stored in the \`reports\` directory.

$(cat "$DETAILED_RESULTS_FILE")

## Recommendations

Based on the validation results, the following recommendations are made:

$(echo -e "$RECOMMENDATIONS")

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Documentation Reorganization Instructions](../30_documentation-reorganization-instructions.md)
- [Documentation Standards](/11newdocs11/05_reference/standards/documentation-standards.md)
EOL

echo "Validation completed. Report generated at: $REPORT"
echo "Summary:"
echo "- Documents Validated: $TOTAL_DOCS"
echo "- Documents with Issues: $DOCS_WITH_ISSUES"
echo "- Total Code Blocks: $TOTAL_CODE_BLOCKS"
echo "- Total Internal Links: $TOTAL_LINKS"
echo "- Broken Links: $TOTAL_BROKEN_LINKS"
echo "- Documents Missing Frontmatter: $MISSING_FRONTMATTER"
echo "- Documents with Structure Issues: $STRUCTURE_ISSUES"

exit 0 