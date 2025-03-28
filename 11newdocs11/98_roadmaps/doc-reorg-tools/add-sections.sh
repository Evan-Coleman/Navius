#!/bin/bash
#
# add-sections.sh
#
# Checks for missing required sections in markdown files and adds them if needed
#
# Usage:
#   ./add-sections.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet] [--verbose]
#

# Handle arguments
FILE=""
OUTPUT=""
DRY_RUN=false
QUIET=false
VERBOSE=false

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        --file)
            FILE="$2"
            shift
            shift
            ;;
        --output)
            OUTPUT="$2"
            shift
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --quiet)
            QUIET=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            if [[ "$QUIET" == "false" ]]; then
                echo "Unknown option: $1"
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if file exists
if [[ ! -f "$FILE" ]]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "File not found: $FILE"
    fi
    exit 1
fi

# Define required sections based on file type/path
REQUIRED_SECTIONS=()
SECTION_TEMPLATES=()
SECTION_ALTERNATIVES=()
# Define section positions (1=top, 2=middle, 3=bottom)
SECTION_POSITIONS=()

# Determine document type based on path
if [[ "$FILE" == *"/01_getting_started/"* ]]; then
    # Getting Started docs require these sections
    REQUIRED_SECTIONS+=("## Prerequisites" "## Installation" "## Configuration" "## Next Steps" "## Troubleshooting")
    SECTION_ALTERNATIVES+=("## Requirements|## Before You Begin" "## Setup|## Installing" "## Settings|## Configure" "## What's Next|## Further Steps" "## Common Issues|## Problems|## FAQ")
    SECTION_TEMPLATES+=("Before you begin, ensure you have the following installed:\n\n- Requirement 1\n- Requirement 2\n"
                        "Follow these steps to install the software:\n\n1. Step one\n2. Step two\n"
                        "Configure your installation with the following settings:\n\n```yaml\n# Configuration example\n```\n"
                        "Now that you've completed the setup, you can:\n\n- Try the [hello world example](../02_examples/hello-world.md)\n- Explore [additional examples](../02_examples/)\n"
                        "If you encounter issues during installation or configuration:\n\n### Common Issues\n\n- **Problem**: Description\n  **Solution**: Fix\n")
    # Position indicators: 1=top (after intro), 2=middle, 3=bottom
    SECTION_POSITIONS+=(1 2 2 3 3)
elif [[ "$FILE" == *"/02_examples/"* ]]; then
    # Examples require these sections
    REQUIRED_SECTIONS+=("## Overview" "## Prerequisites" "## Setup" "## Step-by-Step Guide" "## Complete Example" "## Next Steps")
    SECTION_ALTERNATIVES+=("## Introduction|## Summary" "## Requirements|## Before You Begin" "## Preparation|## Environment Setup" "## Instructions|## Tutorial|## Walkthrough" "## Full Example|## Working Example|## Code Example" "## What's Next|## Further Steps|## See Also")
    SECTION_TEMPLATES+=("This example demonstrates how to use X to accomplish Y.\n"
                        "Before starting this example, ensure you have:\n\n- Requirement 1\n- Requirement 2\n"
                        "Prepare your environment with the following:\n\n```bash\n# Setup commands\n```\n"
                        "Follow these steps to complete the example:\n\n1. First step\n2. Second step\n"
                        "Here's the complete working example:\n\n```rust\n// Complete code example\n```\n"
                        "Now that you've completed this example, you might want to:\n\n- Try [related example](./related-example.md)\n- Learn about [advanced topic](../05_reference/advanced-topic.md)\n")
    SECTION_POSITIONS+=(1 1 2 2 2 3)
elif [[ "$FILE" == *"/05_reference/"* ]]; then
    # API Reference docs require these sections
    REQUIRED_SECTIONS+=("## Overview" "## API" "## Examples" "## Best Practices" "## Related Topics")
    SECTION_ALTERNATIVES+=("## Introduction|## Summary" "## Interface|## Methods|## Functions" "## Example Usage|## Sample Code|## Code Examples" "## Recommendations|## Guidelines" "## Related Information|## See Also|## References|## Related Documents")
    SECTION_TEMPLATES+=("This document provides reference information for X.\n"
                        "### Methods\n\n- : Description\n\n### Types\n\n- : Description\n"
                        "\n"
                        "When using this API, consider the following best practices:\n\n- Practice 1\n- Practice 2\n"
                        "- [Related API](./related-api.md)\n- [Implementation details](./implementation-details.md)\n")
    SECTION_POSITIONS+=(1 2 2 3 3)
else
    # Default sections for any document
    REQUIRED_SECTIONS+=("## Overview" "## Details" "## Examples" "## Related Information")
    SECTION_ALTERNATIVES+=("## Introduction|## Summary" "## Information|## Detailed Description" "## Example Usage|## Sample Code|## Code Examples" "## Related Documents|## See Also|## References|## Related Topics")
    SECTION_TEMPLATES+=("This document covers X.\n"
                        "Detailed information about the topic.\n"
                        "\n"
                        "- [Related document 1](./related1.md)\n- [Related document 2](./related2.md)\n")
    SECTION_POSITIONS+=(1 2 2 3)
fi

# Get file content
FILE_CONTENT=$(cat "$FILE")

# Debug info
if [[ "$VERBOSE" == "true" ]]; then
    echo "Analyzing file: $FILE"
    echo "Total lines: $(wc -l < "$FILE")"
    echo "Section headers:"
    grep -n "^## " "$FILE"
    echo "Required sections: ${REQUIRED_SECTIONS[*]}"
fi

# Special case: check if "Related Documents" is present
# If the file contains "## Related Documents", we should consider the "Related Information" or "Related Topics" requirements satisfied
RELATED_DOCS_PRESENT=false
if grep -q "^## Related Documents" "$FILE"; then
    RELATED_DOCS_PRESENT=true
    if [[ "$VERBOSE" == "true" ]]; then
        echo "Found '## Related Documents' section, which satisfies Related Information/Topics requirement"
    fi
fi

# Get all section headings actually in the file
SECTION_HEADERS=$(grep -n "^## " "$FILE" | cut -d: -f2-)

# Check for existing sections (including alternative names)
MISSING_SECTIONS=()
MISSING_SECTION_TEMPLATES=()
MISSING_SECTION_POSITIONS=()

for i in "${!REQUIRED_SECTIONS[@]}"; do
    SECTION="${REQUIRED_SECTIONS[$i]}"
    ALTERNATIVES="${SECTION_ALTERNATIVES[$i]}"
    
    # Skip Related Information/Topics check if Related Documents is present
    if [[ "$RELATED_DOCS_PRESENT" == "true" && ("$SECTION" == "## Related Information" || "$SECTION" == "## Related Topics") ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo "Skipping check for '$SECTION' since 'Related Documents' is present"
        fi
        continue
    fi
    
    # Convert alternatives to array
    IFS='|' read -ra ALT_ARRAY <<< "$ALTERNATIVES"
    
    # Check for the main section or any of its alternatives
    SECTION_FOUND=false
    
    # Check for main section
    if echo "$SECTION_HEADERS" | grep -q "^$SECTION\$"; then
        SECTION_FOUND=true
        if [[ "$VERBOSE" == "true" ]]; then
            echo "Found section: $SECTION"
        fi
    else
        # Check for alternatives
        for ALT in "${ALT_ARRAY[@]}"; do
            if echo "$SECTION_HEADERS" | grep -q "^$ALT\$"; then
                SECTION_FOUND=true
                if [[ "$VERBOSE" == "true" ]]; then
                    echo "Found alternative section: $ALT for $SECTION"
                fi
                break
            fi
        done
    fi
    
    # If section not found, add to missing list
    if [[ "$SECTION_FOUND" == "false" ]]; then
        MISSING_SECTIONS+=("$SECTION")
        MISSING_SECTION_TEMPLATES+=("${SECTION_TEMPLATES[$i]}")
        MISSING_SECTION_POSITIONS+=("${SECTION_POSITIONS[$i]}")
        if [[ "$VERBOSE" == "true" ]]; then
            echo "Missing section: $SECTION"
        fi
    fi
done

# If no missing sections, exit
if [ ${#MISSING_SECTIONS[@]} -eq 0 ]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "No missing sections in: $FILE"
    fi
    exit 0
fi

# If dry run, just report what would be done
if [[ "$DRY_RUN" == "true" ]]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "Would add the following sections to: $FILE"
        for SECTION in "${MISSING_SECTIONS[@]}"; do
            echo "  - $SECTION"
        done
    fi
else
    # If output file is not specified, create a temp file
    if [[ -z "$OUTPUT" ]]; then
        OUTPUT=$(mktemp)
    fi
    
    # Sort missing sections by position
    # Create associative arrays to hold section data based on position
    declare -a TOP_SECTIONS=()
    declare -a TOP_TEMPLATES=()
    declare -a MIDDLE_SECTIONS=()
    declare -a MIDDLE_TEMPLATES=()
    declare -a BOTTOM_SECTIONS=()
    declare -a BOTTOM_TEMPLATES=()
    
    for i in "${!MISSING_SECTIONS[@]}"; do
        case "${MISSING_SECTION_POSITIONS[$i]}" in
            1)
                TOP_SECTIONS+=("${MISSING_SECTIONS[$i]}")
                TOP_TEMPLATES+=("${MISSING_SECTION_TEMPLATES[$i]}")
                ;;
            2)
                MIDDLE_SECTIONS+=("${MISSING_SECTIONS[$i]}")
                MIDDLE_TEMPLATES+=("${MISSING_SECTION_TEMPLATES[$i]}")
                ;;
            3|*)
                BOTTOM_SECTIONS+=("${MISSING_SECTIONS[$i]}")
                BOTTOM_TEMPLATES+=("${MISSING_SECTION_TEMPLATES[$i]}")
                ;;
        esac
    done
    
    # Find appropriate insertion points
    
    # For TOP_SECTIONS: Insert after the first heading (# Title)
    # For MIDDLE_SECTIONS: Insert before any "## Related" sections, or at the end if none
    # For BOTTOM_SECTIONS: Insert at the end
    
    # First, get the first content heading line number
    FIRST_HEADING_LINE=$(grep -n "^# " "$FILE" | head -1 | cut -d: -f1)
    
    # Find the line number of the first "## Related" section, if any
    RELATED_SECTION_LINE=$(grep -n "^## Related" "$FILE" | head -1 | cut -d: -f1)
    
    # Total number of lines in the file
    TOTAL_LINES=$(wc -l < "$FILE")
    
    if [[ -z "$RELATED_SECTION_LINE" ]]; then
        # No related section, use end of file
        RELATED_SECTION_LINE=$TOTAL_LINES
    fi
    
    # Find the end of the first heading section (look for empty line after # Title)
    if [[ -n "$FIRST_HEADING_LINE" ]]; then
        # Add a small offset to get past the heading
        FIRST_HEADING_LINE=$((FIRST_HEADING_LINE + 1))
        
        # Look for the next empty line after the heading
        END_OF_INTRO=$(tail -n +$FIRST_HEADING_LINE "$FILE" | grep -n "^$" | head -1 | cut -d: -f1)
        
        if [[ -n "$END_OF_INTRO" ]]; then
            # We found an empty line, calculate its actual line number
            TOP_INSERT_POINT=$((FIRST_HEADING_LINE + END_OF_INTRO))
        else
            # No empty line found, use the heading line
            TOP_INSERT_POINT=$FIRST_HEADING_LINE
        fi
    else
        # If no main heading found, insert at line 1
        TOP_INSERT_POINT=1
    fi
    
    # Check for other h2 sections to determine middle insertion point
    FIRST_H2_LINE=$(grep -n "^## " "$FILE" | head -1 | cut -d: -f1)
    
    if [[ -n "$FIRST_H2_LINE" ]]; then
        # If there's at least one h2 section, use that as middle insert point
        MIDDLE_INSERT_POINT=$FIRST_H2_LINE
    else
        # No h2 sections, use same as top
        MIDDLE_INSERT_POINT=$TOP_INSERT_POINT
    fi
    
    # Now split the file at the insertion points and reconstruct it with new sections
    
    # Split the file into parts
    head -n $TOP_INSERT_POINT "$FILE" > "$OUTPUT"
    
    # Add top sections
    if [ ${#TOP_SECTIONS[@]} -gt 0 ]; then
        echo "" >> "$OUTPUT"  # Add a blank line before new sections
        for i in "${!TOP_SECTIONS[@]}"; do
            echo -e "${TOP_SECTIONS[$i]}\n${TOP_TEMPLATES[$i]}" >> "$OUTPUT"
        done
    fi
    
    # Add middle part of file up to related section
    if [[ $TOP_INSERT_POINT -lt $RELATED_SECTION_LINE ]]; then
        sed -n "$((TOP_INSERT_POINT+1)),$((RELATED_SECTION_LINE-1))p" "$FILE" >> "$OUTPUT"
    fi
    
    # Add middle sections
    if [ ${#MIDDLE_SECTIONS[@]} -gt 0 ]; then
        echo "" >> "$OUTPUT"  # Add a blank line before new sections
        for i in "${!MIDDLE_SECTIONS[@]}"; do
            echo -e "${MIDDLE_SECTIONS[$i]}\n${MIDDLE_TEMPLATES[$i]}" >> "$OUTPUT"
        done
    fi
    
    # Add the related section and anything after it
    sed -n "$RELATED_SECTION_LINE,\$p" "$FILE" >> "$OUTPUT"
    
    # Add bottom sections (if not already added)
    if [ ${#BOTTOM_SECTIONS[@]} -gt 0 ]; then
        # Check if file ends with newline
        if [[ $(tail -c 1 "$OUTPUT" | wc -l) -eq 0 ]]; then
            # No newline at end, add one
            echo "" >> "$OUTPUT"
        fi
        
        for i in "${!BOTTOM_SECTIONS[@]}"; do
            echo -e "\n${BOTTOM_SECTIONS[$i]}\n${BOTTOM_TEMPLATES[$i]}" >> "$OUTPUT"
        done
    fi
    
    # Replace original file if no output specified
    if [[ -z "$2" ]]; then
        mv "$OUTPUT" "$FILE"
        if [[ "$QUIET" == "false" ]]; then
            echo "Added missing sections to: $FILE"
            for SECTION in "${MISSING_SECTIONS[@]}"; do
                echo "  - Added: $SECTION"
            done
        fi
    else
        if [[ "$QUIET" == "false" ]]; then
            echo "Wrote file with added sections to: $OUTPUT"
        fi
    fi
fi

exit 0 