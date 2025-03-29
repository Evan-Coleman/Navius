#!/bin/bash
#
# missing-sections-report.sh
#
# Scans markdown files and generates a report of missing required sections
#
# Usage:
#   ./missing-sections-report.sh <directory> [--output <report_file>] [--verbose]
#

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Handle arguments
TARGET=""
OUTPUT_FILE="$SCRIPT_DIR/missing-sections-report.md"
VERBOSE=false

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --output)
            # If the output is a relative path, make it relative to the script directory
            if [[ "${2:0:1}" != "/" ]]; then
                OUTPUT_FILE="$SCRIPT_DIR/$2"
            else
                # If it's an absolute path, use it as is
                OUTPUT_FILE="$2"
            fi
            shift
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            if [[ -z "$TARGET" ]]; then
                TARGET="$1"
            else
                echo "Unknown option: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if target exists
if [[ -z "$TARGET" ]]; then
    echo "Error: No target directory specified"
    echo "Usage: ./missing-sections-report.sh <directory> [--output <report_file>] [--verbose]"
    exit 1
fi

if [[ ! -e "$TARGET" ]]; then
    echo "Error: Target not found: $TARGET"
    exit 1
fi

# Check if target is a directory
if [[ ! -d "$TARGET" ]]; then
    echo "Error: Target must be a directory: $TARGET"
    exit 1
fi

# Initialize report file
echo "---" > "$OUTPUT_FILE"
echo "title: Missing Sections Report" >> "$OUTPUT_FILE"
echo "description: Report of markdown files missing required sections" >> "$OUTPUT_FILE"
echo "category: Documentation" >> "$OUTPUT_FILE"
echo "last_updated: $(date +'%B %d, %Y')" >> "$OUTPUT_FILE"
echo "---" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "# Missing Sections Report" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "This report identifies markdown files that are missing standard required sections. Generated on $(date +'%B %d, %Y')." >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "## Summary" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Find all markdown files in the target directory
MARKDOWN_FILES=$(find "$TARGET" -name "*.md" -type f | sort)
TOTAL_FILES=$(echo "$MARKDOWN_FILES" | wc -l)

# Counters for summary
COMPLIANT_FILES=0
NON_COMPLIANT_FILES=0
TOTAL_MISSING_SECTIONS=0

# Use regular arrays instead of associative arrays for compatibility
declare -a CATEGORIES
declare -a CATEGORY_COUNTS
declare -a CATEGORY_MISSING

# Store compliant files
COMPLIANT_LIST=""

# Process each file
ISSUE_LIST=""
while IFS= read -r FILE; do
    if [[ "$VERBOSE" == "true" ]]; then
        echo "Checking: $FILE"
    fi
    
    # Determine document type based on path
    FILE_CATEGORY="General Documentation"
    REQUIRED_SECTIONS=()
    
    if [[ "$FILE" == *"/01_getting_started/"* ]]; then
        # Getting Started docs require these sections
        FILE_CATEGORY="Getting Started"
        REQUIRED_SECTIONS=("## Prerequisites" "## Installation" "## Configuration" "## Next Steps" "## Troubleshooting")
        SECTION_ALTERNATIVES=("## Requirements" "## Before You Begin" "## Setup" "## Installing" "## Settings" "## Configure" "## What's Next" "## Further Steps" "## Common Issues" "## Problems" "## FAQ")
    elif [[ "$FILE" == *"/02_examples/"* ]]; then
        # Examples require these sections
        FILE_CATEGORY="Examples"
        REQUIRED_SECTIONS=("## Overview" "## Prerequisites" "## Setup" "## Step-by-Step Guide" "## Complete Example" "## Next Steps")
        SECTION_ALTERNATIVES=("## Introduction" "## Summary" "## Requirements" "## Before You Begin" "## Preparation" "## Environment Setup" "## Instructions" "## Tutorial" "## Walkthrough" "## Full Example" "## Working Example" "## Code Example" "## What's Next" "## Further Steps" "## See Also")
    elif [[ "$FILE" == *"/05_reference/"* ]]; then
        # API Reference docs require these sections
        FILE_CATEGORY="API Reference"
        REQUIRED_SECTIONS=("## Overview" "## API" "## Examples" "## Best Practices" "## Related Topics")
        SECTION_ALTERNATIVES=("## Introduction" "## Summary" "## Interface" "## Methods" "## Functions" "## Example Usage" "## Sample Code" "## Code Examples" "## Recommendations" "## Guidelines" "## Related Information" "## See Also" "## References" "## Related Documents")
    else
        # Default sections for any document
        FILE_CATEGORY="General Documentation"
        REQUIRED_SECTIONS=("## Overview" "## Details" "## Examples" "## Related Information")
        SECTION_ALTERNATIVES=("## Introduction" "## Summary" "## Information" "## Detailed Description" "## Example Usage" "## Sample Code" "## Code Examples" "## Related Documents" "## See Also" "## References" "## Related Topics")
    fi
    
    # Special case: check if "Related Documents" is present
    # If the file contains "## Related Documents", we should consider the "Related Information" or "Related Topics" requirements satisfied
    RELATED_DOCS_PRESENT=false
    if grep -q "^## Related Documents" "$FILE"; then
        RELATED_DOCS_PRESENT=true
    fi
    
    # Get all section headings actually in the file
    SECTION_HEADERS=$(grep "^## " "$FILE" || echo "")
    
    # Check for existing sections (including alternative names)
    MISSING_SECTIONS=()
    
    # Check each required section
    for SECTION in "${REQUIRED_SECTIONS[@]}"; do
        # Skip Related Information/Topics check if Related Documents is present
        if [[ "$RELATED_DOCS_PRESENT" == "true" && ("$SECTION" == "## Related Information" || "$SECTION" == "## Related Topics") ]]; then
            continue
        fi
        
        # Check if this section or an alternative exists
        SECTION_FOUND=false
        
        # Check for exact match
        if echo "$SECTION_HEADERS" | grep -q "^$SECTION$"; then
            SECTION_FOUND=true
        else
            # Check for alternatives
            for ALT in "${SECTION_ALTERNATIVES[@]}"; do
                if [[ "$ALT" == "$SECTION"* ]] && echo "$SECTION_HEADERS" | grep -q "^$ALT$"; then
                    SECTION_FOUND=true
                    break
                fi
            done
        fi
        
        # If section not found, add to missing list
        if [[ "$SECTION_FOUND" == "false" ]]; then
            MISSING_SECTIONS+=("$SECTION")
        fi
    done
    
    # Count missing sections
    MISSING_COUNT=${#MISSING_SECTIONS[@]}
    TOTAL_MISSING_SECTIONS=$((TOTAL_MISSING_SECTIONS + MISSING_COUNT))
    
    if [[ "$MISSING_COUNT" -eq 0 ]]; then
        COMPLIANT_FILES=$((COMPLIANT_FILES + 1))
        
        # Create a relative path for display
        REL_PATH="${FILE#$(pwd)/}"
        if [[ "$REL_PATH" == "$FILE" ]]; then
            # If path wasn't shortened, just use the filename
            REL_PATH=$(basename "$FILE")
        fi
        
        # Add to compliant list
        COMPLIANT_LIST+="- **$REL_PATH** ($FILE_CATEGORY)\n"
    else
        NON_COMPLIANT_FILES=$((NON_COMPLIANT_FILES + 1))
        
        # Create a relative path for display
        REL_PATH="${FILE#$(pwd)/}"
        if [[ "$REL_PATH" == "$FILE" ]]; then
            # If path wasn't shortened, just use the filename
            REL_PATH=$(basename "$FILE")
        fi
        
        # Add to issue list
        ISSUE_LIST+="### $REL_PATH\n\n"
        ISSUE_LIST+="**Category:** $FILE_CATEGORY\n\n"
        ISSUE_LIST+="**Missing Sections:**\n\n"
        
        # Add each missing section as a bullet point
        for SECTION in "${MISSING_SECTIONS[@]}"; do
            ISSUE_LIST+="- $SECTION\n"
        done
        
        ISSUE_LIST+="\n"
        
        # Update category counts - find if category exists in array
        CATEGORY_INDEX=-1
        for i in "${!CATEGORIES[@]}"; do
            if [[ "${CATEGORIES[$i]}" == "$FILE_CATEGORY" ]]; then
                CATEGORY_INDEX=$i
                break
            fi
        done
        
        if [[ "$CATEGORY_INDEX" -eq -1 ]]; then
            # Add new category
            CATEGORIES+=("$FILE_CATEGORY")
            CATEGORY_COUNTS+=(1)
            CATEGORY_MISSING+=("$MISSING_COUNT")
        else
            # Update existing category
            CATEGORY_COUNTS[$CATEGORY_INDEX]=$((CATEGORY_COUNTS[$CATEGORY_INDEX] + 1))
            CATEGORY_MISSING[$CATEGORY_INDEX]=$((CATEGORY_MISSING[$CATEGORY_INDEX] + MISSING_COUNT))
        fi
    fi
done <<< "$MARKDOWN_FILES"

# Write summary statistics to report
echo "- **Total Files Scanned**: $TOTAL_FILES" >> "$OUTPUT_FILE"
echo "- **Compliant Files**: $COMPLIANT_FILES" >> "$OUTPUT_FILE"
echo "- **Non-Compliant Files**: $NON_COMPLIANT_FILES" >> "$OUTPUT_FILE"
echo "- **Total Missing Sections**: $TOTAL_MISSING_SECTIONS" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# Add category breakdown
echo "## Category Breakdown" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "| Category | Non-Compliant Files | Missing Sections |" >> "$OUTPUT_FILE"
echo "|----------|---------------------|------------------|" >> "$OUTPUT_FILE"

for i in "${!CATEGORIES[@]}"; do
    echo "| ${CATEGORIES[$i]} | ${CATEGORY_COUNTS[$i]} | ${CATEGORY_MISSING[$i]} |" >> "$OUTPUT_FILE"
done

echo "" >> "$OUTPUT_FILE"

# Add detailed list of issues
echo "## Files Missing Sections" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo -e "$ISSUE_LIST" >> "$OUTPUT_FILE"

# Add list of compliant files
echo "## Compliant Files" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
if [[ "$COMPLIANT_FILES" -eq 0 ]]; then
    echo "No fully compliant files found." >> "$OUTPUT_FILE"
else
    echo "The following files have all required sections:" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo -e "$COMPLIANT_LIST" >> "$OUTPUT_FILE"
fi
echo "" >> "$OUTPUT_FILE"

# Add recommendations section
echo "## Recommendations" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "Based on the findings in this report, we recommend:" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "1. Prioritize adding missing sections to the ${NON_COMPLIANT_FILES} non-compliant files" >> "$OUTPUT_FILE"
echo "2. Focus first on files with the most missing sections" >> "$OUTPUT_FILE"
echo "3. Ensure that all files follow the documentation standards for their category" >> "$OUTPUT_FILE"
echo "4. Run this report regularly to track progress" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo "## Related Documents" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "- [Documentation Standards](/11newdocs11/05_reference/standards/documentation-standards.md)" >> "$OUTPUT_FILE"
echo "- [Documentation Reorganization Roadmap](/11newdocs11/98_roadmaps/30_documentation-reorganization-roadmap.md)" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

echo "Report generated on $(date +'%B %d, %Y at %H:%M:%S')." >> "$OUTPUT_FILE"

# Print completion message
# Calculate relative path to the current directory
RELATIVE_OUTPUT_PATH="$OUTPUT_FILE"
if [[ "$OUTPUT_FILE" == "$(pwd)"* ]]; then
    RELATIVE_OUTPUT_PATH="${OUTPUT_FILE#$(pwd)/}"
fi

echo "Report generated: $RELATIVE_OUTPUT_PATH"
echo "- Total files scanned: $TOTAL_FILES"
echo "- Compliant files: $COMPLIANT_FILES"
echo "- Non-compliant files: $NON_COMPLIANT_FILES"
echo "- Total missing sections: $TOTAL_MISSING_SECTIONS"

exit 0 