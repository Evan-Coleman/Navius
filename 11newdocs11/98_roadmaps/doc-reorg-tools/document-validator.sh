#!/bin/bash
#
# document-validator.sh
#
# Validate document structure and frontmatter in Markdown files.
#
# Usage:
#   ./document-validator.sh --file <file_path> [--output <output_file>] [--quiet]
#

# Handle arguments
FILE=""
OUTPUT=""
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

# If no output file specified, use stdout
if [[ -z "$OUTPUT" ]]; then
    OUTPUT="/dev/stdout"
fi

# Validate document
if [[ "$QUIET" == "false" ]]; then
    echo "Validating document $FILE..."
fi

# Create output file with validation results
{
    echo "# Document validation results for $FILE"
    echo "# $(date)"
    echo ""
    
    # Check for frontmatter
    if grep -q "^---" "$FILE"; then
        echo "✅ Frontmatter found"
        
        # Check for required frontmatter fields
        for field in "title" "description" "category" "last_updated"; do
            if grep -q "^$field: " "$FILE"; then
                echo "✅ Required field '$field' present"
            else
                echo "❌ Required field '$field' missing"
            fi
        done
    else
        echo "❌ No frontmatter found"
    fi
    
    echo ""
    
    # Check for main headings
    if grep -q "^# " "$FILE"; then
        echo "✅ Main heading (H1) found"
    else
        echo "❌ No main heading (H1) found"
    fi
    
    # Check for section headings
    OVERVIEW=$(grep -q "^## Overview" "$FILE" && echo "✅ Overview section found" || echo "❌ Overview section missing")
    echo "$OVERVIEW"
    
    RELATED=$(grep -q "^## Related Documents" "$FILE" && echo "✅ Related Documents section found" || echo "❌ Related Documents section missing")
    echo "$RELATED"
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Document validation results written to $OUTPUT"
    fi
} > "$OUTPUT"

exit 0 