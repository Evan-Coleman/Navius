#!/bin/bash
#
# find-duplicates.sh
#
# Finds markdown files with duplicate standard sections
#
# Usage:
#   ./find-duplicates.sh [--fix] <directory>
#

# Check if the correct number of arguments is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 [--fix] <directory>"
    echo "  --fix : Optional flag to automatically fix all duplicates"
    exit 1
fi

# Initialize variables
AUTO_FIX=false
DIRECTORY=""

# Parse arguments
for arg in "$@"; do
    if [ "$arg" == "--fix" ]; then
        AUTO_FIX=true
    else
        DIRECTORY="$arg"
    fi
done

# Verify directory exists
if [ ! -d "$DIRECTORY" ]; then
    echo "Error: $DIRECTORY is not a directory"
    exit 1
fi

echo "Searching for duplicate sections in $DIRECTORY..."
echo ""

# Define common section headings that might be duplicated
SECTION_HEADINGS=(
    "## Overview"
    "## Details"
    "## Examples"
    "## Related Information"
    "## Related Documents"
    "## References"
)

# Initialize counters
TOTAL_FILES=0
FILES_WITH_DUPLICATES=0
FILES_FIXED=0

# Get all markdown files
MD_FILES=$(find "$DIRECTORY" -name "*.md" -type f)
TOTAL_FILES=$(echo "$MD_FILES" | wc -l)

# Process each markdown file
for file in $MD_FILES; do
    HAS_DUPLICATES=false
    DUPLICATE_SECTIONS=""

    # Check for each section heading
    for heading in "${SECTION_HEADINGS[@]}"; do
        # Count occurrences of the section heading
        COUNT=$(grep -c "^$heading$" "$file")
        
        if [ "$COUNT" -gt 1 ]; then
            HAS_DUPLICATES=true
            FILES_WITH_DUPLICATES=$((FILES_WITH_DUPLICATES + 1))
            
            # Get line numbers where duplicates occur
            LINE_NUMBERS=$(grep -n "^$heading$" "$file" | cut -d ':' -f1 | tr '\n' ' ')
            
            echo "File: $file"
            echo "  Duplicate section: $heading"
            echo "  Line numbers: $LINE_NUMBERS"
            echo ""
            
            # Store this information for fixing
            DUPLICATE_SECTIONS="$DUPLICATE_SECTIONS$heading:$LINE_NUMBERS;"
            break  # Only report the first duplicate section type found
        fi
    done
    
    # Fix duplicates if found
    if [ "$HAS_DUPLICATES" = true ] && [ "$AUTO_FIX" = true ]; then
        echo "Fixing duplicates in: $file"
        
        # Extract the first duplicate section and its line numbers
        SECTION_INFO=$(echo "$DUPLICATE_SECTIONS" | cut -d ';' -f1)
        SECTION_NAME=$(echo "$SECTION_INFO" | cut -d ':' -f1)
        
        # Get line numbers array (compatible with older bash versions)
        LINE_NUMBERS_STR=$(echo "$SECTION_INFO" | cut -d ':' -f2)
        FIRST_OCCURRENCE=$(echo "$LINE_NUMBERS_STR" | awk '{print $1}')
        
        # Create a temporary file with content up to the first occurrence
        awk -v line="$FIRST_OCCURRENCE" '{
            if (NR <= line) {
                print $0;
            }
        }' "$file" > "${file}.tmp"
        
        # Replace the original file with the fixed content
        mv "${file}.tmp" "$file"
        
        FILES_FIXED=$((FILES_FIXED + 1))
        echo "  Fixed: Kept content up to line $FIRST_OCCURRENCE"
        echo ""
    fi
done

echo "Scan complete!"
echo "-----------------"
echo "Total files scanned: $TOTAL_FILES"
echo "Files with duplicate sections: $FILES_WITH_DUPLICATES"
echo "Files fixed: $FILES_FIXED"
echo "" 