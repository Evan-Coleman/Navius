#!/bin/bash
#
# generate-summary.sh
#
# Generates an executive summary of validation reports
#
# Usage:
#   ./generate-summary.sh [report_directory]
#

REPORTS_DIR="${1:-$(dirname "$0")/reports}"
OUTPUT_FILE="$(dirname "$0")/validation-summary.md"

if [[ ! -d "$REPORTS_DIR" ]]; then
    echo "Error: Reports directory not found: $REPORTS_DIR"
    exit 1
fi

# Get validation stats
TOTAL_DOCS=$(find "$REPORTS_DIR" -name "*_validation.txt" | wc -l)
FRONTMATTER_ISSUES=$(grep -l "❌.*frontmatter\|❌.*Required field" "$REPORTS_DIR"/*_validation.txt | wc -l)
STRUCTURE_ISSUES=$(grep -l "❌.*heading\|❌.*section" "$REPORTS_DIR"/*_validation.txt | wc -l)
CODE_BLOCKS=$(grep "Code Blocks: " "$REPORTS_DIR"/*_validation.txt | awk '{sum += $3} END {print sum}')
RUST_BLOCKS=$(grep "Rust Blocks: " "$REPORTS_DIR"/*_validation.txt | awk '{sum += $3} END {print sum}')
TOTAL_LINKS=$(grep "Internal Links: " "$REPORTS_DIR"/*_validation.txt | awk '{sum += $3} END {print sum}')
BROKEN_LINKS=$(grep -l "broken link" "$REPORTS_DIR"/*_validation.txt | wc -l)

# Find docs with the most code blocks
TOP_CODE_DOCS=$(grep "Code Blocks: " "$REPORTS_DIR"/*_validation.txt | sort -rn -k3 | head -5 | sed 's/.*\/\(.*\)_validation.txt:Code Blocks: \(.*\)/\1 (\2 blocks)/')

# Find docs with the most links
TOP_LINKED_DOCS=$(grep "Internal Links: " "$REPORTS_DIR"/*_validation.txt | sort -rn -k3 | head -5 | sed 's/.*\/\(.*\)_validation.txt:Internal Links: \(.*\)/\1 (\2 links)/')

# Find docs with issues
DOCS_WITH_FRONTMATTER_ISSUES=$(grep -l "❌.*frontmatter\|❌.*Required field" "$REPORTS_DIR"/*_validation.txt | sed 's/.*\/\(.*\)_validation.txt/\1/')
DOCS_WITH_STRUCTURE_ISSUES=$(grep -l "❌.*heading\|❌.*section" "$REPORTS_DIR"/*_validation.txt | sed 's/.*\/\(.*\)_validation.txt/\1/')
DOCS_WITH_BROKEN_LINKS=$(grep -l "broken link" "$REPORTS_DIR"/*_validation.txt | sed 's/.*\/\(.*\)_validation.txt/\1/')

# Generate markdown summary
cat > "$OUTPUT_FILE" << EOL
---
title: Documentation Validation Executive Summary
description: Executive summary of documentation validation results
category: Documentation
last_updated: $(date +"%B %d, %Y")
---

# Documentation Validation Executive Summary

## Overview

This document provides an executive summary of the documentation validation performed on $(date). The validation process checked frontmatter, structure, code examples, and links in each document.

## Key Metrics

| Metric | Count | Percentage |
|--------|-------|------------|
| Total Documents Validated | $TOTAL_DOCS | 100% |
| Documents with Frontmatter Issues | $FRONTMATTER_ISSUES | $(( FRONTMATTER_ISSUES * 100 / TOTAL_DOCS ))% |
| Documents with Structure Issues | $STRUCTURE_ISSUES | $(( STRUCTURE_ISSUES * 100 / TOTAL_DOCS ))% |
| Documents with Broken Links | $BROKEN_LINKS | $(( BROKEN_LINKS * 100 / TOTAL_DOCS ))% |
| Total Code Blocks | $CODE_BLOCKS | - |
| Rust Code Blocks | $RUST_BLOCKS | $(( RUST_BLOCKS * 100 / CODE_BLOCKS ))% |
| Total Internal Links | $TOTAL_LINKS | - |

## Documents with the Most Code Examples

$(echo "$TOP_CODE_DOCS" | awk '{print "- " $0}')

## Most Referenced Documents

$(echo "$TOP_LINKED_DOCS" | awk '{print "- " $0}')

## Documents Requiring Attention

### Frontmatter Issues

$(echo "$DOCS_WITH_FRONTMATTER_ISSUES" | awk '{print "- " $0}')

### Structure Issues

$(echo "$DOCS_WITH_STRUCTURE_ISSUES" | awk '{print "- " $0}')

### Link Issues

$(echo "$DOCS_WITH_BROKEN_LINKS" | awk '{print "- " $0}')

## Recommendations

Based on the validation results, the following actions are recommended:

1. **Fix Frontmatter**: $(( FRONTMATTER_ISSUES * 100 / TOTAL_DOCS ))% of documents have frontmatter issues. Ensure all documents have proper frontmatter with required fields.

2. **Improve Document Structure**: $(( STRUCTURE_ISSUES * 100 / TOTAL_DOCS ))% of documents have structure issues. Add missing sections and ensure proper heading hierarchy.

3. **Fix Broken Links**: $(( BROKEN_LINKS * 100 / TOTAL_DOCS ))% of documents have broken links. Update links to ensure they point to valid locations.

4. **Code Example Coverage**: Only $(( RUST_BLOCKS * 100 / CODE_BLOCKS ))% of code blocks are specifically marked as Rust code. Ensure all code examples use proper language tags.

## Next Steps

1. Address the issues identified in this summary
2. Re-run validation to track progress
3. Focus on high-priority documents first (most referenced documents)

## Related Documents

- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Documentation Reorganization Instructions](../30_documentation-reorganization-instructions.md)
- [Documentation Standards](/11newdocs11/05_reference/standards/documentation-standards.md)
EOL

echo "Executive summary generated at: $OUTPUT_FILE"
exit 0 