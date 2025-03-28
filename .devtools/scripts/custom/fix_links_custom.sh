#!/bin/bash
# Custom script to fix links in the 11newdocs11 directory - modified from original fix_links.sh

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
DOCS_DIR="11newdocs11"
TARGET_DIR="11newdocs11"
AUTO_CONFIRM=false
VERBOSE=false
RECURSIVE=false

# Print usage information
print_usage() {
    echo "Usage: fix_links_custom.sh [OPTIONS]"
    echo "Options:"
    echo "  --dir DIRECTORY     Process markdown files in specific directory (default: 11newdocs11)"
    echo "  --file FILE         Process a single file only"
    echo "  --recursive, -r     Process directories recursively"
    echo "  --auto              Apply changes automatically without confirmation"
    echo "  --verbose, -v       Show detailed information about each file"
    echo "  --help              Display this help message"
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

# Parse command line arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --dir)
            if [ -z "$2" ]; then
                log_error "--dir requires a directory path"
                print_usage
                exit 1
            fi
            TARGET_DIR="$2"
            shift 2
            ;;
        --file)
            if [ -z "$2" ]; then
                log_error "--file requires a file path"
                print_usage
                exit 1
            fi
            SINGLE_FILE="$2"
            shift 2
            ;;
        --recursive|-r)
            RECURSIVE=true
            shift
            ;;
        --auto)
            AUTO_CONFIRM=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
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

# Verify directory exists
if [ -n "$TARGET_DIR" ] && [ ! -d "$TARGET_DIR" ]; then
    log_error "Directory $TARGET_DIR does not exist"
    exit 1
fi

# Verify file exists and is a markdown file
if [ -n "$SINGLE_FILE" ]; then
    if [ ! -f "$SINGLE_FILE" ]; then
        log_error "File $SINGLE_FILE does not exist"
        exit 1
    elif ! echo "$SINGLE_FILE" | grep -q "\.md$"; then
        log_error "File $SINGLE_FILE is not a Markdown file"
        exit 1
    fi
fi

# Create a temporary file for the file mapping
FILE_MAP=$(mktemp)

# Function to create a mapping of filenames to their paths for suggesting fixes
create_file_mapping() {
    log_info "Creating file mapping for link suggestions..."
    > "$FILE_MAP"
    
    # Find all markdown files in the target directory
    find "$DOCS_DIR" -name "*.md" -type f | while read -r filepath; do
        filename=$(basename "$filepath")
        echo "$filename|$filepath" >> "$FILE_MAP"
    done
    
    log_info "Created file mapping with $(wc -l < "$FILE_MAP") entries"
}

# Function to get relative path (compatible with macOS which doesn't have realpath --relative-to)
get_relative_path() {
    source="$1"
    target="$2"
    
    # Get absolute paths
    source_dir=$(cd "$(dirname "$source")" && pwd)
    target_path=$(cd "$(dirname "$target")" && pwd)/$(basename "$target")
    
    # Get common prefix
    common_prefix=""
    IFS="/" read -ra source_parts <<< "$source_dir"
    IFS="/" read -ra target_parts <<< "$(dirname "$target_path")"
    
    # Find common prefix length
    common_length=0
    for ((i=0; i<${#source_parts[@]} && i<${#target_parts[@]}; i++)); do
        if [ "${source_parts[$i]}" = "${target_parts[$i]}" ]; then
            common_length=$((common_length + 1))
        else
            break
        fi
    done
    
    # Build relative path
    rel_path=""
    for ((i=common_length; i<${#source_parts[@]}; i++)); do
        rel_path="../$rel_path"
    done
    
    for ((i=common_length; i<${#target_parts[@]}; i++)); do
        rel_path="${rel_path}${target_parts[$i]}/"
    done
    
    # Add file name
    rel_path="${rel_path}$(basename "$target_path")"
    
    # Remove trailing slash if any
    rel_path=$(echo "$rel_path" | sed 's/\/$//')
    
    echo "$rel_path"
}

# Extract all internal links from a file
extract_links() {
    file="$1"
    # Find markdown links excluding external URLs
    grep -o -E '\[[^]]*\][[:space:]]*\(/docs/[^)]*\)' "$file" || true
}

# Direct replacement for common link patterns
fix_common_link_patterns() {
    file="$1"
    
    # Handle special case for development-workflow.md link
    if grep -q "/docs/guides/development/development-workflow.md" "$file"; then
        log_info "Found common pattern: /docs/guides/development/development-workflow.md"
        
        # Calculate relative path based on file location
        dir_depth=$(echo "$file" | tr -cd '/' | wc -c)
        
        # Default to a common relative path
        target_path="04_guides/development/development-workflow.md" 
        rel_path=""
        
        # Create appropriate relative path based on location
        case "$file" in
            */01_getting_started/*)
                rel_path="../04_guides/development/development-workflow.md"
                ;;
            */04_guides/*)
                if [[ "$file" == */04_guides/development/* ]]; then
                    rel_path="development-workflow.md"
                elif [[ "$file" == */04_guides/deployment/* ]]; then
                    rel_path="../development/development-workflow.md"
                elif [[ "$file" == */04_guides/features/* ]]; then
                    rel_path="../development/development-workflow.md"
                else
                    rel_path="development/development-workflow.md"
                fi
                ;;
            */05_reference/architecture/*)
                rel_path="../../04_guides/development/development-workflow.md"
                ;;
            *)
                # Calculate based on file depth
                for ((i=2; i<dir_depth; i++)); do
                    rel_path="../$rel_path"
                done
                rel_path="${rel_path}04_guides/development/development-workflow.md"
                ;;
        esac
        
        log_info "Replacing with relative path: $rel_path"
        sed -i.bak "s|/docs/guides/development/development-workflow.md|$rel_path|g" "$file"
        rm -f "${file}.bak"
        
        log_success "Fixed common link: /docs/guides/development/development-workflow.md -> $rel_path"
    fi
    
    # Handle special case for api-standards.md link
    if grep -q "/docs/reference/standards/api-standards.md" "$file"; then
        log_info "Found common pattern: /docs/reference/standards/api-standards.md"
        
        # Calculate appropriate relative path based on location
        rel_path=""
        
        case "$file" in
            */05_reference/patterns/*)
                rel_path="../standards/api-standards.md"
                ;;
            */05_reference/api/*)
                rel_path="../standards/api-standards.md"
                ;;
            */05_reference/architecture/*)
                rel_path="../standards/api-standards.md"
                ;;
            */05_reference/standards/*)
                rel_path="api-standards.md"
                ;;
            *)
                rel_path="../05_reference/standards/api-standards.md"
                ;;
        esac
        
        log_info "Replacing with relative path: $rel_path"
        sed -i.bak "s|/docs/reference/standards/api-standards.md|$rel_path|g" "$file"
        rm -f "${file}.bak"
        
        log_success "Fixed common link: /docs/reference/standards/api-standards.md -> $rel_path"
    fi
    
    # Handle special case for /docs/contributing/contributing.md
    if grep -q "/docs/contributing/contributing.md" "$file"; then
        log_info "Found common pattern: /docs/contributing/contributing.md"
        
        # Calculate appropriate relative path based on location
        rel_path=""
        
        case "$file" in
            */03_contributing/*)
                rel_path="CONTRIBUTING.md"
                ;;
            */01_getting_started/*)
                rel_path="../03_contributing/CONTRIBUTING.md"
                ;;
            */04_guides/*)
                rel_path="../03_contributing/CONTRIBUTING.md"
                ;;
            */05_reference/*)
                rel_path="../03_contributing/CONTRIBUTING.md"
                ;;
            *)
                rel_path="03_contributing/CONTRIBUTING.md"
                ;;
        esac
        
        log_info "Replacing with relative path: $rel_path"
        sed -i.bak "s|/docs/contributing/contributing.md|$rel_path|g" "$file"
        rm -f "${file}.bak"
        
        log_success "Fixed common link: /docs/contributing/contributing.md -> $rel_path"
    fi
    
    # Handle special case for /docs/guides/deployment.md
    if grep -q "/docs/guides/deployment.md" "$file"; then
        log_info "Found common pattern: /docs/guides/deployment.md"
        
        # Calculate appropriate relative path based on location
        rel_path=""
        
        case "$file" in
            */01_getting_started/*)
                rel_path="../04_guides/deployment.md"
                ;;
            */04_guides/*)
                rel_path="deployment.md"
                ;;
            */05_reference/*)
                rel_path="../04_guides/deployment.md"
                ;;
            */03_contributing/*)
                rel_path="../04_guides/deployment.md"
                ;;
            *)
                rel_path="04_guides/deployment.md"
                ;;
        esac
        
        log_info "Replacing with relative path: $rel_path"
        sed -i.bak "s|/docs/guides/deployment.md|$rel_path|g" "$file"
        rm -f "${file}.bak"
        
        log_success "Fixed common link: /docs/guides/deployment.md -> $rel_path"
    fi
}

# Check if a linked file exists
link_exists() {
    link="$1"
    source_file="$2"
    
    # Try to find the correct path by replacing /docs/ with 11newdocs11/
    if echo "$link" | grep -q "^/docs/"; then
        # Remove the leading /docs/ prefix and replace with 11newdocs11/
        new_path=$(echo "$link" | sed 's|^/docs/|11newdocs11/|')
        
        if [ -f "$new_path" ]; then
            return 0  # Link exists
        else
            # Handle number prefixed directories
            # Try to find it with any numbered prefix
            dir_name=$(dirname "$new_path")
            file_name=$(basename "$new_path")
            
            # If in the top level dirs (11newdocs11/guides etc)
            if [[ "$dir_name" == "11newdocs11/"* ]]; then
                dir_parts=(${dir_name//\// })
                if [ "${#dir_parts[@]}" -ge 2 ]; then
                    # Check if it's a pattern like 11newdocs11/guides that needs to become 11newdocs11/04_guides
                    second_part="${dir_parts[1]}"
                    # Look for numbered prefix version
                    potential_dirs=$(find "11newdocs11" -maxdepth 1 -type d -name "[0-9]*_$second_part")
                    
                    if [ -n "$potential_dirs" ]; then
                        # Use the first match
                        for dir in $potential_dirs; do
                            new_dir=$(echo "$dir_name" | sed "s|11newdocs11/$second_part|$dir|")
                            if [ -f "$new_dir/$file_name" ]; then
                                return 0  # Link exists with numbered prefix
                            fi
                        done
                    fi
                fi
            fi
            
            return 1  # Link doesn't exist
        fi
    else
        # Not a /docs/ link
        return 1
    fi
}

# Suggest fixes for broken links
suggest_fix() {
    link="$1"
    source_file="$2"
    
    # Handle /docs/ links by converting to 11newdocs11 directory structure
    if echo "$link" | grep -q "^/docs/"; then
        # Remove the leading /docs/ prefix
        path_after_docs=$(echo "$link" | sed 's|^/docs/||')
        
        # Handle common directory mappings
        # docs/getting-started -> 11newdocs11/01_getting_started
        # docs/guides -> 11newdocs11/04_guides
        # docs/reference -> 11newdocs11/05_reference
        # docs/contributing -> 11newdocs11/03_contributing
        # docs/examples -> 11newdocs11/02_examples
        # docs/roadmaps -> 11newdocs11/98_roadmaps
        
        # First part of the path
        first_part=$(echo "$path_after_docs" | cut -d'/' -f1)
        rest_of_path=$(echo "$path_after_docs" | cut -d'/' -f2-)
        
        case "$first_part" in
            "getting-started")
                suggestion="11newdocs11/01_getting_started/$rest_of_path"
                ;;
            "guides")
                suggestion="11newdocs11/04_guides/$rest_of_path"
                ;;
            "reference")
                suggestion="11newdocs11/05_reference/$rest_of_path"
                ;;
            "contributing")
                suggestion="11newdocs11/03_contributing/$rest_of_path"
                ;;
            "examples")
                suggestion="11newdocs11/02_examples/$rest_of_path"
                ;;
            "roadmaps")
                suggestion="11newdocs11/98_roadmaps/$rest_of_path"
                ;;
            "architecture")
                suggestion="11newdocs11/05_reference/architecture/$rest_of_path"
                ;;
            *)
                suggestion="11newdocs11/$path_after_docs"
                ;;
        esac
        
        # Check if the suggested path exists
        if [ -f "$suggestion" ]; then
            # Convert to relative path based on the source file
            rel_path=$(get_relative_path "$source_file" "$suggestion")
            echo "$rel_path"
            return 0
        fi
        
        # Try to find any file with the same basename
        filename=$(basename "$link")
        matches=$(find "11newdocs11" -name "$filename" -type f)
        
        if [ -n "$matches" ]; then
            # Use the first match
            first_match=$(echo "$matches" | head -n 1)
            # Convert to relative path based on the source file
            rel_path=$(get_relative_path "$source_file" "$first_match")
            echo "$rel_path"
            return 0
        fi
    fi
    
    # No suggestion found
    echo ""
    return 1
}

# Process a single file
process_file() {
    file="$1"
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    log_info "Processing file: $file"
    
    # First, fix common link patterns that we know about
    fix_common_link_patterns "$file"
    
    # Extract absolute links from file
    links=$(extract_links "$file")
    
    # Initialize counters
    link_count=0
    broken_count=0
    fixed_count=0
    
    # Check if there are any links
    if [ -z "$links" ]; then
        log_info "No absolute links found in: $file"
        return 0
    fi
    
    # Process each link
    echo "$links" | while IFS= read -r match; do
        if [ -z "$match" ]; then
            continue
        fi
        
        # Extract the actual link from the match (remove the markdown part)
        link=$(echo "$match" | sed 's/.*(//' | sed 's/).*//')
        
        link_count=$((link_count + 1))
        log_info "Found link #$link_count: $link"
        
        # Check if link exists
        if ! link_exists "$link" "$file"; then
            broken_count=$((broken_count + 1))
            
            # Get suggestion
            suggestion=$(suggest_fix "$link" "$file")
            
            if [ -n "$suggestion" ]; then
                log_info "Found broken link: $link"
                log_info "Suggested fix: $suggestion"
                
                if [ "$AUTO_CONFIRM" = true ]; then
                    # Automatically apply fix
                    escaped_link=$(echo "$link" | sed 's/\//\\\//g')
                    escaped_suggestion=$(echo "$suggestion" | sed 's/\//\\\//g')
                    sed -i.bak "s|]($escaped_link)|]($escaped_suggestion)|g" "$file"
                    rm -f "${file}.bak"
                    log_success "Automatically fixed link: $link -> $suggestion"
                    fixed_count=$((fixed_count + 1))
                else
                    # Ask for confirmation
                    printf "Fix broken link? [y/N]: "
                    read -r confirm
                    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                        escaped_link=$(echo "$link" | sed 's/\//\\\//g')
                        escaped_suggestion=$(echo "$suggestion" | sed 's/\//\\\//g')
                        sed -i.bak "s|]($escaped_link)|]($escaped_suggestion)|g" "$file"
                        rm -f "${file}.bak"
                        log_success "Fixed link: $link -> $suggestion"
                        fixed_count=$((fixed_count + 1))
                    else
                        log_info "Skipped fixing link: $link"
                    fi
                fi
            else
                log_warning "Broken link in $file with no suggestion: $link"
            fi
        fi
    done
    
    # Print summary for this file
    log_info "File $file summary:"
    log_info "  Total links checked: $link_count"
    
    if [ $broken_count -gt 0 ]; then
        log_warning "  Broken links: $broken_count"
    else
        log_success "  All links valid"
    fi
    
    if [ $fixed_count -gt 0 ]; then
        log_success "  Fixed links: $fixed_count"
    fi
    
    # Return error code if broken links found
    if [ $broken_count -gt 0 ]; then
        return 1
    else
        return 0
    fi
}

# Process all files in a directory
process_directory() {
    dir="$1"
    recursive="$2"
    
    log_info "Processing directory: $dir (recursive: $recursive)"
    
    # Find markdown files
    if [ "$recursive" = true ]; then
        file_list=$(find "$dir" -type f -name "*.md")
    else
        file_list=$(find "$dir" -maxdepth 1 -type f -name "*.md")
    fi
    
    # Process each file
    for file in $file_list; do
        process_file "$file"
    done
}

# Create file mapping for link suggestions
create_file_mapping

# Main execution
if [ -n "$SINGLE_FILE" ]; then
    process_file "$SINGLE_FILE"
    exit_code=$?
else
    process_directory "$TARGET_DIR" "$RECURSIVE"
    exit_code=$?
fi

# Clean up
rm -f "$FILE_MAP"

exit $exit_code 