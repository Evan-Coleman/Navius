#!/bin/sh
# Script to fix broken links in Markdown files

SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

# Set strict mode with standard shell settings
set -e  # Exit on error
set -u  # Error on undefined variables

# Default configuration
DOCS_DIR="docs"
TARGET_DIR="docs"
REPORTS_DIR="target/reports/docs_validation"
REPORT_FILE="${REPORTS_DIR}/link_validation_$(get_today_date)_$(date '+%H%M%S').md"
SINGLE_FILE=""
CHECK_ONLY=false
RECURSIVE=false
GENERATE_REPORT=false
AUTO_CONFIRM=false
VERBOSE=false

print_usage() {
    echo "Usage: fix_links.sh [OPTIONS]"
    echo "Options:"
    echo "  --dir DIRECTORY     Process markdown files in specific directory"
    echo "  --file FILE         Process a single file only"
    echo "  --recursive, -r     Process directories recursively"
    echo "  --check-only        Only validate links without making changes"
    echo "  --report            Generate a detailed report of validation results"
    echo "  --auto              Apply changes automatically without confirmation"
    echo "  --verbose, -v       Show detailed information about each file"
    echo "  --help              Display this help message"
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
        --recursive|-r)
            RECURSIVE=true
            shift
            ;;
        --check-only)
            CHECK_ONLY=true
            shift
            ;;
        --report)
            GENERATE_REPORT=true
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

# Verify inputs
if [ -n "$SINGLE_FILE" ] && [ "$CHECK_ONLY" = true ]; then
    log_error "Cannot specify a file with --check-only. Use --dir instead."
    exit 1
fi

if [ -n "$SINGLE_FILE" ] && [ -n "$TARGET_DIR" ] && [ "$TARGET_DIR" != "docs" ]; then
    log_error "Cannot specify both a file and --dir"
    exit 1
fi

if [ -z "$SINGLE_FILE" ] && [ "$TARGET_DIR" = "docs" ] && [ "$CHECK_ONLY" = false ]; then
    log_info "No file or directory specified, using docs directory"
fi

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

# Create reports directory if generating a report
if [ "$GENERATE_REPORT" = true ]; then
    ensure_dir "$REPORTS_DIR"
    
    # Initialize report
    log_info "Initializing report: $REPORT_FILE"
    {
        echo "# Link Validation Report"
        echo "Generated on: $(date)"
        echo 
        echo "## Summary"
        echo 
        echo "| Metric | Count |"
        echo "|--------|-------|"
        echo "| Total files | 0 |"
        echo "| Total links | 0 |"
        echo "| Broken links | 0 |"
        echo "| Relative links | 0 |"
        echo "| Fixed links | 0 |"
        echo 
        echo "## Detailed Results"
        echo 
    } > "$REPORT_FILE"
fi

# Create a temporary file for the file mapping
FILE_MAP=$(mktemp)

# Function to create a mapping of filenames to their paths for suggesting fixes
create_file_mapping() {
    log_info "Creating file mapping for link suggestions..."
    > "$FILE_MAP"
    
    file_list=$(find_files "$DOCS_DIR" "*.md")
    
    for filepath in $file_list; do
        filename=$(basename "$filepath")
        echo "$filename|$filepath" >> "$FILE_MAP"
    done
}

# Extract all internal links from a file
extract_links() {
    file="$1"
    # Find markdown links excluding external URLs and anchors
    grep -o -E '\[.*?\]\(.*?\)' "$file" | grep -v "http" | grep -v "#" | sed 's/.*(//' | sed 's/).*//'
}

# Check if a linked file exists
link_exists() {
    link="$1"
    
    # Handle absolute paths (starting with /docs/)
    if echo "$link" | grep -q "^/docs/"; then
        # Remove the leading /docs prefix
        path_after_docs=$(echo "$link" | sed 's|^/docs||')
        if [ -f "docs${path_after_docs}" ]; then
            return 0  # Link exists
        else
            return 1  # Link doesn't exist
        fi
    # Handle paths starting with / but not /docs/ (probably a mistake)
    elif echo "$link" | grep -q "^/"; then
        # Try to find the file assuming docs was omitted
        if [ -f "docs$link" ]; then
            return 0  # Link exists
        else
            return 1  # Link doesn't exist
        fi
    # Handle relative paths
    else
        # Get directory of current file
        current_dir=$(dirname "$2")
        
        # Resolve the relative path
        if [ -f "$current_dir/$link" ]; then
            return 0  # Link exists
        else
            return 1  # Link doesn't exist
        fi
    fi
}

# Suggest fixes for broken links
suggest_fix() {
    link="$1"
    filename=$(basename "$link")
    
    # Find potential matches in the file mapping
    matches=$(grep "|" "$FILE_MAP" | grep "$filename" | cut -d'|' -f2)
    
    if [ -n "$matches" ]; then
        # Return the suggested file path
        echo "$matches" | head -n 1
    else
        # No matches found
        echo ""
    fi
}

# Convert a relative link to an absolute link
make_absolute_path() {
    relative_path="$1"
    file_path="$2"
    
    # Skip if already absolute
    if echo "$relative_path" | grep -q "^/"; then
        # If it starts with / but not /docs/, add the /docs prefix
        if ! echo "$relative_path" | grep -q "^/docs/"; then
            echo "/docs$relative_path"
        else
            echo "$relative_path"
        fi
        return
    fi
    
    # Get directory of current file
    current_dir=$(dirname "$file_path")
    
    # Resolve the path (this is a simplistic approach and might not handle all cases)
    resolved_path="$current_dir/$relative_path"
    
    # Normalize the path by resolving . and .. segments
    normalized_path=$(echo "$resolved_path" | sed 's|/\./|/|g' | sed 's|/[^/]*/\.\./||g')
    
    # Convert to absolute path from the docs directory
    absolute_path=$(echo "$normalized_path" | sed "s|^$DOCS_DIR|/docs|")
    
    echo "$absolute_path"
}

# Process a single file
process_file() {
    file="$1"
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    log_info "Processing file: $file"
    
    # Extract links from file
    links=$(extract_links "$file")
    link_count=0
    broken_count=0
    relative_count=0
    fixed_count=0
    broken_link_details=""
    
    # Process each link
    for link in $links; do
        link_count=$((link_count + 1))
        
        # Check if it's a relative link (doesn't start with /)
        if ! echo "$link" | grep -q "^/"; then
            relative_count=$((relative_count + 1))
            if [ "$VERBOSE" = true ]; then
                log_warning "Relative link found: $link"
            fi
        fi
        
        # Check if link exists
        if ! link_exists "$link" "$file"; then
            broken_count=$((broken_count + 1))
            
            if [ "$CHECK_ONLY" = true ]; then
                log_warning "Broken link in $file: $link"
                
                # Add to report details
                if [ -n "$broken_link_details" ]; then
                    broken_link_details="${broken_link_details}\n- Broken link: $link"
                else
                    broken_link_details="- Broken link: $link"
                fi
            else
                # Suggest fix
                suggestion=$(suggest_fix "$link")
                
                if [ -n "$suggestion" ]; then
                    log_info "Found broken link: $link"
                    log_info "Suggested fix: /docs/$suggestion"
                    
                    if [ "$AUTO_CONFIRM" = true ]; then
                        # Automatically apply fix
                        sed -i.bak "s|]($link)|](/docs/$suggestion)|g" "$file"
                        rm "${file}.bak"
                        log_success "Automatically fixed link: $link -> /docs/$suggestion"
                        fixed_count=$((fixed_count + 1))
                    else
                        # Ask for confirmation
                        printf "Fix broken link? [y/N]: "
                        read -r confirm
                        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                            sed -i.bak "s|]($link)|](/docs/$suggestion)|g" "$file"
                            rm "${file}.bak"
                            log_success "Fixed link: $link -> /docs/$suggestion"
                            fixed_count=$((fixed_count + 1))
                        else
                            log_info "Skipped fixing link: $link"
                        fi
                    fi
                else
                    log_warning "Broken link in $file with no suggestion: $link"
                    
                    # Add to report details
                    if [ -n "$broken_link_details" ]; then
                        broken_link_details="${broken_link_details}\n- Broken link (no suggestion): $link"
                    else
                        broken_link_details="- Broken link (no suggestion): $link"
                    fi
                fi
            fi
        elif [ "$relative_count" -gt 0 ] && [ "$CHECK_ONLY" = false ]; then
            # Fix relative links
            if ! echo "$link" | grep -q "^/"; then
                absolute_path=$(make_absolute_path "$link" "$file")
                
                if [ "$AUTO_CONFIRM" = true ]; then
                    # Automatically apply fix
                    sed -i.bak "s|]($link)|]($absolute_path)|g" "$file"
                    rm "${file}.bak"
                    log_success "Automatically converted relative link: $link -> $absolute_path"
                    fixed_count=$((fixed_count + 1))
                else
                    # Ask for confirmation
                    log_info "Found relative link: $link"
                    log_info "Suggested absolute path: $absolute_path"
                    printf "Convert to absolute path? [y/N]: "
                    read -r confirm
                    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
                        sed -i.bak "s|]($link)|]($absolute_path)|g" "$file"
                        rm "${file}.bak"
                        log_success "Converted relative link: $link -> $absolute_path"
                        fixed_count=$((fixed_count + 1))
                    else
                        log_info "Skipped converting link: $link"
                    fi
                fi
            fi
        fi
    done
    
    # Update file stats for report
    if [ "$GENERATE_REPORT" = true ]; then
        report_status="OK"
        if [ $broken_count -gt 0 ]; then
            report_status="Contains $broken_count broken links"
        fi
        
        add_to_report "$file" "$report_status" "$broken_link_details"
    fi
    
    # Print summary for this file
    if [ "$VERBOSE" = true ] || [ $broken_count -gt 0 ] || [ $fixed_count -gt 0 ]; then
        log_info "File $file summary:"
        log_info "  Total links: $link_count"
        
        if [ $broken_count -gt 0 ]; then
            log_warning "  Broken links: $broken_count"
        else
            log_success "  All links valid"
        fi
        
        if [ $relative_count -gt 0 ]; then
            log_warning "  Relative links: $relative_count"
        fi
        
        if [ $fixed_count -gt 0 ]; then
            log_success "  Fixed links: $fixed_count"
        fi
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
        file_list=$(find_files "$dir" "*.md")
    else
        file_list=$(find "$dir" -maxdepth 1 -type f -name "*.md")
    fi
    
    # Initialize counters
    total_files=0
    total_links=0
    total_broken=0
    total_relative=0
    total_fixed=0
    
    # Process each file
    for file in $file_list; do
        total_files=$((total_files + 1))
        
        # Extract file stats
        links=$(extract_links "$file")
        link_count=$(echo "$links" | wc -l)
        if [ "$link_count" = "" ]; then link_count=0; fi
        
        total_links=$((total_links + link_count))
        
        # Process the file
        process_file "$file"
        result=$?
        
        # Count broken links
        if [ $result -eq 1 ]; then
            broken_links=$(echo "$links" | wc -l)
            if [ "$broken_links" = "" ]; then broken_links=0; fi
            total_broken=$((total_broken + broken_links))
        fi
        
        # Count relative links
        relative_links=$(echo "$links" | grep -v "^/" | wc -l)
        if [ "$relative_links" = "" ]; then relative_links=0; fi
        total_relative=$((total_relative + relative_links))
    done
    
    # Update global counters for report
    TOTAL_FILES=$total_files
    TOTAL_LINKS=$total_links
    BROKEN_LINKS=$total_broken
    RELATIVE_LINKS=$total_relative
    FIXED_LINKS=$total_fixed
    
    # Print directory summary
    log_info "Directory $dir summary:"
    log_info "  Total files: $total_files"
    log_info "  Total links: $total_links"
    
    if [ $total_broken -gt 0 ]; then
        log_warning "  Broken links: $total_broken"
    else
        log_success "  All links valid"
    fi
    
    if [ $total_relative -gt 0 ]; then
        log_warning "  Relative links: $total_relative"
    fi
    
    if [ $total_fixed -gt 0 ]; then
        log_success "  Fixed links: $total_fixed"
    fi
    
    # Update report counts
    if [ "$GENERATE_REPORT" = true ]; then
        update_report_counts
    fi
    
    # Return error code if broken links found
    if [ $total_broken -gt 0 ]; then
        return 1
    else
        return 0
    fi
}

# Create file mapping for link suggestions
create_file_mapping

# Main execution
if [ -n "$SINGLE_FILE" ]; then
    process_file "$SINGLE_FILE"
    exit_code=$?
elif [ "$CHECK_ONLY" = true ]; then
    process_directory "$TARGET_DIR" "$RECURSIVE"
    exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_success "All links are valid!"
    else
        log_error "Found broken links. See report for details."
    fi
else
    process_directory "$TARGET_DIR" "$RECURSIVE"
    exit_code=$?
fi

# Clean up
rm -f "$FILE_MAP"

# Output report location if generated
if [ "$GENERATE_REPORT" = true ]; then
    log_success "Link validation report generated: $REPORT_FILE"
fi

exit $exit_code 