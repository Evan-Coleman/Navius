#!/bin/bash
#
# link-analyzer.sh
#
# Analyze links in Markdown files.
#
# Usage:
#   ./link-analyzer.sh --file <file_path> [--output <output_file>] [--quiet]
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

# Analyze links
if [[ "$QUIET" == "false" ]]; then
    echo "Analyzing links in $FILE..."
fi

# Create output file with link analysis results
{
    echo "# Link analysis results for $FILE"
    echo "# $(date)"
    echo ""
    
    # Extract links from file
    LINKS=$(grep -o "\[.*\](.*)" "$FILE" || echo "")
    
    if [[ -z "$LINKS" ]]; then
        echo "No links found in $FILE"
    else
        # Count links
        LINK_COUNT=$(echo "$LINKS" | wc -l | tr -d ' ')
        echo "Found $LINK_COUNT links in $FILE"
        echo ""
        
        # List links
        echo "## Link Details"
        echo "$LINKS" | while read -r link; do
            echo "- $link"
        done
    fi
    
    if [[ "$QUIET" == "false" ]]; then
        echo "Link analysis results written to $OUTPUT"
    fi
} > "$OUTPUT"

exit 0 