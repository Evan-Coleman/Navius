#!/bin/bash

# fix-links.sh - Fix broken links in markdown documentation files
# 
# This script analyzes and fixes broken links in markdown files.
# It can handle both absolute and relative paths, and can fix common issues
# like missing file extensions, case sensitivity, and path resolution.
#
# Usage: ./fix-links.sh --dir <directory> [--dry-run] [--verbose]

SCRIPT_DIR="$(dirname "$0")"
BASE_DIR="/Users/goblin/dev/git/navius"
VERBOSE=false
DRY_RUN=false
TARGET_DIR=""

# Process command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dir)
            TARGET_DIR="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            echo "Usage: ./fix-links.sh --dir <directory> [--dry-run] [--verbose]"
            exit 1
            ;;
    esac
done

# Check for required arguments
if [[ -z "$TARGET_DIR" ]]; then
    echo "Error: --dir argument is required"
    echo "Usage: ./fix-links.sh --dir <directory> [--dry-run] [--verbose]"
    exit 1
fi

# Check if target directory exists
if [[ ! -d "$TARGET_DIR" ]]; then
    echo "Error: Directory not found: $TARGET_DIR"
    exit 1
fi

# Helper function to log messages
log_message() {
    local message="$1"
    if [[ "$VERBOSE" == "true" ]]; then
        echo "$message"
    fi
}

# Helper function to log errors
log_error() {
    local message="$1"
    echo "ERROR: $message" >&2
}

# Path mapping of common directories (for common redirects)
declare -A PATH_MAPPINGS=(
    ["11newdocs11/reference"]="11newdocs11/05_reference"
    ["11newdocs11/guides"]="11newdocs11/04_guides"
    ["11newdocs11/examples"]="11newdocs11/02_examples"
    ["11newdocs11/getting-started"]="11newdocs11/01_getting_started"
    ["11newdocs11/contributing"]="11newdocs11/03_contributing"
    ["11newdocs11/misc"]="11newdocs11/99_misc"
    ["11newdocs11/roadmaps"]="11newdocs11/98_roadmaps"
)

# Function to check if a file exists in the base directory
check_file_exists() {
    local path="$1"
    if [[ -f "${BASE_DIR}/${path}" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to check if a file exists with a different case
find_case_insensitive() {
    local path="$1"
    local dir_part=$(dirname "$path")
    local file_part=$(basename "$path")
    
    # Avoid searching in non-existent directories
    if [[ ! -d "${BASE_DIR}/${dir_part}" ]]; then
        return 1
    fi
    
    # Try to find a case-insensitive match
    local matches=$(find "${BASE_DIR}/${dir_part}" -type f -iname "${file_part}" 2>/dev/null)
    
    if [[ -n "$matches" ]]; then
        # Return the first match (relative to BASE_DIR)
        echo "$(echo "$matches" | head -1 | sed "s|${BASE_DIR}/||")"
        return 0
    else
        return 1
    fi
}

# Function to check common alternative paths
check_alternative_paths() {
    local path="$1"
    
    # Try with .md extension if not present
    if [[ "$path" != *.md ]]; then
        if check_file_exists "${path}.md"; then
            echo "${path}.md"
            return 0
        fi
    fi
    
    # Try path mappings
    for old_path in "${!PATH_MAPPINGS[@]}"; do
        if [[ "$path" == ${old_path}* ]]; then
            local new_path="${PATH_MAPPINGS[$old_path]}${path#$old_path}"
            if check_file_exists "$new_path"; then
                echo "$new_path"
                return 0
            elif [[ "$new_path" != *.md ]] && check_file_exists "${new_path}.md"; then
                echo "${new_path}.md"
                return 0
            fi
        fi
    done
    
    # Try different case
    local case_match=$(find_case_insensitive "$path")
    if [[ -n "$case_match" ]]; then
        echo "$case_match"
        return 0
    fi
    
    # Try README.md in directory
    if [[ -d "${BASE_DIR}/${path}" ]]; then
        if check_file_exists "${path}/README.md"; then
            echo "${path}/README.md"
            return 0
        fi
    fi
    
    return 1
}

# Function to fix a single link
fix_link() {
    local file="$1"
    local link="$2"
    local line_num="$3"
    
    # Skip anchors, URLs, and empty links
    if [[ "$link" == "#"* ]] || [[ "$link" == http* ]] || [[ -z "$link" ]]; then
        return 0
    fi
    
    log_message "Checking link: $link in $file:$line_num"
    
    # Determine the absolute path of the link
    local absolute_path=""
    if [[ "${link:0:1}" == "/" ]]; then
        # Absolute path within the repo
        absolute_path="${link:1}" # Remove leading /
    else
        # Relative path, resolve based on the file's location
        local file_dir=$(dirname "$file")
        absolute_path=$(realpath -m --relative-to="$BASE_DIR" "${file_dir}/${link}")
    fi
    
    # Check if the link is valid
    if check_file_exists "$absolute_path"; then
        log_message "  Link is valid: $absolute_path"
        return 0
    fi
    
    # Try to find an alternative path
    local fixed_path=$(check_alternative_paths "$absolute_path")
    if [[ -n "$fixed_path" ]]; then
        log_message "  Found alternative: $fixed_path"
        
        # Convert back to the appropriate relative or absolute link
        local new_link=""
        if [[ "${link:0:1}" == "/" ]]; then
            # It was an absolute link, keep it that way
            new_link="/${fixed_path}"
        else
            # It was a relative link, make the fixed path relative to the file location
            local file_dir=$(dirname "$file")
            file_dir_rel=$(realpath -m --relative-to="$BASE_DIR" "$file_dir")
            new_link=$(realpath -m --relative-to="$file_dir_rel" "${BASE_DIR}/${fixed_path}")
            
            # Add ./ prefix if needed
            if [[ "${new_link:0:1}" != "/" && "${new_link:0:1}" != "." && "${new_link:0:2}" != ".." ]]; then
                new_link="./$new_link"
            fi
        fi
        
        log_message "  Fixed link: $link -> $new_link"
        
        # Check for an anchor in the original link
        local anchor=""
        if [[ "$link" == *"#"* ]]; then
            anchor="#${link#*#}"
            new_link="${new_link}${anchor}"
        fi
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_message "  DRY RUN: Would fix $link to $new_link in $file"
        else
            # Replace the link in the file, being careful with regex special characters
            local escaped_link=$(echo "$link" | sed 's/[\/&]/\\&/g')
            local escaped_new_link=$(echo "$new_link" | sed 's/[\/&]/\\&/g')
            
            # Use perl for better regex handling with complex patterns
            perl -i -pe "s/\]\\($escaped_link\)/]($escaped_new_link)/g" "$file"
            
            log_message "  Fixed link in $file: $link -> $new_link"
        fi
        
        return 0
    fi
    
    log_message "  Could not fix link: $link"
    return 1
}

# Function to process a single file
process_file() {
    local file="$1"
    local fixed_count=0
    local broken_count=0
    
    log_message "Processing file: $file"
    
    # Find markdown links in the file
    local line_num=0
    while IFS= read -r line; do
        ((line_num++))
        
        # Extract all markdown links from the line
        local links=$(echo "$line" | grep -o '\[[^]]*\]([^)]*)' | grep -o '([^)]*)' | sed 's/^(//' | sed 's/)$//')
        
        for link in $links; do
            if fix_link "$file" "$link" "$line_num"; then
                ((fixed_count++))
            else
                ((broken_count++))
            fi
        done
    done < "$file"
    
    if [[ $fixed_count -gt 0 ]]; then
        echo "Fixed $fixed_count links in $file"
    fi
    
    if [[ $broken_count -gt 0 ]]; then
        echo "Could not fix $broken_count links in $file"
    fi
    
    if [[ $fixed_count -eq 0 && $broken_count -eq 0 ]]; then
        log_message "No broken links found in $file"
    fi
}

# Main function to process all markdown files in a directory
process_directory() {
    local dir="$1"
    local file_count=0
    local processed_count=0
    
    echo "Fixing links in directory: $dir"
    if [[ "$DRY_RUN" == "true" ]]; then
        echo "DRY RUN MODE - No changes will be made"
    fi
    
    # Find all markdown files in the directory
    local md_files=$(find "$dir" -type f -name "*.md")
    file_count=$(echo "$md_files" | wc -l)
    
    echo "Found $file_count markdown files to process"
    
    for file in $md_files; do
        process_file "$file"
        ((processed_count++))
        
        # Show progress
        if [[ "$VERBOSE" == "true" && $(($processed_count % 10)) -eq 0 ]]; then
            echo "Progress: $processed_count/$file_count files processed"
        fi
    done
    
    echo "Completed processing $processed_count files"
}

# Call the main function with the target directory
process_directory "$TARGET_DIR"

exit 0 