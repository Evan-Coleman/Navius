#!/bin/sh
# Shell Utilities for Documentation Scripts
# Provides shell-agnostic utility functions for documentation scripts

# Set strict mode but in a cross-shell compatible way
set -e  # Exit on error
set -u  # Error on undefined variables

# Colors for terminal output (ANSI escape codes work in both bash and zsh)
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Detect shell and OS for shell-specific workarounds
detect_shell() {
    # Get the current shell name using portable method
    current_shell=$(basename "$SHELL")
    echo "$current_shell"
}

detect_os() {
    if [ "$(uname)" = "Darwin" ]; then
        echo "macOS"
    elif [ "$(uname)" = "Linux" ]; then
        echo "Linux"
    else
        echo "Unknown"
    fi
}

# Logging functions
log_info() {
    echo -e "${BLUE}INFO:${NC} $1"
}

log_success() {
    echo -e "${GREEN}SUCCESS:${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

log_error() {
    echo -e "${RED}ERROR:${NC} $1" >&2
}

# Check if command exists in a cross-shell way
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Create directory if it doesn't exist
ensure_dir() {
    if [ ! -d "$1" ]; then
        mkdir -p "$1"
        log_info "Created directory: $1"
    fi
}

# Count files matching a pattern in a directory
# Usage: count_files "directory" "*.md"
count_files() {
    dir="$1"
    pattern="$2"
    
    # Use find which works consistently across bash and zsh
    if [ -d "$dir" ]; then
        count=$(find "$dir" -name "$pattern" -type f | wc -l)
        # Trim whitespace from count (needed for macOS)
        count=$(echo "$count" | tr -d '[:space:]')
        echo "$count"
    else
        echo "0"
    fi
}

# Check if a file exists and is readable
file_exists() {
    [ -f "$1" ] && [ -r "$1" ]
}

# Read file content in a safe way
read_file() {
    if file_exists "$1"; then
        cat "$1"
    else
        log_error "File does not exist or is not readable: $1"
        return 1
    fi
}

# Write content to file safely
write_file() {
    content="$1"
    target_file="$2"
    
    # Create directory if needed
    dir=$(dirname "$target_file")
    ensure_dir "$dir"
    
    # Write content to file
    echo "$content" > "$target_file"
}

# Append content to file safely
append_file() {
    content="$1"
    target_file="$2"
    
    # Create directory if needed
    dir=$(dirname "$target_file")
    ensure_dir "$dir"
    
    # Append content to file
    echo "$content" >> "$target_file"
}

# Check if tool is installed, install if missing (npm packages)
ensure_npm_tool() {
    tool_name="$1"
    package_name="${2:-$tool_name}"
    
    if ! command_exists "$tool_name"; then
        log_warning "$tool_name not found. Attempting to install..."
        if command_exists "npm"; then
            npm install -g "$package_name"
            if ! command_exists "$tool_name"; then
                log_error "Failed to install $tool_name. Please install it manually."
                return 1
            else
                log_success "Successfully installed $tool_name"
            fi
        else
            log_error "npm not found. Please install $package_name manually."
            return 1
        fi
    fi
    return 0
}

# Check if required dependencies are installed
check_dependencies() {
    missing_deps=0
    for dep in "$@"; do
        if ! command_exists "$dep"; then
            log_error "Required dependency '$dep' is not installed."
            missing_deps=$((missing_deps + 1))
        fi
    done
    
    if [ $missing_deps -gt 0 ]; then
        log_error "$missing_deps required dependencies are missing. Please install them before continuing."
        return 1
    fi
    return 0
}

# Cross-shell compatible array handling
# Note: this uses a delimiter-separated string approach rather than arrays
# to ensure compatibility with both bash and sh
create_array() {
    # Return a delimiter-separated string (using a character unlikely to appear in filenames)
    echo "$*" | tr ' ' '␟'
}

array_length() {
    # Count items in a delimiter-separated string
    echo "$1" | tr '␟' '\n' | wc -l | tr -d '[:space:]'
}

array_get() {
    # Get item at index from a delimiter-separated string
    index=$1
    array=$2
    echo "$array" | tr '␟' '\n' | sed -n "$((index+1))p"
}

array_contains() {
    # Check if array contains a value
    needle="$1"
    haystack="$2"
    echo "$haystack" | tr '␟' '\n' | grep -q "^$needle$"
    return $?
}

# Find markdown files recursively in a cross-shell way
# Usage: find_markdown_files "/path/to/dir"
find_markdown_files() {
    find "${1:-.}" -type f -name "*.md" -not -path "*/\.*"
}

# Find markdown files in a directory (non-recursive)
# Usage: find_markdown_files_nonrecursive "/path/to/dir"
find_markdown_files_nonrecursive() {
    find "${1:-.}" -maxdepth 1 -type f -name "*.md" -not -path "*/\.*"
}

# Safely extract YAML frontmatter from markdown
# Returns 1 if frontmatter is not found or is invalid
extract_frontmatter() {
    input_file="$1"
    if [ ! -f "$input_file" ]; then
        log_error "File not found: $input_file"
        return 1
    fi
    
    # Extract content between --- markers
    awk 'BEGIN{f=0} /^---$/{f=1-f;next} f{print}' "$input_file"
}

# Get today's date in YYYY-MM-DD format (cross-shell compatible)
get_today_date() {
    date "+%Y-%m-%d"
}

# Get today's date in Month DD, YYYY format (cross-shell compatible)
get_today_date_formal() {
    date "+%B %d, %Y"
}

# Custom temp file handling (with automatic cleanup)
create_temp_file() {
    temp_file=$(mktemp)
    # Add to global array of temp files to clean up
    TEMP_FILES="$TEMP_FILES␟$temp_file"
    echo "$temp_file"
}

cleanup_temp_files() {
    if [ -n "${TEMP_FILES:-}" ]; then
        echo "$TEMP_FILES" | tr '␟' '\n' | while read -r file; do
            if [ -f "$file" ]; then
                rm "$file"
            fi
        done
    fi
}

# Set up trap to clean up temp files on exit
trap cleanup_temp_files EXIT

# Initialize global temp files tracker
TEMP_FILES=""

# Cross-shell compatible string functions
string_contains() {
    case "$1" in
        *"$2"*) return 0;;
        *) return 1;;
    esac
}

# Trim whitespace from string
string_trim() {
    echo "$1" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//'
}

# Common validation functions for documentation

# Check if a path is a documentation file (considers .md extension and location)
is_doc_file() {
    file="$1"
    if [ -f "$file" ] && echo "$file" | grep -q "\.md$"; then
        # Exclude temporary files, hidden files
        basename=$(basename "$file")
        case "$basename" in
            .*) return 1;; # Hidden file
            *~) return 1;; # Temp file
            \#*\#) return 1;; # Emacs backup
            *) return 0;; # Regular markdown file
        esac
    else
        return 1
    fi
}

# Get document category from path for the new numbered directory structure
get_doc_category_from_path() {
    path="$1"
    
    # Extract category from numbered directory structure
    dir=$(dirname "$path")
    base_dir=$(basename "$dir")
    
    # Strip numbers and underscores from directory name
    category=$(echo "$base_dir" | sed -E 's/^[0-9]+_//' | tr '_' '-')
    
    # Special case for nested subdirectories
    if [ "$category" = "reference" ] || [ "$category" = "guides" ]; then
        subdir=$(basename "$(dirname "$dir")")
        if echo "$subdir" | grep -q "^[0-9]*_"; then
            # This is a subdirectory within a main category
            subcategory=$(basename "$dir")
            category="$category-$subcategory"
        fi
    fi
    
    echo "$category"
}

# Get document type for document structure processing
get_doc_type() {
    path="$1"
    
    if echo "$path" | grep -q "/01_getting_started/"; then
        echo "getting-started"
    elif echo "$path" | grep -q "/02_examples/"; then
        echo "examples"
    elif echo "$path" | grep -q "/03_contributing/"; then
        echo "contributing"
    elif echo "$path" | grep -q "/04_guides/"; then
        echo "guides"
    elif echo "$path" | grep -q "/05_reference/"; then
        echo "reference"
    elif echo "$path" | grep -q "/05_reference/architecture/"; then
        echo "architecture"
    elif echo "$path" | grep -q "/98_roadmaps/"; then
        echo "roadmaps"
    elif echo "$path" | grep -q "/99_misc/"; then
        echo "misc"
    # Support the old structure too
    elif echo "$path" | grep -q "/getting-started/"; then
        echo "getting-started"
    elif echo "$path" | grep -q "/examples/"; then
        echo "examples"
    elif echo "$path" | grep -q "/contributing/"; then
        echo "contributing"
    elif echo "$path" | grep -q "/guides/"; then
        echo "guides"
    elif echo "$path" | grep -q "/reference/"; then
        echo "reference"
    elif echo "$path" | grep -q "/architecture/"; then
        echo "architecture"
    elif echo "$path" | grep -q "/roadmaps/"; then
        echo "roadmaps"
    else
        echo "unknown"
    fi
}

# Get required sections for a document type
get_required_sections() {
    doc_type="$1"
    
    case "$doc_type" in
        "getting-started")
            echo "Overview␟Prerequisites␟Installation␟Usage␟Troubleshooting␟Related Documents"
            ;;
        "guides")
            echo "Overview␟Prerequisites␟Usage␟Configuration␟Examples␟Troubleshooting␟Related Documents"
            ;;
        "reference")
            echo "Overview␟Configuration␟Examples␟Implementation Details␟Related Documents"
            ;;
        "examples")
            echo "Overview␟Prerequisites␟Usage␟Related Documents"
            ;;
        "contributing")
            echo "Overview␟Prerequisites␟Related Documents"
            ;;
        "architecture")
            echo "Overview␟Implementation Details␟Related Documents"
            ;;
        "roadmaps")
            echo "Overview␟Current State␟Target State␟Implementation Phases␟Success Criteria␟Related Documents"
            ;;
        *)
            echo "Overview␟Related Documents"
            ;;
    esac
}

# Display progress bar
# Usage: show_progress $current $total
show_progress() {
    current=$1
    total=$2
    percent=$((current * 100 / total))
    completed=$((percent / 2))
    remaining=$((50 - completed))
    
    # Create the progress bar string
    progress="["
    i=0
    while [ $i -lt $completed ]; do
        progress="${progress}="
        i=$((i + 1))
    done
    
    progress="${progress}>"
    
    i=0
    while [ $i -lt $remaining ]; do
        progress="${progress} "
        i=$((i + 1))
    done
    
    progress="${progress}] ${percent}%"
    
    # Print progress bar
    printf "\r%s" "$progress"
    if [ $current -eq $total ]; then
        echo ""
    fi
}

# Get a specific field from YAML frontmatter
# Usage: get_frontmatter_field "frontmatter_content" "field_name"
get_frontmatter_field() {
    frontmatter="$1"
    field="$2"
    
    # Look for the field in the frontmatter
    echo "$frontmatter" | grep -E "^$field:" | sed -E "s/^$field:[[:space:]]*(.*)/\1/" | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//"
}

# Get a list field from YAML frontmatter
# Usage: get_frontmatter_list "frontmatter_content" "list_field_name"
get_frontmatter_list() {
    frontmatter="$1"
    field="$2"
    
    # Find the list field
    in_list=0
    echo "$frontmatter" | while IFS= read -r line; do
        # Detect start of the list
        if echo "$line" | grep -qE "^$field:"; then
            # If the field has an inline value, extract it
            if echo "$line" | grep -qE "^$field:[[:space:]]*\["; then
                # Extract inline list items
                items=$(echo "$line" | sed -E "s/^$field:[[:space:]]*\[(.*)\]/\1/")
                echo "$items" | tr ',' '\n' | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//"
                return
            else
                in_list=1
            fi
        # If we're in the list, extract items
        elif [ $in_list -eq 1 ]; then
            if echo "$line" | grep -qE "^[[:space:]]*-"; then
                # Extract the list item
                item=$(echo "$line" | sed -E "s/^[[:space:]]*-[[:space:]]*(.*)/\1/" | sed -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")
                echo "$item"
            elif echo "$line" | grep -qE "^[a-zA-Z]"; then
                # We've reached the next field, exit the list
                in_list=0
            fi
        fi
    done
}

# Calculate readability metrics for a document
# Usage: calculate_readability "file_path"
calculate_readability() {
    file="$1"
    
    # Extract content without frontmatter and code blocks
    content=$(sed '1,/^---$/d' "$file" | sed '1,/^---$/d' | sed '/^```/,/^```/d')
    
    # Count words
    word_count=$(echo "$content" | wc -w | tr -d '[:space:]')
    
    # Count sentences (approximately by counting periods, exclamation marks, and question marks)
    sentence_count=$(echo "$content" | tr -c -d '.!?' | wc -c | tr -d '[:space:]')
    
    # Avoid division by zero
    if [ "$sentence_count" -eq 0 ]; then
        sentence_count=1
    fi
    
    # Calculate words per sentence
    words_per_sentence=$((word_count / sentence_count))
    
    # Determine readability category
    if [ "$words_per_sentence" -gt 20 ]; then
        readability="Complex"
    elif [ "$words_per_sentence" -lt 10 ]; then
        readability="Simple"
    else
        readability="Good"
    fi
    
    # Return readability:words_per_sentence:word_count
    echo "$readability:$words_per_sentence:$word_count"
}

# Simple key-value store functions for document analysis
# Usage: store_kv "file_path" "key" "value"
store_kv() {
    file="$1"
    key="$2"
    value="$3"
    
    # Check if key already exists
    if grep -q "^$key:" "$file" 2>/dev/null; then
        # Append to existing value with newline separation for array-like storage
        current=$(grep "^$key:" "$file" | cut -d':' -f2-)
        if [ -z "$current" ]; then
            sed -i.bak "s|^$key:.*|$key:$value|" "$file"
        else
            if ! echo "$current" | grep -q "$value"; then
                sed -i.bak "s|^$key:.*|$key:$current\n$value|" "$file"
            fi
        fi
    else
        # Create new key-value pair
        echo "$key:$value" >> "$file"
    fi
    
    # Remove backup file if created by sed
    if [ -f "$file.bak" ]; then
        rm "$file.bak"
    fi
}

# Get value for a key from key-value store
# Usage: get_kv "file_path" "key"
get_kv() {
    file="$1"
    key="$2"
    
    if [ -f "$file" ]; then
        grep "^$key:" "$file" 2>/dev/null | cut -d':' -f2-
    fi
}

# Export all utility functions
export -f detect_shell
export -f detect_os
export -f log_info
export -f log_success
export -f log_warning
export -f log_error
export -f command_exists
export -f ensure_dir
export -f count_files
export -f file_exists
export -f read_file
export -f write_file
export -f append_file
export -f ensure_npm_tool
export -f check_dependencies
export -f create_array
export -f array_length
export -f array_get
export -f array_contains
export -f find_markdown_files
export -f find_markdown_files_nonrecursive
export -f extract_frontmatter
export -f get_today_date
export -f get_today_date_formal
export -f create_temp_file
export -f cleanup_temp_files
export -f string_contains
export -f string_trim
export -f is_doc_file
export -f get_doc_category_from_path
export -f get_doc_type
export -f get_required_sections
export -f show_progress
export -f get_frontmatter_field
export -f get_frontmatter_list
export -f calculate_readability
export -f store_kv
export -f get_kv 