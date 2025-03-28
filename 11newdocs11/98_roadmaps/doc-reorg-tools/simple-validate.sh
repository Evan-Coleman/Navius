#!/bin/bash
#
# simple-validate.sh
#
# Simple document validator for the documentation reorganization project
#
# Usage:
#   ./simple-validate.sh <file_path>
#

if [[ $# -lt 1 ]]; then
    echo "Usage: $0 <file_path>"
    exit 1
fi

FILE="$1"

if [[ ! -f "$FILE" ]]; then
    echo "File not found: $FILE"
    exit 1
fi

echo "===== Simple Document Validator ====="
echo "Validating: $FILE"
echo "Timestamp: $(date)"
echo

# Validate frontmatter
echo "## Frontmatter validation"
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

echo

# Validate document structure
echo "## Document structure validation"
if grep -q "^# " "$FILE"; then
    echo "✅ Main heading (H1) found"
else
    echo "❌ No main heading (H1) found"
fi

if grep -q "^## Overview" "$FILE"; then
    echo "✅ Overview section found"
else
    echo "❌ Overview section missing"
fi

if grep -q "^## Related" "$FILE"; then
    echo "✅ Related Documents section found"
else
    echo "❌ Related Documents section missing"
fi

echo

# Validate code examples
echo "## Code examples validation"
CODE_BLOCKS=$(grep -c "\`\`\`" "$FILE" || true)
# If grep returns nothing or an error, set to 0
if [[ -z "$CODE_BLOCKS" ]]; then
    CODE_BLOCKS=0
fi
CODE_BLOCKS_COUNT=$((CODE_BLOCKS / 2))

RUST_BLOCKS=$(grep -c "\`\`\`rust" "$FILE" || true)
# If grep returns nothing or an error, set to 0
if [[ -z "$RUST_BLOCKS" ]]; then
    RUST_BLOCKS=0
fi

echo "Found $CODE_BLOCKS_COUNT code blocks ($RUST_BLOCKS Rust blocks)"

echo

# Validate links
echo "## Link validation"
LINKS=$(grep -c "\[.*\](.*\.md)" "$FILE" || true)
# If grep returns nothing or an error, set to 0
if [[ -z "$LINKS" ]]; then
    LINKS=0
fi
echo "Found $LINKS internal links"

# Count broken links (simplified check - just looks for non-existent files in common patterns)
BROKEN=0
while IFS= read -r link_line; do
    # Extract the URL part from the markdown link
    link_url=$(echo "$link_line" | sed -E 's/\[(.*)\]\((.*)\)/\2/g')
    
    # Skip if not an internal link to a markdown file
    if [[ ! "$link_url" == *".md"* ]]; then
        continue
    fi
    
    # Determine target file path (simplified)
    if [[ "$link_url" == /* ]]; then
        # Absolute path (from repo root)
        target_file="11newdocs11$link_url"
    elif [[ "$link_url" == ../* ]]; then
        # Relative path using parent directory
        target_file="$(dirname "$FILE")/$link_url"
    else
        # Relative path in same directory
        target_file="$(dirname "$FILE")/$link_url"
    fi
    
    # Check if target file exists
    if [[ ! -f "$target_file" ]]; then
        echo "❌ Broken link: $link_url -> $target_file"
        ((BROKEN++))
    fi
done < <(grep -o "\[.*\](.*\.md)" "$FILE")

if [[ $BROKEN -eq 0 && $LINKS -gt 0 ]]; then
    echo "✅ All internal links are valid"
elif [[ $LINKS -eq 0 ]]; then
    echo "⚠️ No internal links to validate"
else
    echo "❌ Found $BROKEN broken internal links"
fi

echo

# Summary
echo "## Validation Summary"
echo "Document: $FILE"
echo "Timestamp: $(date)"
echo "Code Blocks: $CODE_BLOCKS_COUNT"
echo "Internal Links: $LINKS"
echo "Broken Links: $BROKEN"

exit 0 