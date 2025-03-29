#!/bin/bash
#
# fix-duplicate-sections.sh
#
# Fixes files with duplicate sections appended at the end
#
# Usage:
#   ./fix-duplicate-sections.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet]
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

# Debug info
if [[ "$VERBOSE" == "true" ]]; then
    echo "Analyzing file: $FILE"
    echo "Total lines: $(wc -l < "$FILE")"
    echo "Section headers:"
    grep -n "^## " "$FILE"
fi

# Specific indicators of the template sections we add
TEMPLATE_INDICATORS=(
    "This document provides reference information for X"
    "When using this API, consider the following best practices"
    "Practice 1"
    "Practice 2"
    "This document covers X"
    "Detailed information about the topic"
    "\[Related document 1\]"
    "\[Related document 2\]"
)

# Search for any of our template indicators
for indicator in "${TEMPLATE_INDICATORS[@]}"; do
    LINE_NUM=$(grep -n "$indicator" "$FILE" | head -1 | cut -d: -f1 || echo "")
    
    if [[ -n "$LINE_NUM" ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo "Found template indicator '$indicator' at line $LINE_NUM"
        fi
        
        # Check if there are blank lines before this line (indicating a section break)
        PREV_LINE=$((LINE_NUM - 1))
        PREV_LINE_CONTENT=$(sed -n "${PREV_LINE}p" "$FILE")
        PREV_PREV_LINE=$((LINE_NUM - 2))
        PREV_PREV_LINE_CONTENT=$(sed -n "${PREV_PREV_LINE}p" "$FILE")
        
        if [[ -z "$PREV_LINE_CONTENT" && -z "$PREV_PREV_LINE_CONTENT" ]]; then
            # This might be a section break followed by our template content
            
            # Get nearest section header before this line
            PREV_SECTION=$(grep -n "^## " "$FILE" | awk -F: '$1 < '"$LINE_NUM"'' | tail -1)
            PREV_SECTION_LINE=$(echo "$PREV_SECTION" | cut -d: -f1 || echo "")
            PREV_SECTION_NAME=$(echo "$PREV_SECTION" | cut -d: -f2- || echo "")
            
            if [[ -n "$PREV_SECTION_LINE" && -n "$PREV_SECTION_NAME" ]]; then
                if [[ "$VERBOSE" == "true" ]]; then
                    echo "Previous section: '$PREV_SECTION_NAME' at line $PREV_SECTION_LINE"
                fi
                
                # Look for this section name earlier in the file too
                SECTION_COUNT=$(grep -c "^$PREV_SECTION_NAME$" "$FILE")
                
                if [[ "$SECTION_COUNT" -gt 1 ]]; then
                    # We found a duplicate section!
                    DUPLICATE_LINE=$PREV_SECTION_LINE
                    
                    if [[ "$QUIET" == "false" ]]; then
                        echo "Found duplicate section '$PREV_SECTION_NAME' at line $DUPLICATE_LINE"
                    fi
                    
                    # Find the previous occurrence of this section
                    FIRST_OCCURRENCE=$(grep -n "^$PREV_SECTION_NAME$" "$FILE" | head -1 | cut -d: -f1)
                    
                    if [[ "$VERBOSE" == "true" ]]; then
                        echo "First occurrence at line $FIRST_OCCURRENCE"
                    fi
                    
                    # If dry run, just report
                    if [[ "$DRY_RUN" == "true" ]]; then
                        if [[ "$QUIET" == "false" ]]; then
                            echo "Would remove content from line $DUPLICATE_LINE to end of file"
                        fi
                        continue
                    fi
                    
                    # Otherwise, fix the file
                    TMP_FILE=$(mktemp)
                    head -n $((DUPLICATE_LINE - 2)) "$FILE" > "$TMP_FILE"
                    
                    if [[ -z "$OUTPUT" ]]; then
                        mv "$TMP_FILE" "$FILE"
                        if [[ "$QUIET" == "false" ]]; then
                            echo "Removed duplicate sections from $FILE"
                        fi
                    else
                        mv "$TMP_FILE" "$OUTPUT"
                        if [[ "$QUIET" == "false" ]]; then
                            echo "Wrote file without duplicate sections to: $OUTPUT"
                        fi
                    fi
                    
                    exit 0
                fi
            fi
        fi
    fi
done

# If no templates found, try another approach - find sections that appear more than once
DUPLICATE_FOUND=false
DUPLICATE_LINE=0

# Get all section headings
SECTION_HEADERS=$(grep -n "^## " "$FILE")

# Extract just the names
SECTION_NAMES=$(echo "$SECTION_HEADERS" | cut -d: -f2- | sort)

# Find duplicates
DUPLICATE_SECTIONS=$(echo "$SECTION_NAMES" | uniq -d)

if [[ -n "$DUPLICATE_SECTIONS" ]]; then
    if [[ "$VERBOSE" == "true" ]]; then
        echo "Found duplicate section names:"
        echo "$DUPLICATE_SECTIONS"
    fi
    
    # Take the first duplicate section
    DUPLICATE_SECTION=$(echo "$DUPLICATE_SECTIONS" | head -1)
    
    # Find all occurrences
    OCCURRENCES=$(grep -n "^$DUPLICATE_SECTION$" "$FILE" | cut -d: -f1)
    
    # Get the second occurrence
    SECOND_OCCURRENCE=$(echo "$OCCURRENCES" | sed -n '2p')
    
    if [[ -n "$SECOND_OCCURRENCE" ]]; then
        DUPLICATE_FOUND=true
        DUPLICATE_LINE=$SECOND_OCCURRENCE
        
        if [[ "$QUIET" == "false" ]]; then
            echo "Found duplicate section '$DUPLICATE_SECTION' at line $DUPLICATE_LINE"
        fi
        
        # If dry run, just report
        if [[ "$DRY_RUN" == "true" ]]; then
            if [[ "$QUIET" == "false" ]]; then
                echo "Would remove content from line $DUPLICATE_LINE to end of file"
            fi
            exit 0
        fi
        
        # Otherwise, fix the file
        TMP_FILE=$(mktemp)
        head -n $((DUPLICATE_LINE - 2)) "$FILE" > "$TMP_FILE"
        
        if [[ -z "$OUTPUT" ]]; then
            mv "$TMP_FILE" "$FILE"
            if [[ "$QUIET" == "false" ]]; then
                echo "Removed duplicate sections from $FILE"
            fi
        else
            mv "$TMP_FILE" "$OUTPUT"
            if [[ "$QUIET" == "false" ]]; then
                echo "Wrote file without duplicate sections to: $OUTPUT"
            fi
        fi
        
        exit 0
    fi
fi

# Special case for the specific pattern you showed - look for the Overview section that appears at certain position
# First, check if we have a file over 150 lines (most files with duplicates are lengthy)
FILE_LINES=$(wc -l < "$FILE")

if [[ "$FILE_LINES" -gt 150 ]]; then
    # Look for "## Overview" around line 170-190 which is typical for the duplicate pattern
    OVERVIEW_LINE=$(grep -n "^## Overview$" "$FILE" | awk '$1 > 150 && $1 < 190 {print $0}' | cut -d: -f1 || echo "")
    
    if [[ -n "$OVERVIEW_LINE" ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            echo "Found Overview section at line $OVERVIEW_LINE which may indicate duplicates"
        fi
        
        # Check if other template sections are nearby (API, Examples, etc.)
        API_NEARBY=$(tail -n +$OVERVIEW_LINE "$FILE" | head -n 20 | grep -c "^## API$")
        EXAMPLES_NEARBY=$(tail -n +$OVERVIEW_LINE "$FILE" | head -n 20 | grep -c "^## Examples$")
        
        if [[ "$API_NEARBY" -gt 0 || "$EXAMPLES_NEARBY" -gt 0 ]]; then
            DUPLICATE_FOUND=true
            DUPLICATE_LINE=$OVERVIEW_LINE
            
            if [[ "$QUIET" == "false" ]]; then
                echo "Found likely duplicate sections starting at line $DUPLICATE_LINE"
            fi
            
            # If dry run, just report
            if [[ "$DRY_RUN" == "true" ]]; then
                if [[ "$QUIET" == "false" ]]; then
                    echo "Would remove content from line $DUPLICATE_LINE to end of file"
                fi
                exit 0
            fi
            
            # Otherwise, fix the file
            TMP_FILE=$(mktemp)
            head -n $((DUPLICATE_LINE - 2)) "$FILE" > "$TMP_FILE"
            
            if [[ -z "$OUTPUT" ]]; then
                mv "$TMP_FILE" "$FILE"
                if [[ "$QUIET" == "false" ]]; then
                    echo "Removed duplicate sections from $FILE"
                fi
            else
                mv "$TMP_FILE" "$OUTPUT"
                if [[ "$QUIET" == "false" ]]; then
                    echo "Wrote file without duplicate sections to: $OUTPUT"
                fi
            fi
            
            exit 0
        fi
    fi
fi

if [[ "$QUIET" == "false" ]]; then
    echo "No duplicate sections found in: $FILE"
fi
exit 0 