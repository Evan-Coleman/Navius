#!/bin/bash

# fix-file-links.sh - Fix broken links in individual markdown files
# 
# This script analyzes and fixes broken links in a specific markdown file.
# It is a modified version of fix-links.sh that works on individual files.
#
# Usage: ./fix-file-links.sh --file <file_path> [--dry-run] [--verbose]

SCRIPT_DIR="$(dirname "$0")"
BASE_DIR="/Users/goblin/dev/git/navius"
VERBOSE=false
DRY_RUN=false
TARGET_FILE=""

# Process command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --file)
            TARGET_FILE="$2"
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
            echo "Usage: ./fix-file-links.sh --file <file_path> [--dry-run] [--verbose]"
            exit 1
            ;;
    esac
done

# Check for required arguments
if [[ -z "$TARGET_FILE" ]]; then
    echo "Error: --file argument is required"
    echo "Usage: ./fix-file-links.sh --file <file_path> [--dry-run] [--verbose]"
    exit 1
fi

# Check if target file exists
if [[ ! -f "$TARGET_FILE" ]]; then
    echo "Error: File not found: $TARGET_FILE"
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
declare -A PATH_MAPPINGS
PATH_MAPPINGS["11newdocs11/reference"]="11newdocs11/05_reference"
PATH_MAPPINGS["11newdocs11/guides"]="11newdocs11/04_guides"
PATH_MAPPINGS["11newdocs11/examples"]="11newdocs11/02_examples"
PATH_MAPPINGS["11newdocs11/getting-started"]="11newdocs11/01_getting_started"
PATH_MAPPINGS["11newdocs11/contributing"]="11newdocs11/03_contributing"
PATH_MAPPINGS["11newdocs11/misc"]="11newdocs11/99_misc"
PATH_MAPPINGS["11newdocs11/roadmaps"]="11newdocs11/98_roadmaps"
PATH_MAPPINGS["02_examples/api-example"]="02_examples"
PATH_MAPPINGS["03_reference/01_api"]="05_reference/01_api"

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
        fi
        
        # Replace the link in the file
        if [[ "$DRY_RUN" != "true" ]]; then
            # Escape special characters for sed
            local escaped_link=$(echo "$link" | sed 's/[\/&]/\\&/g')
            local escaped_new_link=$(echo "$new_link" | sed 's/[\/&]/\\&/g')
            
            log_message "  Replacing: $link with $new_link at line $line_num"
            if sed -i.bak "${line_num}s/${escaped_link}/${escaped_new_link}/" "$file"; then
                log_message "  Replacement succeeded."
                rm -f "${file}.bak"
                return 1  # Return 1 to indicate a successful fix
            else
                log_error "  Failed to replace link in file."
                rm -f "${file}.bak"
                return 0
            fi
        else
            log_message "  DRY RUN: Would replace: $link with $new_link at line $line_num"
            return 1  # Pretend we fixed it for counting
        fi
    fi
    
    # If we get here, the link is broken and could not be fixed
    log_message "  Could not fix broken link: $link"
    return 0
}

# Function to process a single file
process_file() {
    local file="$1"
    local relative_path=$(realpath --relative-to="$BASE_DIR" "$file")
    local extension="${file##*.}"
    
    # Skip non-markdown files
    if [[ "$extension" != "md" ]]; then
        log_message "Skipping non-markdown file: $file"
        return
    fi
    
    log_message "----------------------------------------"
    log_message "Processing file: $relative_path"
    
    # Create a directory for logs if it doesn't exist
    local log_dir="${SCRIPT_DIR}/logs"
    mkdir -p "$log_dir"
    
    # Log file for this fix operation
    local timestamp=$(date +"%Y%m%d-%H%M%S")
    local log_file="${log_dir}/fix-links-${timestamp}-$(basename "$file").log"
    
    local fixed_count=0
    local line_num=1
    
    # Process each line in the file
    while IFS= read -r line; do
        # Extract markdown links: [text](link)
        if [[ "$line" =~ \[.*\]\((.*)\) ]]; then
            local link="${BASH_REMATCH[1]}"
            local result=0
            
            # Skip links that contain only anchors or are already processed
            if [[ "$link" != "#"* ]] && [[ "$link" != http* ]] && [[ -n "$link" ]]; then
                # Fix the link if needed
                fix_link "$file" "$link" "$line_num"
                result=$?
                
                # Count fixed links
                if [[ "$result" -eq 1 ]]; then
                    ((fixed_count++))
                fi
            fi
        fi
        
        ((line_num++))
    done < "$file"
    
    log_message "Fixed $fixed_count links in $relative_path"
    log_message "Details recorded in $log_file"
    log_message "----------------------------------------"
    
    # Log summary to the log file
    {
        echo "Link Fix Report for: $relative_path"
        echo "Timestamp: $(date)"
        echo "Fixed Links: $fixed_count"
        echo "----------------------------------------"
    } > "$log_file"
    
    echo "$fixed_count"  # Return the number of fixed links
}

# Check if target file is in the base directory
TARGET_FILE_REL=$(realpath --relative-to="$BASE_DIR" "$TARGET_FILE")
log_message "Processing file: $TARGET_FILE_REL"

# Process the file
fixed_count=$(process_file "$TARGET_FILE")

# Print summary
log_message "----------------------------------------"
log_message "Link Fix Summary:"
log_message "Target File: $TARGET_FILE_REL"
log_message "Fixed Links: $fixed_count"
log_message "----------------------------------------"
if [[ "$DRY_RUN" == "true" ]]; then
    log_message "DRY RUN - No actual changes were made"
fi
log_message "Process completed."

exit 0 