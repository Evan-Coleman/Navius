#!/bin/bash
#
# batch-fix.sh
#
# Runs fix tools (excluding section additions) on markdown files in a specified directory or a single file
#
# Usage:
#   ./batch-fix.sh [--dir DIRECTORY] [--file FILE] [--verbose] [--dry-run]
#

SCRIPT_DIR="$(dirname "$0")"
VERBOSE=false
DRY_RUN=false

# Process command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --dir)
            TARGET_DIR="$2"
            shift 2
            ;;
        --file)
            TARGET_FILE="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

# Prepare dry-run flag for tool calls
DRY_RUN_FLAG=""
if [[ "$DRY_RUN" == "true" ]]; then
    DRY_RUN_FLAG="--dry-run"
fi

# Prepare verbose flag for tool calls
VERBOSE_FLAG=""
if [[ "$VERBOSE" == "true" ]]; then
    VERBOSE_FLAG="--verbose"
fi

# Helper function to log messages
log_message() {
    local message="$1"
    echo "$message"
}

# Process a single file
process_file() {
    local file="$1"
    local basename=$(basename "$file")
    
    if [[ "$VERBOSE" == "true" ]]; then
        log_message "Processing file: $file"
    fi

    # Check for frontmatter
    if [[ "$VERBOSE" == "true" ]]; then
        log_message "Checking for frontmatter..."
    fi
    "${SCRIPT_DIR}/fix-frontmatter.sh" --file "$file" $DRY_RUN_FLAG >/dev/null 2>&1
    
    # Check for duplicate sections before making other changes
    # This prevents issues where we might add sections and then have duplicates
    if [[ "$VERBOSE" == "true" ]]; then
        log_message "Checking for duplicate sections..."
    fi
    
    # Store the count of section headers before duplicate removal
    section_count_before=$(grep -c "^##" "$file" || echo 0)
    
    "${SCRIPT_DIR}/fix-duplicate-sections.sh" --file "$file" $DRY_RUN_FLAG >/dev/null 2>&1
    
    # Store the count of section headers after duplicate removal
    section_count_after=$(grep -c "^##" "$file" || echo 0)
    
    if [[ $section_count_before -ne $section_count_after ]]; then
        if [[ "$VERBOSE" == "true" ]]; then
            log_message "Removed $(($section_count_before - $section_count_after)) duplicate section(s)."
        fi
    fi
    
    # Check for broken links
    if [[ "$VERBOSE" == "true" ]]; then
        log_message "Checking for broken links..."
    fi
    "${SCRIPT_DIR}/fix-links.sh" --file "$file" $DRY_RUN_FLAG $VERBOSE_FLAG >/dev/null 2>&1
    
    if [[ "$VERBOSE" == "true" ]]; then
        log_message "Batch fix complete for $basename."
    fi
}

# Main execution
if [[ -n "$TARGET_FILE" ]]; then
    # Process a single file
    if [[ ! -f "$TARGET_FILE" ]]; then
        echo "Error: File '$TARGET_FILE' not found."
        exit 1
    fi
    
    # Only process markdown files
    if [[ "$TARGET_FILE" == *.md ]]; then
        process_file "$TARGET_FILE"
    else
        echo "Error: Only markdown (.md) files are supported."
        exit 1
    fi
elif [[ -n "$TARGET_DIR" ]]; then
    # Process all markdown files in the directory
    if [[ ! -d "$TARGET_DIR" ]]; then
        echo "Error: Directory '$TARGET_DIR' not found."
        exit 1
    fi
    
    # Find all markdown files and process them
    find "$TARGET_DIR" -type f -name "*.md" | while read -r file; do
        process_file "$file"
    done
else
    echo "Error: Either --dir or --file must be specified."
    echo "Usage: ./batch-fix.sh [--dir DIRECTORY] [--file FILE] [--verbose] [--dry-run]"
    exit 1
fi

exit 0 