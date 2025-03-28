#!/bin/bash
#
# run-consolidated-validation.sh
#
# Integrated validation script that runs all validation tools on a document or directory
# and generates a consolidated report.
#
# Usage:
#   ./run-consolidated-validation.sh --file <file_path>
#   ./run-consolidated-validation.sh --dir <directory_path>
#   ./run-consolidated-validation.sh --dir <directory_path> --tier <1|2|3>
#   ./run-consolidated-validation.sh --dir <directory_path> --report
#
# Created: March 27, 2025
# Author: Navius Documentation Team
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
TARGET_FILE=""
TARGET_DIR=""
GENERATE_REPORT=false
VALIDATION_TIER=1

# Output formatting
BOLD="\033[1m"
RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
BLUE="\033[34m"
RESET="\033[0m"

# Help message
show_help() {
    echo -e "${BOLD}Consolidated Validation Script${RESET}"
    echo "Runs all validation tools on specified documents and generates a consolidated report."
    echo
    echo -e "${BOLD}Usage:${RESET}"
    echo "  ./run-consolidated-validation.sh --file <file_path>"
    echo "  ./run-consolidated-validation.sh --dir <directory_path>"
    echo "  ./run-consolidated-validation.sh --dir <directory_path> --tier <1|2|3>"
    echo "  ./run-consolidated-validation.sh --dir <directory_path> --report"
    echo
    echo -e "${BOLD}Options:${RESET}"
    echo "  --file <file_path>    Path to a single Markdown document to validate"
    echo "  --dir <directory_path> Path to a directory of Markdown documents to validate"
    echo "  --tier <1|2|3>        Validation tier (1=100%, 2=50% sample, 3=spot checking)"
    echo "  --report              Generate a consolidated validation report"
    echo "  --help                Show this help message"
    echo
    echo -e "${BOLD}Examples:${RESET}"
    echo "  ./run-consolidated-validation.sh --file ../01_getting_started/installation.md"
    echo "  ./run-consolidated-validation.sh --dir ../01_getting_started/ --tier 1"
    echo "  ./run-consolidated-validation.sh --dir ../02_examples/ --report"
    echo
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --file)
            TARGET_FILE="$2"
            shift
            shift
            ;;
        --dir)
            TARGET_DIR="$2"
            shift
            shift
            ;;
        --tier)
            VALIDATION_TIER="$2"
            shift
            shift
            ;;
        --report)
            GENERATE_REPORT=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${RESET}"
            show_help
            exit 1
            ;;
    esac
done

# Check for valid arguments
if [[ -z "$TARGET_FILE" && -z "$TARGET_DIR" ]]; then
    echo -e "${RED}Error: Either --file or --dir must be specified${RESET}"
    show_help
    exit 1
fi

if [[ -n "$TARGET_FILE" && -n "$TARGET_DIR" ]]; then
    echo -e "${RED}Error: Cannot specify both --file and --dir${RESET}"
    show_help
    exit 1
fi

# Check if file or directory exists
if [[ -n "$TARGET_FILE" && ! -f "$TARGET_FILE" ]]; then
    echo -e "${RED}Error: File $TARGET_FILE does not exist${RESET}"
    exit 1
fi

if [[ -n "$TARGET_DIR" && ! -d "$TARGET_DIR" ]]; then
    echo -e "${RED}Error: Directory $TARGET_DIR does not exist${RESET}"
    exit 1
fi

# Create reports directory
REPORTS_DIR="$SCRIPT_DIR/reports"
mkdir -p "$REPORTS_DIR"

# Generate the report file name
if [[ -n "$TARGET_FILE" ]]; then
    FILE_NAME=$(basename "$TARGET_FILE" .md)
    REPORT_FILE="$REPORTS_DIR/validation_report_${FILE_NAME}_${TIMESTAMP}.md"
    FILES_TO_VALIDATE=("$TARGET_FILE")
else
    DIR_NAME=$(basename "$TARGET_DIR")
    REPORT_FILE="$REPORTS_DIR/validation_report_${DIR_NAME}_${TIMESTAMP}.md"
    
    # Find markdown files in the directory
    if [[ "$VALIDATION_TIER" == "1" ]]; then
        # For Tier 1, validate all files
        FILES_TO_VALIDATE=($(find "$TARGET_DIR" -name "*.md" -type f))
    elif [[ "$VALIDATION_TIER" == "2" ]]; then
        # For Tier 2, validate approximately 50% of files
        FILES_TO_VALIDATE=($(find "$TARGET_DIR" -name "*.md" -type f | sort -R | head -n $(( $(find "$TARGET_DIR" -name "*.md" -type f | wc -l) / 2 ))))
    else
        # For Tier 3, validate a small sample (approximately 20%)
        FILES_TO_VALIDATE=($(find "$TARGET_DIR" -name "*.md" -type f | sort -R | head -n $(( $(find "$TARGET_DIR" -name "*.md" -type f | wc -l) / 5 ))))
    fi
fi

# Initialize the report
cat > "$REPORT_FILE" << EOF
---
title: "Consolidated Validation Report"
description: "Validation results for documentation quality checks"
category: reference
tags:
  - documentation
  - validation
  - quality
  - report
related:
  - ../30_documentation-reorganization-roadmap.md
  - ./phase2-completion-plan.md
  - ./validation-tracking-template.md
last_updated: $(date "+%B %d, %Y")
version: 1.0
---

# Consolidated Validation Report

## Overview

This report contains the consolidated results of validation checks performed on 
$(if [[ -n "$TARGET_FILE" ]]; then echo "the document: \`$TARGET_FILE\`"; else echo "the documents in: \`$TARGET_DIR\`"; fi).

Validation Tier: $VALIDATION_TIER
Generated on: $(date "+%B %d, %Y at %H:%M:%S")

## Summary

| Category | Count |
|----------|-------|
| Documents Validated | ${#FILES_TO_VALIDATE[@]} |
| Code Examples Found | 0 |
| Links Found | 0 |
| Frontmatter Issues | 0 |
| Structure Issues | 0 |
| Code Issues | 0 |
| Link Issues | 0 |
| Overall Validation Status | Pending |

## Detailed Results

EOF

# Initialize counters
TOTAL_CODE_EXAMPLES=0
TOTAL_LINKS=0
TOTAL_FRONTMATTER_ISSUES=0
TOTAL_STRUCTURE_ISSUES=0
TOTAL_CODE_ISSUES=0
TOTAL_LINK_ISSUES=0
VALIDATION_SUCCESS=0
VALIDATION_FAILURE=0

# Process each file
for file in "${FILES_TO_VALIDATE[@]}"; do
    echo -e "\n${BOLD}${BLUE}Validating:${RESET} $file"
    
    TEMP_DIR=$(mktemp -d)
    TEMP_EXTRACT="$TEMP_DIR/examples.md"
    TEMP_VERIFY="$TEMP_DIR/verification.md"
    TEMP_LINKS="$TEMP_DIR/links.md"
    TEMP_STRUCTURE="$TEMP_DIR/structure.md"
    
    # Document Section in Report
    FILE_SECTION="### $(basename "$file")\n\n"
    
    # Initialize counters for this file
    CODE_EXAMPLES=0
    CODE_ISSUES=0
    LINKS=0
    LINK_ISSUES=0
    FRONTMATTER_ISSUES=0
    STRUCTURE_ISSUES=0
    
    # Extract and verify code examples
    echo -e "${BLUE}Extracting code examples...${RESET}"
    if [[ -f "$SCRIPT_DIR/code-example-extractor.sh" ]]; then
        "$SCRIPT_DIR/code-example-extractor.sh" --file "$file" --output "$TEMP_EXTRACT" --quiet
        
        CODE_EXAMPLES=$(grep -c "\`\`\`rust" "$file" || echo "0")
        TOTAL_CODE_EXAMPLES=$((TOTAL_CODE_EXAMPLES + CODE_EXAMPLES))
        
        echo -e "${BLUE}Verifying code examples...${RESET}"
        if [[ -f "$SCRIPT_DIR/code-example-verifier.sh" ]]; then
            "$SCRIPT_DIR/code-example-verifier.sh" --file "$TEMP_EXTRACT" --output "$TEMP_VERIFY" --quiet
            
            CODE_ISSUES=$(grep -c "ERROR" "$TEMP_VERIFY" || echo "0")
            TOTAL_CODE_ISSUES=$((TOTAL_CODE_ISSUES + CODE_ISSUES))
            
            # Add code validation to report
            FILE_SECTION+="#### Code Examples\n\n"
            FILE_SECTION+="- Total Examples: $CODE_EXAMPLES\n"
            FILE_SECTION+="- Issues Found: $CODE_ISSUES\n\n"
            
            if [[ "$CODE_ISSUES" -gt 0 ]]; then
                FILE_SECTION+="<details>\n<summary>Code Issues</summary>\n\n"
                FILE_SECTION+="$(cat "$TEMP_VERIFY" | sed 's/^/    /')\n\n"
                FILE_SECTION+="</details>\n\n"
            fi
        else
            echo -e "${YELLOW}Warning: code-example-verifier.sh not found, skipping code verification${RESET}"
            FILE_SECTION+="#### Code Examples\n\n"
            FILE_SECTION+="- Total Examples: $CODE_EXAMPLES\n"
            FILE_SECTION+="- Code verification skipped (tool not available)\n\n"
        fi
    else
        echo -e "${YELLOW}Warning: code-example-extractor.sh not found, skipping code extraction${RESET}"
        FILE_SECTION+="#### Code Examples\n\n"
        FILE_SECTION+="- Code extraction skipped (tool not available)\n\n"
    fi
    
    # Analyze links
    echo -e "${BLUE}Analyzing links...${RESET}"
    if [[ -f "$SCRIPT_DIR/link-analyzer.sh" ]]; then
        "$SCRIPT_DIR/link-analyzer.sh" --file "$file" --output "$TEMP_LINKS" --quiet
        
        LINKS=$(grep -c "\[.*\](.*)" "$file" || echo "0")
        TOTAL_LINKS=$((TOTAL_LINKS + LINKS))
        
        LINK_ISSUES=$(grep -c "ERROR" "$TEMP_LINKS" || echo "0")
        TOTAL_LINK_ISSUES=$((TOTAL_LINK_ISSUES + LINK_ISSUES))
        
        # Add link validation to report
        FILE_SECTION+="#### Links\n\n"
        FILE_SECTION+="- Total Links: $LINKS\n"
        FILE_SECTION+="- Issues Found: $LINK_ISSUES\n\n"
        
        if [[ "$LINK_ISSUES" -gt 0 ]]; then
            FILE_SECTION+="<details>\n<summary>Link Issues</summary>\n\n"
            FILE_SECTION+="$(cat "$TEMP_LINKS" | sed 's/^/    /')\n\n"
            FILE_SECTION+="</details>\n\n"
        fi
    else
        echo -e "${YELLOW}Warning: link-analyzer.sh not found, skipping link analysis${RESET}"
        FILE_SECTION+="#### Links\n\n"
        FILE_SECTION+="- Link analysis skipped (tool not available)\n\n"
    fi
    
    # Check document structure
    echo -e "${BLUE}Validating document structure...${RESET}"
    if [[ -f "$SCRIPT_DIR/document-validator.sh" ]]; then
        "$SCRIPT_DIR/document-validator.sh" --file "$file" --output "$TEMP_STRUCTURE" --quiet
        
        # Count frontmatter issues
        FRONTMATTER_ISSUES=$(grep -c "frontmatter" "$TEMP_STRUCTURE" || echo "0")
        TOTAL_FRONTMATTER_ISSUES=$((TOTAL_FRONTMATTER_ISSUES + FRONTMATTER_ISSUES))
        
        # Count structure issues
        STRUCTURE_ISSUES=$(grep -c "structure\|section\|heading" "$TEMP_STRUCTURE" || echo "0")
        TOTAL_STRUCTURE_ISSUES=$((TOTAL_STRUCTURE_ISSUES + STRUCTURE_ISSUES))
        
        # Add structure validation to report
        FILE_SECTION+="#### Document Structure\n\n"
        FILE_SECTION+="- Frontmatter Issues: $FRONTMATTER_ISSUES\n"
        FILE_SECTION+="- Structure Issues: $STRUCTURE_ISSUES\n\n"
        
        if [[ "$FRONTMATTER_ISSUES" -gt 0 || "$STRUCTURE_ISSUES" -gt 0 ]]; then
            FILE_SECTION+="<details>\n<summary>Structure Issues</summary>\n\n"
            FILE_SECTION+="$(cat "$TEMP_STRUCTURE" | sed 's/^/    /')\n\n"
            FILE_SECTION+="</details>\n\n"
        fi
    else
        echo -e "${YELLOW}Warning: document-validator.sh not found, skipping structure validation${RESET}"
        FILE_SECTION+="#### Document Structure\n\n"
        FILE_SECTION+="- Structure validation skipped (tool not available)\n\n"
    fi
    
    # Determine overall validation status for this file
    TOTAL_ISSUES=$((CODE_ISSUES + LINK_ISSUES + FRONTMATTER_ISSUES + STRUCTURE_ISSUES))
    if [[ "$TOTAL_ISSUES" -eq 0 ]]; then
        FILE_SECTION+="#### Overall Status: ${GREEN}PASS${RESET}\n\n"
        VALIDATION_SUCCESS=$((VALIDATION_SUCCESS + 1))
        echo -e "${GREEN}Validation successful!${RESET}"
    else
        FILE_SECTION+="#### Overall Status: ${RED}FAIL${RESET} ($TOTAL_ISSUES issues found)\n\n"
        VALIDATION_FAILURE=$((VALIDATION_FAILURE + 1))
        echo -e "${RED}Validation failed with $TOTAL_ISSUES issues!${RESET}"
    fi
    
    # Add file section to report
    echo -e "$FILE_SECTION" >> "$REPORT_FILE"
    
    # Clean up temp files
    rm -rf "$TEMP_DIR"
done

# Update summary in the report
SUMMARY=$(cat << EOF
| Category | Count |
|----------|-------|
| Documents Validated | ${#FILES_TO_VALIDATE[@]} |
| Code Examples Found | $TOTAL_CODE_EXAMPLES |
| Links Found | $TOTAL_LINKS |
| Frontmatter Issues | $TOTAL_FRONTMATTER_ISSUES |
| Structure Issues | $TOTAL_STRUCTURE_ISSUES |
| Code Issues | $TOTAL_CODE_ISSUES |
| Link Issues | $TOTAL_LINK_ISSUES |
| Overall Validation Status | $(if [[ "$VALIDATION_FAILURE" -eq 0 ]]; then echo "✅ PASS"; else echo "❌ FAIL ($VALIDATION_FAILURE/$((VALIDATION_SUCCESS + VALIDATION_FAILURE)) documents failed)"; fi) |
EOF
)

# Replace the summary section in the report
# Create a temporary file with the updated content
{
    # Add everything before the summary section
    sed -n '1,/^| Documents Validated/p' "$REPORT_FILE" | sed '$d'
    
    # Add the new summary
    echo "$SUMMARY"
    
    # Add everything after the summary section
    sed -n '/^| Overall Validation Status/,$p' "$REPORT_FILE" | sed '1,2d'
} > "${REPORT_FILE}.new"

# Replace the original file with the updated one
mv "${REPORT_FILE}.new" "$REPORT_FILE"

# Add recommendations section
cat >> "$REPORT_FILE" << EOF

## Recommendations

Based on the validation results, the following recommendations are provided:

$(if [[ "$TOTAL_CODE_ISSUES" -gt 0 ]]; then echo "- Fix code examples in documents with code issues"; fi)
$(if [[ "$TOTAL_LINK_ISSUES" -gt 0 ]]; then echo "- Update internal links to the correct paths"; fi)
$(if [[ "$TOTAL_FRONTMATTER_ISSUES" -gt 0 ]]; then echo "- Fix frontmatter issues to ensure proper metadata"; fi)
$(if [[ "$TOTAL_STRUCTURE_ISSUES" -gt 0 ]]; then echo "- Address document structure issues to meet documentation standards"; fi)
$(if [[ "$TOTAL_CODE_ISSUES" -eq 0 && "$TOTAL_LINK_ISSUES" -eq 0 && "$TOTAL_FRONTMATTER_ISSUES" -eq 0 && "$TOTAL_STRUCTURE_ISSUES" -eq 0 ]]; then echo "- All documents passed validation. No immediate action needed."; fi)

## Next Steps

1. Update the validation tracking document with the results from this report
2. Prioritize fixing issues based on document importance
3. Re-run validation after fixes to verify improvements

## Related Documents

- [Phase 2 Completion Plan](./phase2-completion-plan.md)
- [Documentation Reorganization Roadmap](../30_documentation-reorganization-roadmap.md)
- [Validation Tracking Template](./validation-tracking-template.md)
EOF

echo -e "\n${BOLD}${GREEN}Validation complete!${RESET}"
echo -e "Report generated at: ${BOLD}$REPORT_FILE${RESET}"

# Print summary results
echo -e "\n${BOLD}Summary:${RESET}"
echo -e "Documents Validated: ${#FILES_TO_VALIDATE[@]}"
echo -e "Code Examples Found: $TOTAL_CODE_EXAMPLES"
echo -e "Links Found: $TOTAL_LINKS"
echo -e "Frontmatter Issues: $TOTAL_FRONTMATTER_ISSUES"
echo -e "Structure Issues: $TOTAL_STRUCTURE_ISSUES"
echo -e "Code Issues: $TOTAL_CODE_ISSUES"
echo -e "Link Issues: $TOTAL_LINK_ISSUES"

if [[ "$VALIDATION_FAILURE" -eq 0 ]]; then
    echo -e "\n${BOLD}${GREEN}Overall Status: PASS${RESET}"
else
    echo -e "\n${BOLD}${RED}Overall Status: FAIL ($VALIDATION_FAILURE/${#FILES_TO_VALIDATE[@]} documents failed)${RESET}"
fi

exit 0 