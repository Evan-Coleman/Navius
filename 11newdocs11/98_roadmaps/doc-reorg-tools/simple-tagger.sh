#!/bin/bash

# Simple script to add Rust language tags to all untagged code blocks

if [ $# -lt 1 ]; then
    echo "Usage: $0 <file_or_directory>"
    exit 1
fi

total_files=0
total_tagged=0

process_file() {
    local file="$1"
    echo "Processing $file"
    
    # Replace all instances of ``` (alone on a line) with ```rust
    sed -i '' 's/^```$/```rust/g' "$file"
    
    # Count how many replacements were made
    local tagged=$(grep -c '^```rust' "$file")
    echo "  Tagged $tagged code blocks in $file"
    total_tagged=$((total_tagged + tagged))
}

target="$1"
if [ -d "$target" ]; then
    # Find all markdown files
    echo "Finding files in $target"
    for file in $(find "$target" -name "*.md"); do
        process_file "$file"
        total_files=$((total_files + 1))
    done
elif [ -f "$target" ]; then
    process_file "$target"
    total_files=1
else
    echo "Error: $target is not a valid file or directory"
    exit 1
fi

echo
echo "Tagging complete!"
echo "-----------------"
echo "Files processed: $total_files"
echo "Code blocks tagged: $total_tagged"
