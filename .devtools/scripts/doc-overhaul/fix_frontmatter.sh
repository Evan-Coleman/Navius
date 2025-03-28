#!/bin/sh
# Script to validate and fix frontmatter in markdown files

SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

# Set strict mode with standard shell settings
set -e  # Exit on error
set -u  # Error on undefined variables

# Default configuration
TARGET_DIR="docs"
SINGLE_FILE=""
FIX_MODE=false
DRY_RUN=false
VERBOSE=false
TODAY_DATE=$(get_today_date)

print_usage() {
    echo "Usage: fix_frontmatter.sh [OPTIONS]"
    echo "Options:"
    echo "  --dir DIRECTORY   Process markdown files in specific directory"
    echo "  --file FILE       Process a single file only"
    echo "  --fix             Automatically fix issues (default: report only)"
    echo "  --dry-run         Show what would be fixed without making changes"
    echo "  --verbose         Show detailed information about each file"
    echo "  --help            Display this help message"
}

# Parse command line arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --dir)
            if [ -z "$2" ] || [ "${2:0:1}" = "-" ]; then
                log_error "Error: --dir requires a directory path"
                print_usage
                exit 1
            fi
            TARGET_DIR="$2"
            shift 2
            ;;
        --file)
            if [ -z "$2" ] || [ "${2:0:1}" = "-" ]; then
                log_error "Error: --file requires a file path"
                print_usage
                exit 1
            fi
            SINGLE_FILE="$2"
            shift 2
            ;;
        --fix)
            FIX_MODE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Create reports directory
REPORTS_DIR="target/reports/frontmatter"
ensure_dir "$REPORTS_DIR"

# Function to get document type from path
get_doc_type() {
    path="$1"
    
    if echo "$path" | grep -q "/tutorial/"; then
        echo "tutorial"
    elif echo "$path" | grep -q "/guide/"; then
        echo "guide"
    elif echo "$path" | grep -q "/reference/"; then
        echo "reference"
    elif echo "$path" | grep -q "/concept/"; then
        echo "concept"
    elif echo "$path" | grep -q "/faq/"; then
        echo "faq"
    elif echo "$path" | grep -q "/roadmap/"; then
        echo "roadmap"
    else
        echo "unknown"
    fi
}

# Function to determine required frontmatter fields based on document type
get_required_fields() {
    doc_type="$1"
    
    # Common fields for all document types
    common_fields="title description last_updated"
    
    case "$doc_type" in
        tutorial)
            echo "$common_fields difficulty time_required prerequisites"
            ;;
        guide)
            echo "$common_fields audience use_cases related_content"
            ;;
        reference)
            echo "$common_fields version status api_version"
            ;;
        concept)
            echo "$common_fields importance related_concepts"
            ;;
        faq)
            echo "$common_fields topics frequently_asked_with"
            ;;
        roadmap)
            echo "$common_fields status owner target_date"
            ;;
        *)
            echo "$common_fields"
            ;;
    esac
}

# Function to check for frontmatter
check_frontmatter() {
    file="$1"
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    # Extract the frontmatter using the utility function
    frontmatter=$(extract_frontmatter "$file")
    status=$?
    
    if [ $status -ne 0 ]; then
        log_warning "No frontmatter found in $file"
        return 1
    fi
    
    # Determine document type
    doc_type=$(get_doc_type "$file")
    
    # Get required fields for this document type
    required_fields=$(get_required_fields "$doc_type")
    
    missing_fields=""
    
    # Check each required field
    for field in $required_fields; do
        if ! echo "$frontmatter" | grep -q "^$field:"; then
            if [ -z "$missing_fields" ]; then
                missing_fields="$field"
            else
                missing_fields="$missing_fields, $field"
            fi
        fi
    done
    
    if [ -n "$missing_fields" ]; then
        if $VERBOSE; then
            log_warning "Missing required frontmatter in $file ($doc_type type): $missing_fields"
        else
            log_warning "Missing required frontmatter in $file: $missing_fields"
        fi
        return 1
    else
        if $VERBOSE; then
            log_success "Frontmatter OK in $file"
        fi
        return 0
    fi
}

# Function to fix frontmatter
fix_frontmatter() {
    file="$1"
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    # Extract existing frontmatter
    frontmatter=$(extract_frontmatter "$file")
    has_frontmatter=$?
    
    # Determine document type
    doc_type=$(get_doc_type "$file")
    
    # Get required fields for this document type
    required_fields=$(get_required_fields "$doc_type")
    
    # Create temporary file
    temp_file=$(mktemp)
    
    if [ $has_frontmatter -eq 0 ]; then
        # Extract content without frontmatter
        content_without_frontmatter=$(get_content_without_frontmatter "$file")
        
        # Start with existing frontmatter
        echo "---" > "$temp_file"
        echo "$frontmatter" >> "$temp_file"
        
        # Check and add missing fields
        for field in $required_fields; do
            if ! echo "$frontmatter" | grep -q "^$field:"; then
                case "$field" in
                    title)
                        # Try to extract title from first heading
                        title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
                        if [ -z "$title" ]; then
                            # Use filename as fallback
                            title=$(basename "$file" .md | tr '-' ' ' | tr '_' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
                        fi
                        echo "title: $title" >> "$temp_file"
                        ;;
                    description)
                        # Try to extract description from first paragraph after title
                        description=$(sed -n '/^# /,/^$/p' "$file" | tail -n +2 | grep -v "^$" | head -n 1)
                        if [ -z "$description" ]; then
                            description="Description of $(basename "$file" .md | tr '-' ' ' | tr '_' ' ')"
                        fi
                        echo "description: $description" >> "$temp_file"
                        ;;
                    last_updated)
                        echo "last_updated: $TODAY_DATE" >> "$temp_file"
                        ;;
                    *)
                        # Add placeholder for other fields
                        echo "$field: TODO - Fill in $field" >> "$temp_file"
                        ;;
                esac
            fi
        done
        
        # Close frontmatter
        echo "---" >> "$temp_file"
        
        # Add content
        echo "$content_without_frontmatter" >> "$temp_file"
        
    else
        # No existing frontmatter, create new one
        echo "---" > "$temp_file"
        
        # Try to extract title from first heading
        title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
        if [ -z "$title" ]; then
            # Use filename as fallback
            title=$(basename "$file" .md | tr '-' ' ' | tr '_' ' ' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
        fi
        echo "title: $title" >> "$temp_file"
        
        # Try to extract description from first paragraph after title
        description=$(sed -n '/^# /,/^$/p' "$file" | tail -n +2 | grep -v "^$" | head -n 1)
        if [ -z "$description" ]; then
            description="Description of $(basename "$file" .md | tr '-' ' ' | tr '_' ' ')"
        fi
        echo "description: $description" >> "$temp_file"
        
        # Add last_updated date
        echo "last_updated: $TODAY_DATE" >> "$temp_file"
        
        # Add other required fields with placeholders
        for field in $required_fields; do
            if [ "$field" != "title" ] && [ "$field" != "description" ] && [ "$field" != "last_updated" ]; then
                echo "$field: TODO - Fill in $field" >> "$temp_file"
            fi
        done
        
        # Close frontmatter
        echo "---" >> "$temp_file"
        
        # Add original content
        cat "$file" >> "$temp_file"
    fi
    
    # If in dry-run mode, just show diff
    if $DRY_RUN; then
        log_info "Changes that would be made to $file:"
        diff -u "$file" "$temp_file" | grep -v "^---" | grep -v "^+++"
    else
        # Otherwise apply changes
        mv "$temp_file" "$file"
        log_success "Fixed frontmatter in $file"
    fi
    
    # Clean up temp file if it still exists
    if [ -f "$temp_file" ]; then
        rm "$temp_file"
    fi
    
    return 0
}

# Main process function
process_file() {
    file="$1"
    
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    # Check if file is a markdown file
    if ! echo "$file" | grep -q "\.md$"; then
        if $VERBOSE; then
            log_info "Skipping non-markdown file: $file"
        fi
        return 0
    fi
    
    # Check frontmatter
    check_frontmatter "$file"
    result=$?
    
    # If in fix mode and frontmatter needs fixing
    if $FIX_MODE && [ $result -ne 0 ]; then
        fix_frontmatter "$file"
    fi
    
    return $result
}

# Process single file if specified
if [ -n "$SINGLE_FILE" ]; then
    process_file "$SINGLE_FILE"
    exit $?
fi

# Find and process all markdown files in target directory
find_result=0
file_count=0
success_count=0
failure_count=0

log_info "Scanning markdown files in $TARGET_DIR..."

# Use the find_files utility function instead of find directly
file_list=$(find_files "$TARGET_DIR" "*.md")

# Process each file
for file in $file_list; do
    file_count=$((file_count + 1))
    
    process_file "$file"
    result=$?
    
    if [ $result -eq 0 ]; then
        success_count=$((success_count + 1))
    else
        failure_count=$((failure_count + 1))
        find_result=1
    fi
done

# Generate report
if [ $file_count -eq 0 ]; then
    log_warning "No markdown files found in $TARGET_DIR"
else
    log_info "Frontmatter Validation Summary:"
    log_info "  Total files: $file_count"
    log_success "  Files with valid frontmatter: $success_count"
    
    if [ $failure_count -gt 0 ]; then
        log_warning "  Files with missing or invalid frontmatter: $failure_count"
        
        if $FIX_MODE; then
            log_info "Frontmatter issues were automatically fixed."
        elif $DRY_RUN; then
            log_info "Run with --fix to automatically fix frontmatter issues."
        else
            log_info "Run with --fix to automatically fix these issues."
        fi
    fi
fi

exit $find_result 