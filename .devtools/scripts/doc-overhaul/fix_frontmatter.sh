#!/bin/bash

# Script to add or fix frontmatter in a single Markdown file
# Usage: ./fix_frontmatter.sh <markdown_file> [auto]
# If "auto" is passed as the second parameter, changes will be applied automatically without confirmation

set -e

TODAY_DATE=$(date "+%B %d, %Y")

# Check arguments
if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <markdown_file> [auto]"
    echo "Example: $0 docs/guides/authentication.md"
    echo "Add 'auto' as second parameter to apply changes automatically without confirmation"
    exit 1
fi

FILE="$1"
AUTO_CONFIRM=""
if [ $# -eq 2 ] && [ "$2" = "auto" ]; then
    AUTO_CONFIRM="auto"
fi

# Verify the file exists and is a markdown file
if [ ! -f "$FILE" ] || [[ "$FILE" != *.md ]]; then
    echo "Error: $FILE is not a valid markdown file."
    exit 1
fi

# Check if a file has frontmatter
has_frontmatter() {
    local file="$1"
    head -n 20 "$file" | grep -q "^---" && head -n 20 "$file" | grep -q "^title:"
    return $?
}

# Extract title from first heading in the file
extract_title() {
    local file="$1"
    # First try to get the title from the first heading
    title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
    
    # If title is empty, use the filename
    if [ -z "$title" ]; then
        filename=$(basename "$file" .md)
        # Convert kebab-case or snake_case to Title Case
        title=$(echo "$filename" | sed 's/[-_]/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
    fi
    
    echo "$title"
}

# Determine document type based on path
get_document_type() {
    local file="$1"
    
    if [[ "$file" == *"/getting-started/"* ]]; then
        echo "getting-started"
    elif [[ "$file" == *"/guides/"* ]]; then
        echo "guide"
    elif [[ "$file" == *"/reference/"* ]]; then
        echo "reference"
    elif [[ "$file" == *"/contributing/"* ]]; then
        echo "contributing"
    elif [[ "$file" == *"/roadmaps/"* ]]; then
        echo "roadmap"
    elif [[ "$file" == *"/architecture/"* ]]; then
        echo "architecture"
    else
        echo "documentation"
    fi
}

# Get related documents
get_related_docs() {
    local file="$1"
    local related_docs=""
    
    # Find markdown links [text](path.md)
    related_files=$(grep -o -E '\[[^\]]+\]\([^)]+\.md\)' "$file" | grep -o -E '\([^)]+\.md\)' | sed 's/^(//' | sed 's/)$//' | grep -v "README.md" | sort | uniq | head -3)
    
    for related in $related_files; do
        if [ -n "$related_docs" ]; then
            related_docs="${related_docs}
  - $related"
        else
            related_docs="  - $related"
        fi
    done
    
    echo "$related_docs"
}

# Get tags from file content
get_tags() {
    local file="$1"
    local tags=""
    
    # List of common tags to check for
    common_tags=("api" "architecture" "authentication" "aws" "caching" "database" "deployment" "development" "documentation" "error-handling" "installation" "integration" "performance" "postgres" "redis" "security" "testing")
    
    for tag in "${common_tags[@]}"; do
        if grep -iq "\b$tag\b" "$file"; then
            if [ -n "$tags" ]; then
                tags="${tags}
  - $tag"
            else
                tags="  - $tag"
            fi
        fi
    done
    
    # If no tags found, add a generic one based on document type
    if [ -z "$tags" ]; then
        doc_type=$(get_document_type "$file")
        tags="  - $doc_type"
    fi
    
    echo "$tags"
}

# Create and apply frontmatter
if ! has_frontmatter "$FILE"; then
    echo "Adding frontmatter to $FILE"
    
    # Make a backup
    cp "$FILE" "${FILE}.bak"
    
    title=$(extract_title "$FILE")
    doc_type=$(get_document_type "$FILE")
    related_docs=$(get_related_docs "$FILE")
    tags=$(get_tags "$FILE")
    
    # Create description from first paragraph after heading
    description=$(sed -n '/^# /,/^$/p' "$FILE" | tail -n +2 | grep -v "^$" | head -n 1 | sed 's/^## //')
    # If description is empty or too long, create a generic one
    if [ -z "$description" ] || [ ${#description} -gt 120 ]; then
        description="Documentation about $title"
    fi
    
    # Create frontmatter
    frontmatter="---
title: \"$title\"
description: \"$description\"
category: $doc_type
tags:
$tags"

    if [ -n "$related_docs" ]; then
        frontmatter="${frontmatter}
related:
$related_docs"
    fi
    
    frontmatter="${frontmatter}
last_updated: $TODAY_DATE
version: 1.0
---
"
    
    # Add frontmatter to the file
    echo -e "${frontmatter}$(cat "$FILE")" > "$FILE"
    
    echo "✅ Added frontmatter to $FILE"
    echo "   Title: $title"
    echo "   Type: $doc_type"
    echo "   Description: $description"
    
    # Show diff of changes
    echo ""
    echo "Changes made:"
    diff -u "${FILE}.bak" "$FILE" || true
    
    # Ask user if they want to keep the changes unless auto confirmation is enabled
    if [ "$AUTO_CONFIRM" = "auto" ]; then
        rm "${FILE}.bak"
        echo "Changes automatically saved."
    else
        read -p "Keep these changes? (y/n): " confirm
        if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
            rm "${FILE}.bak"
            echo "Changes saved."
        else
            mv "${FILE}.bak" "$FILE"
            echo "Changes discarded."
        fi
    fi
else
    echo "File already has frontmatter: $FILE"
fi 