#!/bin/bash
#
# batch-validate.sh
#
# Batch document validator for the documentation reorganization project
#
# Usage:
#   ./batch-validate.sh <directory> [output_report]
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
DETAILED_RESULTS=""

# Process each markdown file
while IFS= read -r file; do
    echo "Validating: $file"
    
    # Run validation and capture output
    VALIDATION_OUTPUT=$(bash "$VALIDATOR" "$file")
    
    # Parse validation results
    FILE_HAS_ISSUES=0
    FILE_NAME=$(basename "$file")
    
    # Extract validation date
    VALIDATION_DATE=$(echo "$VALIDATION_OUTPUT" | grep "Timestamp:" | head -1 | cut -d ' ' -f 2-)
    
    # Extract frontmatter results
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Frontmatter found"; then
        FRONTMATTER_FOUND="✅"
    else
        FRONTMATTER_FOUND="❌"
        FILE_HAS_ISSUES=1
        ((MISSING_FRONTMATTER++))
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Required field 'title' present"; then
        TITLE_PRESENT="✅"
    else
        TITLE_PRESENT="❌"
        FILE_HAS_ISSUES=1
        if [[ "$FRONTMATTER_FOUND" == "✅" ]]; then
            ((MISSING_FRONTMATTER++))
        fi
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Required field 'description' present"; then
        DESC_PRESENT="✅"
    else
        DESC_PRESENT="❌"
        FILE_HAS_ISSUES=1
        if [[ "$FRONTMATTER_FOUND" == "✅" ]]; then
            ((MISSING_FRONTMATTER++))
        fi
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Required field 'category' present"; then
        CATEGORY_PRESENT="✅"
    else
        CATEGORY_PRESENT="❌"
        FILE_HAS_ISSUES=1
        if [[ "$FRONTMATTER_FOUND" == "✅" ]]; then
            ((MISSING_FRONTMATTER++))
        fi
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Required field 'last_updated' present"; then
        UPDATED_PRESENT="✅"
    else
        UPDATED_PRESENT="❌"
        FILE_HAS_ISSUES=1
        if [[ "$FRONTMATTER_FOUND" == "✅" ]]; then
            ((MISSING_FRONTMATTER++))
        fi
    fi
    
    # Extract structure results
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Main heading (H1) found"; then
        H1_FOUND="✅"
    else
        H1_FOUND="❌"
        FILE_HAS_ISSUES=1
        ((STRUCTURE_ISSUES++))
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Overview section found"; then
        OVERVIEW_FOUND="✅"
    else
        OVERVIEW_FOUND="❌"
        FILE_HAS_ISSUES=1
        ((STRUCTURE_ISSUES++))
    fi
    
    if echo "$VALIDATION_OUTPUT" | grep -q "✅ Related Documents section found"; then
        RELATED_FOUND="✅"
    else
        RELATED_FOUND="❌"
        FILE_HAS_ISSUES=1
        ((STRUCTURE_ISSUES++))
    fi
    
    # Extract code blocks count and links count
    CODE_BLOCKS_LINE=$(echo "$VALIDATION_OUTPUT" | grep "Found .* code blocks")
    if [[ -n "$CODE_BLOCKS_LINE" ]]; then
        CODE_BLOCKS=$(echo "$CODE_BLOCKS_LINE" | sed -E 's/Found ([0-9]+) code blocks.*/\1/' || echo "0")
        RUST_BLOCKS=$(echo "$CODE_BLOCKS_LINE" | sed -E 's/.*\(([0-9]+) Rust blocks\)/\1/' || echo "0")
    else
        CODE_BLOCKS=0
        RUST_BLOCKS=0
    fi
    
    LINKS_LINE=$(echo "$VALIDATION_OUTPUT" | grep "Found .* internal links")
    if [[ -n "$LINKS_LINE" ]]; then
        LINKS=$(echo "$LINKS_LINE" | sed -E 's/Found ([0-9]+) internal links/\1/' || echo "0")
    else
        LINKS=0
    fi
    
    # Check if broken links are reported
    BROKEN_LINKS=0
    if echo "$VALIDATION_OUTPUT" | grep -q "❌ Found .* broken internal links"; then
        BROKEN_LINKS_LINE=$(echo "$VALIDATION_OUTPUT" | grep "❌ Found .* broken internal links")
        BROKEN_LINKS=$(echo "$BROKEN_LINKS_LINE" | sed -E 's/.*Found ([0-9]+) broken internal links/\1/' || echo "0")
        LINK_STATUS="❌"
        FILE_HAS_ISSUES=1
    elif echo "$VALIDATION_OUTPUT" | grep -q "✅ All internal links are valid"; then
        LINK_STATUS="✅"
    else
        LINK_STATUS="⚠️"
    fi
    
    # Update counters
    ((TOTAL_DOCS++))
    TOTAL_CODE_BLOCKS=$((TOTAL_CODE_BLOCKS + CODE_BLOCKS))
    TOTAL_LINKS=$((TOTAL_LINKS + LINKS))
    TOTAL_BROKEN_LINKS=$((TOTAL_BROKEN_LINKS + BROKEN_LINKS))
    
    if [[ $FILE_HAS_ISSUES -eq 1 ]]; then
        ((DOCS_WITH_ISSUES++))
    fi
    
    # Add to detailed results
    DETAILED_RESULTS="${DETAILED_RESULTS}### Document: ${FILE_NAME}

**Validation Date:** ${VALIDATION_DATE}

#### Frontmatter
- ${FRONTMATTER_FOUND} Frontmatter found
- ${TITLE_PRESENT} Required field 'title' present
- ${DESC_PRESENT} Required field 'description' present
- ${CATEGORY_PRESENT} Required field 'category' present
- ${UPDATED_PRESENT} Required field 'last_updated' present

#### Document Structure
- ${H1_FOUND} Main heading (H1) found
- ${OVERVIEW_FOUND} Overview section found
- ${RELATED_FOUND} Related Documents section found

#### Code Examples
- Code Blocks: ${CODE_BLOCKS}
- Rust Blocks: ${RUST_BLOCKS}

#### Links
- Internal Links: ${LINKS}
- Broken Links: ${BROKEN_LINKS}
- ${LINK_STATUS} Link validation status

"
    
done < <(find "$DIR" -type f -name "*.md")

# Generate recommendations
RECOMMENDATIONS=""
if [[ $TOTAL_BROKEN_LINKS -gt 0 ]]; then
    RECOMMENDATIONS="${RECOMMENDATIONS}1. Fix the ${TOTAL_BROKEN_LINKS} broken links identified in the report.
"
fi
if [[ $MISSING_FRONTMATTER -gt 0 ]]; then
    RECOMMENDATIONS="${RECOMMENDATIONS}2. Add or correct frontmatter in the ${MISSING_FRONTMATTER} documents with missing or incomplete frontmatter.
"
fi
if [[ $STRUCTURE_ISSUES -gt 0 ]]; then
    RECOMMENDATIONS="${RECOMMENDATIONS}3. Fix structure issues in the ${STRUCTURE_ISSUES} documents that are missing required sections.
"
fi
if [[ -z "$RECOMMENDATIONS" ]]; then
    RECOMMENDATIONS="1. All documents passed validation. No recommendations at this time.
"
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

This section contains validation results for individual documents.

$DETAILED_RESULTS

## Recommendations

Based on the validation results, the following recommendations are made:

$RECOMMENDATIONS

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