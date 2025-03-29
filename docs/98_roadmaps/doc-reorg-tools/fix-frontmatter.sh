#!/bin/bash
#
# fix-frontmatter.sh
#
# Checks for missing frontmatter in markdown files and adds a basic template if missing
#
# Usage:
#   ./fix-frontmatter.sh --file <file_path> [--output <output_file>] [--dry-run] [--quiet]
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

# Check if the file has frontmatter
HAS_FRONTMATTER=false
if grep -q "^---" "$FILE" && grep -q "^title:" "$FILE"; then
    HAS_FRONTMATTER=true
fi

if [[ "$HAS_FRONTMATTER" == "true" ]]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "File already has frontmatter: $FILE"
    fi
    exit 0
fi

# Get title from first heading or filename
TITLE=$(grep -m 1 "^# " "$FILE" | sed 's/^# //' || echo "")
if [[ -z "$TITLE" ]]; then
    # Extract filename without extension and convert hyphens to spaces
    TITLE=$(basename "$FILE" .md | sed 's/-/ /g' | sed 's/_/ /g')
    # Capitalize first letter of each word
    TITLE=$(echo "$TITLE" | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
fi

# Get date
CURRENT_DATE=$(date +"%Y-%m-%d")

# Create frontmatter template
FRONTMATTER="---
title: $TITLE
description: 
category: documentation
tags:
  - docs
last_updated: $CURRENT_DATE
---

"

# If dry run, just report what would be done
if [[ "$DRY_RUN" == "true" ]]; then
    if [[ "$QUIET" == "false" ]]; then
        echo "Would add frontmatter to: $FILE"
        echo "Generated frontmatter:"
        echo "$FRONTMATTER"
    fi
else
    # If output file is not specified, create a temp file
    if [[ -z "$OUTPUT" ]]; then
        OUTPUT=$(mktemp)
    fi
    
    # Write frontmatter and original content to output file
    echo "$FRONTMATTER" > "$OUTPUT"
    cat "$FILE" >> "$OUTPUT"
    
    # Replace original file if no output specified
    if [[ -z "$2" ]]; then
        mv "$OUTPUT" "$FILE"
        if [[ "$QUIET" == "false" ]]; then
            echo "Added frontmatter to: $FILE"
        fi
    else
        if [[ "$QUIET" == "false" ]]; then
            echo "Wrote file with frontmatter to: $OUTPUT"
        fi
    fi
fi

exit 0 