#!/bin/bash

# Script to fix broken links in a single Markdown file
# Usage: ./fix_links.sh <markdown_file> [auto]
# If "auto" is passed as the second parameter, links will be fixed with automatic selection of the first suggestion

set -e

DOCS_DIR="docs"

# Check arguments
if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <markdown_file> [auto]"
    echo "Example: $0 docs/guides/authentication.md"
    echo "Add 'auto' as second parameter to apply changes automatically without confirmation"
    exit 1
fi

FILE="$1"
AUTO_MODE=""
if [ $# -eq 2 ] && [ "$2" = "auto" ]; then
    AUTO_MODE="auto"
fi

# Verify the file exists and is a markdown file
if [ ! -f "$FILE" ] || [[ "$FILE" != *.md ]]; then
    echo "Error: $FILE is not a valid markdown file."
    exit 1
fi

# Function to get all markdown files as potential link targets
get_all_files() {
    find "$DOCS_DIR" -type f -name "*.md" | sort
}

# Create a mapping of filenames to their paths for suggesting fixes
create_file_mapping() {
    local all_files=$1
    local file_map=$2
    
    > "$file_map"
    
    while IFS= read -r filepath; do
        filename=$(basename "$filepath")
        echo "$filename|$filepath" >> "$file_map"
    done <<< "$all_files"
}

# Extract all internal links from a file
extract_links() {
    local file=$1
    grep -o -E '\[.*?\]\(.*?\)' "$file" | grep -v "http" | grep -v "#" | sed 's/.*(//' | sed 's/).*//'
}

# Check if a linked file exists
link_exists() {
    local link="$1"
    
    # Handle absolute paths (starting with /docs/)
    if [[ "$link" == /docs/* ]]; then
        local path_after_docs="${link#/docs}"
        if [[ -f "docs${path_after_docs}" ]]; then
            return 0  # Link exists
        else
            return 1  # Link doesn't exist
        fi
    fi
    
    # Handle relative paths
    local dir=$(dirname "$FILE")
    local target_path
    
    # Handle ../ paths
    if [[ "$link" =~ ^\.\./ ]]; then
        target_path=$(realpath --relative-to="$(pwd)" "$dir/$link" 2>/dev/null)
        if [[ -f "$target_path" ]]; then
            return 0  # Link exists
        fi
        return 1  # Link doesn't exist
    fi
    
    # Handle ./ paths
    if [[ "$link" =~ ^\./ ]]; then
        target_path="$dir/${link:2}"
        if [[ -f "$target_path" ]]; then
            return 0  # Link exists
        fi
        return 1  # Link doesn't exist
    fi
    
    # Handle paths with no ./ or ../
    target_path="$dir/$link"
    if [[ -f "$target_path" ]]; then
        return 0  # Link exists
    fi
    
    return 1  # Link doesn't exist
}

# Convert a file path to an absolute path from the docs directory
get_absolute_path() {
    local file_path=$1
    
    # If already an absolute path (starts with /), return as is
    if [[ "$file_path" == /* ]]; then
        echo "$file_path"
    else
        # Get full path, then extract the part after docs/
        local abs_path=$(realpath "$file_path")
        if [[ "$abs_path" == */docs/* ]]; then
            # Extract the part after docs/ and prefix with /docs/
            echo "/docs$(echo "$abs_path" | sed "s|.*/docs||")"
        else
            # If not in docs directory, just use the path as is with /docs/ prefix
            echo "/docs/$(basename "$file_path")"
        fi
    fi
}

# Suggest potential fixes for a broken link
suggest_fixes() {
    local broken_link=$1
    local file_map=$2
    local current_file=$3
    local filename=$(basename "$broken_link")
    
    echo "Possible files for '$filename':"
    
    # Find matches by filename
    matches=$(grep -i "$filename" "$file_map" | cut -d'|' -f2)
    
    if [ -z "$matches" ]; then
        # Try to find similar filenames
        filename_base=$(echo "$filename" | sed 's/\.md$//')
        matches=$(grep -i "$filename_base" "$file_map" | cut -d'|' -f2)
    fi
    
    if [ -n "$matches" ]; then
        count=1
        while IFS= read -r suggestion; do
            # Get the absolute path from the project root, ensuring it starts with /docs/
            abs_path=$(get_absolute_path "$suggestion")
            echo "[$count] $abs_path"
            count=$((count + 1))
        done <<< "$matches"
    else
        echo "No suggestions found."
    fi
}

# Function to replace a link in a file
replace_link() {
    local file=$1
    local old_link=$2
    local new_link=$3
    
    # Escape special characters for sed
    local old_link_escaped=$(echo "$old_link" | sed 's/[\/&]/\\&/g')
    local new_link_escaped=$(echo "$new_link" | sed 's/[\/&]/\\&/g')
    
    # Replace the link in the file
    sed -i "s/\(\\[.*\](\)${old_link_escaped}\(.*\))/\1${new_link_escaped}\2)/g" "$file"
}

# Temporary file for the file mapping
FILE_MAP=$(mktemp)

# Get all markdown files
ALL_FILES=$(get_all_files)

# Create file mapping
create_file_mapping "$ALL_FILES" "$FILE_MAP"

echo "Checking for broken links..."
echo

broken_links=()
ignored_links=0

while IFS= read -r line; do
    link=$(echo "$line" | tr -d "[]" | awk -F'(' '{print $2}' | tr -d ')')
    
    # Skip external links
    if [[ "$link" =~ ^(http|https|ftp):// ]]; then
        continue
    fi
    
    # Check for /docs/ absolute paths that exist 
    # and skip them as they're already correctly formatted
    if [[ "$link" == /docs/* ]] && link_exists "$link"; then
        ((ignored_links++))
        continue
    fi
    
    # Add other links to the broken_links array
    broken_links+=("$link")
done < <(grep -o -E '\[[^]]+\]\([^)]+\)' "$FILE")

echo "Found ${#broken_links[@]} links to check..."
if [ $ignored_links -gt 0 ]; then
    echo "Skipped $ignored_links links that are already correctly formatted with /docs/ prefix and exist."
fi
echo

# If no broken links found
if [ ${#broken_links[@]} -eq 0 ]; then
    echo "✅ All links in $FILE are correctly formatted with /docs/ prefix and point to existing files."
    echo "No fixes needed!"
    rm "$FILE_MAP"
    exit 0
fi

# Process each broken link
echo "Found ${#broken_links[@]} broken links in $FILE."
echo "Note: Links that are already properly formatted with /docs/ prefix and are working will be skipped."
echo "We will fix broken links using absolute paths (starting with /docs/) instead of relative paths."
echo ""

if [[ ${#broken_links[@]} -gt 0 || ${#relative_links[@]} -gt 0 ]]; then
    # Make a backup
    cp "$FILE" "${FILE}.bak"

    # Process links
    if [ "$AUTO_MODE" = "auto" ]; then
        # In auto mode, automatically select the first suggestion for each link
        for link in "${broken_links[@]}"; do
            # Get the first suggestion which is usually the best one
            fix_broken_link "$link" 1
        done
        
        for link in "${relative_links[@]}"; do
            # For relative links, fix to absolute paths
            fix_relative_link "$link" 1
        done
        
        # Automatically keep changes
        if [ -f "${FILE}.bak" ]; then
            rm "${FILE}.bak"
        fi
        echo "Changes automatically saved."
    else
        # Interactive mode - ask user for each link
        echo ""
        echo "Select a link to fix:"
        echo "1. Fix broken links"
        echo "2. Fix relative links"
        echo "3. Exit (keep current changes)"
        echo ""
        read -p "Enter your choice: " choice
    fi
fi

# Show diff and ask to keep changes
if [ -f "${FILE}.bak" ]; then
    echo ""
    echo "Changes made:"
    diff -u "${FILE}.bak" "$FILE" || true
    
    # If in auto mode, auto-confirm
    if [ "$AUTO_MODE" = "auto" ]; then
        rm "${FILE}.bak"
        echo "Changes automatically saved."
    else
        echo ""
        echo "Does this look good?"
        echo "y - Keep all changes"
        echo "n - Discard all changes"
        echo ""
        read -p "Keep all changes? (y/n): " confirm
        
        if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
            rm "${FILE}.bak"
            echo "Changes saved."
        else
            mv "${FILE}.bak" "$FILE"
            echo "Changes discarded."
        fi
    fi
fi

# Clean up any temporary files
if [ -f "$FILE_MAP" ]; then
    rm "$FILE_MAP"
fi

echo "Link fixing completed for $FILE."

# Function to fix a broken link with a suggestion
fix_broken_link() {
    local link="$1"
    local suggestion_index="$2"
    
    echo "Fixing broken link: $link"
    
    # Extract the text from the link
    link_text=$(grep -o -E "\[[^]]+\]\(\s*$link\s*\)" "$FILE" | sed -E 's/\[([^]]+)\].*/\1/')
    
    if [[ -z "$link_text" ]]; then
        # Try to extract with more flexible matching
        link_text=$(grep -o -E "\[[^]]+\]\([^)]*$link[^)]*\)" "$FILE" | sed -E 's/\[([^]]+)\].*/\1/')
    fi
    
    # Get suggestions for the broken link
    suggested_links=()
    
    # If the link is a relative path, try to get an absolute path suggestion
    if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
        base_dir=$(dirname "$FILE")
        target_path=""
        
        if [[ "$link" == ../* ]]; then
            target_path=$(realpath --relative-to="$(pwd)" "$base_dir/$link" 2>/dev/null)
        elif [[ "$link" == ./* ]]; then
            target_path="$base_dir/${link:2}"
        else
            target_path="$base_dir/$link"
        fi
        
        if [[ -f "$target_path" && "$target_path" == */docs/* ]]; then
            # Extract the part after /docs/
            suggestion="/docs${target_path#*docs}"
            suggested_links+=("$suggestion")
        fi
    fi
    
    # Search for similar filenames
    filename=$(basename "$link")
    if [[ -n "$filename" && "$filename" != "/" ]]; then
        while IFS= read -r match; do
            # Convert the match to an absolute path with /docs prefix
            match_abs="/docs${match#*docs}"
            # Don't add duplicates
            if [[ ! " ${suggested_links[*]} " =~ " ${match_abs} " ]]; then
                suggested_links+=("$match_abs")
            fi
        done < <(find "$(pwd)/docs" -type f -name "$filename" | sort)
    fi
    
    # If we have suggestions, use the specified one
    if [ ${#suggested_links[@]} -gt 0 ]; then
        # Adjust index to 0-based
        index=$((suggestion_index-1))
        # Make sure index is valid
        if [ $index -ge 0 ] && [ $index -lt ${#suggested_links[@]} ]; then
            replacement=${suggested_links[$index]}
            replace_link "$FILE" "$link" "$replacement"
            echo "✅ Replaced \"$link\" with \"$replacement\""
        else
            # Use the first suggestion if the requested index is out of bounds
            replacement=${suggested_links[0]}
            replace_link "$FILE" "$link" "$replacement"
            echo "✅ Replaced \"$link\" with \"$replacement\" (first suggestion)"
        fi
    else
        echo "⚠️ No suggestions found for broken link: $link"
    fi
}

# Function to fix a relative link
fix_relative_link() {
    local link="$1"
    local suggestion_index="$2"
    
    echo "Fixing relative link: $link"
    
    # Make sure it's actually a relative link
    if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
        base_dir=$(dirname "$FILE")
        target_path=""
        
        if [[ "$link" == ../* ]]; then
            target_path=$(realpath --relative-to="$(pwd)" "$base_dir/$link" 2>/dev/null)
        elif [[ "$link" == ./* ]]; then
            target_path="$base_dir/${link:2}"
        else
            target_path="$base_dir/$link"
        fi
        
        if [[ -f "$target_path" && "$target_path" == */docs/* ]]; then
            # Extract the part after /docs/
            replacement="/docs${target_path#*docs}"
            replace_link "$FILE" "$link" "$replacement"
            echo "✅ Replaced relative link \"$link\" with absolute path \"$replacement\""
        else
            echo "⚠️ Cannot convert relative link to absolute path: $link"
        fi
    else
        echo "⚠️ Not a relative link: $link"
    fi
} 