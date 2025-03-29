#!/bin/bash
#
# clean-duplicates.sh
#
# Removes duplicate sections from markdown files
#
# Usage:
#   ./clean-duplicates.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet]
#

# Handle arguments
FILE=""
OUTPUT=""
DRY_RUN=false
QUIET=false

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

# Define patterns for common section headings
SECTION_HEADINGS=(
    "## Overview"
    "## Details"
    "## Examples"
    "## Related Information"
    "## Related Documents"
    "## References"
)

# Initialize variables
DUPLICATE_SECTIONS_FOUND=false
SECTIONS_TO_FIX=()

# Check each section heading for duplicates
for heading in "${SECTION_HEADINGS[@]}"; do
    # Count occurrences of the section heading
    COUNT=$(grep -c "^$heading$" "$FILE" || true)
    
    if [ "$COUNT" -gt 1 ]; then
        DUPLICATE_SECTIONS_FOUND=true
        
        # Get line numbers where duplicates occur
        LINE_NUMBERS=($(grep -n "^$heading$" "$FILE" | cut -d: -f1))
        
        if [[ "$QUIET" == "false" ]]; then
            echo "Found duplicate section: $heading"
            echo "  Line numbers: ${LINE_NUMBERS[*]}"
        fi
        
        # Store the first occurrence line number for fixing
        SECTIONS_TO_FIX+=("$heading:${LINE_NUMBERS[0]}")
    fi
done

# If no duplicate sections found, exit
if [ "$DUPLICATE_SECTIONS_FOUND" = false ]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "No duplicate sections found in: $FILE"
    fi
    exit 0
fi

# If dry run, just report what would be done
if [ "$DRY_RUN" = true ]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "Would fix duplicate sections in: $FILE"
        echo "  Sections to keep up to first occurrence:"
        for section_info in "${SECTIONS_TO_FIX[@]}"; do
            heading=$(echo "$section_info" | cut -d: -f1)
            line=$(echo "$section_info" | cut -d: -f2)
            echo "    - $heading (Line $line)"
        done
    fi
    exit 0
fi

# Create temporary file
TMP_FILE=$(mktemp)

# Use the first duplicate section to determine the cutoff point
# We'll keep everything up to the first occurrence of the duplicate section
if [ ${#SECTIONS_TO_FIX[@]} -gt 0 ]; then
    FIRST_SECTION=$(echo "${SECTIONS_TO_FIX[0]}" | cut -d: -f1)
    FIRST_LINE=$(echo "${SECTIONS_TO_FIX[0]}" | cut -d: -f2)
    
    # Keep content up to the first occurrence of the section + a few lines for the section's content
    LINES_TO_KEEP=$((FIRST_LINE + 5))
    
    # Get the next section after this one, if available
    NEXT_SECTION_LINE=$(grep -n "^## " "$FILE" | awk -F: '$1 > '"$FIRST_LINE"'' | head -1 | cut -d: -f1 || echo "")
    
    if [ -n "$NEXT_SECTION_LINE" ]; then
        # Keep content up to the next section
        LINES_TO_KEEP=$NEXT_SECTION_LINE
    fi
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Keeping content up to line $LINES_TO_KEEP"
    fi
    
    # Create a fixed version of the file that keeps content up to the determined line
    head -n "$LINES_TO_KEEP" "$FILE" > "$TMP_FILE"
    
    # Set output file
    if [ -z "$OUTPUT" ]; then
        mv "$TMP_FILE" "$FILE"
        if [[ "$QUIET" == "false" ]]; then
            echo "Fixed duplicate sections in: $FILE"
        fi
    else
        mv "$TMP_FILE" "$OUTPUT"
        if [[ "$QUIET" == "false" ]]; then
            echo "Wrote fixed file to: $OUTPUT"
        fi
    fi
else
    # This shouldn't happen, but just in case
    if [[ "$QUIET" == "false" ]]; then
        echo "Error: Couldn't determine how to fix duplicate sections"
    fi
    exit 1
fi

exit 0 