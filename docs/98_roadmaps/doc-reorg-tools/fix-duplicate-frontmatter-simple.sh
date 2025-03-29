#!/bin/bash

# fix-duplicate-frontmatter-simple.sh - Created April 3, 2025
# Simple script to fix duplicate frontmatter in markdown files

if [ -z "$1" ]; then
  echo "Usage: $0 <directory>"
  exit 1
fi

DIR="$1"

find "$DIR" -type f -name "*.md" | while read file; do
  # Count the number of --- lines
  COUNT=$(grep -c "^---$" "$file")
  
  # If more than 2 (which means duplicate frontmatter), fix it
  if [ "$COUNT" -gt 2 ]; then
    echo "Fixing duplicate frontmatter in $file"
    
    # Get the better frontmatter block (the one with more content)
    FIRST_BLOCK=$(sed -n '1,/^---$/p' "$file" | wc -l)
    SECOND_START=$((FIRST_BLOCK + 1))
    SECOND_END=$(sed -n "${SECOND_START},/^---$/p" "$file" | wc -l)
    SECOND_END=$((SECOND_START + SECOND_END - 1))
    
    FIRST_CONTENT=$(sed -n "1,${FIRST_BLOCK}p" "$file" | grep -v "^$" | grep -v "^---$" | wc -l)
    SECOND_CONTENT=$(sed -n "${SECOND_START},${SECOND_END}p" "$file" | grep -v "^$" | grep -v "^---$" | wc -l)
    
    # Start after the second frontmatter block
    CONTENT_START=$((SECOND_END + 1))
    CONTENT=$(sed -n "${CONTENT_START},\$p" "$file")
    
    # Choose the better frontmatter block
    if [ "$SECOND_CONTENT" -gt "$FIRST_CONTENT" ]; then
      # Second block has more content, use it
      FRONTMATTER=$(sed -n "${SECOND_START},${SECOND_END}p" "$file")
    else
      # First block has more content, use it
      FRONTMATTER=$(sed -n "1,${FIRST_BLOCK}p" "$file")
    fi
    
    # Write the fixed file
    echo "$FRONTMATTER" > "$file.new"
    echo "$CONTENT" >> "$file.new"
    mv "$file.new" "$file"
    
    echo "Fixed $file"
  fi
done

# Also fix files with empty frontmatter (just --- and ---)
find "$DIR" -type f -name "*.md" | while read file; do
  # Check for empty frontmatter
  EMPTY_FRONTMATTER=$(awk '
    BEGIN { count=0; empty=1; }
    /^---$/ { 
      count++; 
      if (count == 1) { start=NR; }
      if (count == 2) { end=NR; }
    }
    { if (NR > start && NR < end && $0 != "") empty=0; }
    END { if (count == 2 && empty == 1) print "empty"; else print "not-empty"; }
  ' "$file")
  
  if [ "$EMPTY_FRONTMATTER" == "empty" ]; then
    echo "Fixing empty frontmatter in $file"
    
    # Get content after the frontmatter
    CONTENT=$(sed -n '/^---$/,$ p' "$file" | sed '1,/^---$/ d')
    
    # Get the filename as title
    TITLE=$(basename "$file" .md | tr '-_' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
    
    # Write the fixed file with better frontmatter
    cat > "$file.new" << EOF
---
title: "$TITLE"
description: "Reference documentation for Navius $TITLE"
category: "Reference"
tags: ["documentation", "reference"]
last_updated: "April 3, 2025"
version: "1.0"
---

$CONTENT
EOF
    
    mv "$file.new" "$file"
    
    echo "Fixed empty frontmatter in $file"
  fi
done

echo "Done processing $DIR" 