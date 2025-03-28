#!/bin/bash

# Script to fix broken links in Markdown files
# Usage: 
#   Single file: ./fix_links.sh <markdown_file> [auto]
#   Directory: ./fix_links.sh --dir <directory> [--recursive]
#   Validation: ./fix_links.sh --check-only [--dir <directory>] [--report]
#   Help: ./fix_links.sh --help

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="docs"
TARGET_DIR="docs"
REPORT_FILE="target/reports/docs_validation/link_validation_$(date '+%Y%m%d_%H%M%S').md"
CHECK_ONLY=false
PROCESS_DIR=false
RECURSIVE=false
GENERATE_REPORT=false
AUTO_MODE=""
VERBOSE=false
EXIT_CODE=0

# Terminal colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Display help message
show_help() {
    echo -e "${BLUE}Link Fixing Tool${NC}"
    echo -e "This script identifies and fixes broken links in Markdown files."
    echo
    echo -e "${CYAN}Usage:${NC}"
    echo "  ./fix_links.sh <markdown_file> [auto]       # Process a single file"
    echo "  ./fix_links.sh --dir <directory>            # Process all markdown files in a directory" 
    echo "  ./fix_links.sh --check-only                 # Validate links without making changes"
    echo "  ./fix_links.sh --help                       # Display this help message"
    echo
    echo -e "${CYAN}Options:${NC}"
    echo "  auto                  Apply changes automatically without confirmation"
    echo "  --dir <directory>     Specify the directory to process (default: docs)"
    echo "  --recursive, -r       Process directories recursively"
    echo "  --check-only          Only validate links without making changes"
    echo "  --report              Generate a detailed report of validation results"
    echo "  --verbose, -v         Show more detailed information during processing"
    echo "  --help, -h            Display this help message"
    echo
    echo -e "${CYAN}Examples:${NC}"
    echo "  ./fix_links.sh docs/guides/authentication.md"
    echo "  ./fix_links.sh --dir docs/guides --recursive"
    echo "  ./fix_links.sh --check-only --dir docs --report"
    echo
    echo -e "${CYAN}Integration:${NC}"
    echo "  This tool can be used with other documentation validation tools:"
    echo "  - generate_report.sh: Comprehensive documentation quality report"
    echo "  - fix_frontmatter.sh: Fix or validate document frontmatter"
    echo "  - comprehensive_test.sh: In-depth documentation analysis"
    echo "  - add_sections.sh: Add missing sections to documents"
    echo
}

# Parse command line arguments
parse_args() {
    if [ $# -eq 0 ]; then
        show_help
        exit 0
    fi

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --help|-h)
                show_help
                exit 0
                ;;
            --dir)
                if [ -n "$2" ] && [ "${2:0:1}" != "-" ]; then
                    TARGET_DIR="$2"
                    PROCESS_DIR=true
                    shift 2
                else
                    echo -e "${RED}Error: Argument for $1 is missing${NC}" >&2
                    exit 1
                fi
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
                mkdir -p "$(dirname "$REPORT_FILE")"
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            auto)
                AUTO_MODE="auto"
                shift
                ;;
            -*)
                echo -e "${RED}Error: Unknown option $1${NC}" >&2
                show_help
                exit 1
                ;;
            *)
                if [ -z "$FILE" ]; then
                    FILE="$1"
                    shift
                else
                    echo -e "${RED}Error: Unexpected argument $1${NC}" >&2
                    show_help
                    exit 1
                fi
                ;;
        esac
    done

    # Validate arguments
    if [ "$CHECK_ONLY" = true ] && [ -n "$FILE" ]; then
        echo -e "${RED}Error: Cannot specify a file with --check-only. Use --dir instead.${NC}" >&2
        exit 1
    fi

    if [ "$PROCESS_DIR" = true ] && [ -n "$FILE" ]; then
        echo -e "${RED}Error: Cannot specify both a file and --dir${NC}" >&2
        exit 1
    fi

    if [ -z "$FILE" ] && [ "$PROCESS_DIR" = false ] && [ "$CHECK_ONLY" = false ]; then
        echo -e "${RED}Error: No file or directory specified${NC}" >&2
        show_help
        exit 1
    fi

    # Verify directory exists
    if [ "$PROCESS_DIR" = true ] && [ ! -d "$TARGET_DIR" ]; then
        echo -e "${RED}Error: Directory $TARGET_DIR does not exist${NC}" >&2
        exit 1
    fi

    # Verify file exists and is a markdown file
    if [ -n "$FILE" ]; then
        if [ ! -f "$FILE" ]; then
            echo -e "${RED}Error: $FILE does not exist${NC}" >&2
            exit 1
        elif [[ "$FILE" != *.md ]]; then
            echo -e "${RED}Error: $FILE is not a Markdown file${NC}" >&2
            exit 1
        fi
    fi
}

# Initialize report
init_report() {
    if [ "$GENERATE_REPORT" = true ]; then
        echo "# Link Validation Report" > "$REPORT_FILE"
        echo "Generated on: $(date)" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Summary" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "| Metric | Count |" >> "$REPORT_FILE"
        echo "|--------|-------|" >> "$REPORT_FILE"
        echo "| Total files | 0 |" >> "$REPORT_FILE"
        echo "| Total links | 0 |" >> "$REPORT_FILE"
        echo "| Broken links | 0 |" >> "$REPORT_FILE"
        echo "| Relative links | 0 |" >> "$REPORT_FILE"
        echo "| Fixed links | 0 |" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Detailed Results" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
    fi
}

# Update report counts
update_report_counts() {
    if [ "$GENERATE_REPORT" = true ]; then
        sed -i.bak "s/| Total files | [0-9]* |/| Total files | $TOTAL_FILES |/g" "$REPORT_FILE"
        sed -i.bak "s/| Total links | [0-9]* |/| Total links | $TOTAL_LINKS |/g" "$REPORT_FILE"
        sed -i.bak "s/| Broken links | [0-9]* |/| Broken links | $BROKEN_LINKS |/g" "$REPORT_FILE"
        sed -i.bak "s/| Relative links | [0-9]* |/| Relative links | $RELATIVE_LINKS |/g" "$REPORT_FILE"
        sed -i.bak "s/| Fixed links | [0-9]* |/| Fixed links | $FIXED_LINKS |/g" "$REPORT_FILE"
        rm "${REPORT_FILE}.bak"
    fi
}

# Add entry to report
add_to_report() {
    local file="$1"
    local status="$2"
    local details="$3"

    if [ "$GENERATE_REPORT" = true ]; then
        echo "### $file" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "**Status**: $status" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        if [ -n "$details" ]; then
            echo "**Details**:" >> "$REPORT_FILE"
            echo "$details" >> "$REPORT_FILE"
            echo >> "$REPORT_FILE"
        fi
    fi
}

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

# Function to fix a broken link with a suggestion
fix_broken_link() {
    local file="$1"
    local link="$2"
    local suggestion_index="$3"
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${YELLOW}Fixing broken link: $link${NC}"
    fi
    
    # Extract the text from the link
    link_text=$(grep -o -E "\[[^]]+\]\(\s*$link\s*\)" "$file" | sed -E 's/\[([^]]+)\].*/\1/')
    
    if [[ -z "$link_text" ]]; then
        # Try to extract with more flexible matching
        link_text=$(grep -o -E "\[[^]]+\]\([^)]*$link[^)]*\)" "$file" | sed -E 's/\[([^]]+)\].*/\1/')
    fi
    
    # Get suggestions for the broken link
    suggested_links=()
    
    # If the link is a relative path, try to get an absolute path suggestion
    if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
        base_dir=$(dirname "$file")
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
            replace_link "$file" "$link" "$replacement"
            if [ "$VERBOSE" = true ]; then
                echo -e "${GREEN}✅ Replaced \"$link\" with \"$replacement\"${NC}"
            fi
            return 0
        else
            # Use the first suggestion if the requested index is out of bounds
            replacement=${suggested_links[0]}
            replace_link "$file" "$link" "$replacement"
            if [ "$VERBOSE" = true ]; then
                echo -e "${GREEN}✅ Replaced \"$link\" with \"$replacement\" (first suggestion)${NC}"
            fi
            return 0
        fi
    else
        if [ "$VERBOSE" = true ]; then
            echo -e "${RED}⚠️ No suggestions found for broken link: $link${NC}"
        fi
        return 1
    fi
}

# Function to fix a relative link
fix_relative_link() {
    local file="$1"
    local link="$2"
    
    if [ "$VERBOSE" = true ]; then
        echo -e "${YELLOW}Fixing relative link: $link${NC}"
    fi
    
    # Make sure it's actually a relative link
    if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
        base_dir=$(dirname "$file")
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
            replace_link "$file" "$link" "$replacement"
            if [ "$VERBOSE" = true ]; then
                echo -e "${GREEN}✅ Replaced relative link \"$link\" with absolute path \"$replacement\"${NC}"
            fi
            return 0
        else
            if [ "$VERBOSE" = true ]; then
                echo -e "${RED}⚠️ Cannot convert relative link to absolute path: $link${NC}"
            fi
            return 1
        fi
    else
        if [ "$VERBOSE" = true ]; then
            echo -e "${RED}⚠️ Not a relative link: $link${NC}"
        fi
        return 1
    fi
}

# Process a single file
process_file() {
    local file="$1"
    local status=""
    local details=""
    local file_issues=0
    local file_fixed=0
    
    # Temporary file for the file mapping
    local FILE_MAP=$(mktemp)
    
    # Get all markdown files
    local ALL_FILES=$(get_all_files)
    
    # Create file mapping
    create_file_mapping "$ALL_FILES" "$FILE_MAP"
    
    echo -e "${BLUE}Checking for broken links in $file...${NC}"
    
    # Initialize counters
    local file_broken_links=0
    local file_relative_links=0
    local file_total_links=0
    local file_fixed_links=0
    local broken_link_list=""
    local relative_link_list=""
    
    # Find all links in the file
    while IFS= read -r line; do
        link=$(echo "$line" | tr -d "[]" | awk -F'(' '{print $2}' | tr -d ')')
        
        # Increment total links counter
        ((file_total_links++))
        
        # Skip external links (http, https, ftp)
        if [[ "$link" =~ ^(http|https|ftp):// ]]; then
            continue
        fi
        
        # Skip links with anchors only (like #section)
        if [[ "$link" == "#"* ]]; then
            continue
        fi
        
        # Check for /docs/ absolute paths that exist 
        # and skip them as they're already correctly formatted
        if [[ "$link" == /docs/* ]] && link_exists "$link"; then
            continue
        fi
        
        # Check if it's a relative link
        if [[ "$link" == ../* || "$link" == ./* || ! "$link" == /* ]]; then
            ((file_relative_links++))
            RELATIVE_LINKS=$((RELATIVE_LINKS + 1))
            relative_link_list="${relative_link_list}${link}\n"
            file_issues=$((file_issues + 1))
        fi
        
        # Check if it's a broken link
        if ! link_exists "$link"; then
            ((file_broken_links++))
            BROKEN_LINKS=$((BROKEN_LINKS + 1))
            broken_link_list="${broken_link_list}${link}\n"
            file_issues=$((file_issues + 1))
        fi
    done < <(grep -o -E '\[[^]]+\]\([^)]+\)' "$file")
    
    TOTAL_LINKS=$((TOTAL_LINKS + file_total_links))
    
    if [ $file_broken_links -eq 0 ] && [ $file_relative_links -eq 0 ]; then
        echo -e "${GREEN}✅ All links in $file are correctly formatted and point to existing files.${NC}"
        status="All links valid"
        add_to_report "$file" "$status" ""
        return 0
    fi
    
    # Check-only mode just reports issues
    if [ "$CHECK_ONLY" = true ]; then
        if [ $file_broken_links -gt 0 ]; then
            echo -e "${RED}⚠️ Found $file_broken_links broken links in $file:${NC}"
            echo -e "${broken_link_list}" | sort | uniq
        fi
        if [ $file_relative_links -gt 0 ]; then
            echo -e "${YELLOW}⚠️ Found $file_relative_links relative links that should be absolute in $file:${NC}"
            echo -e "${relative_link_list}" | sort | uniq
        fi
        
        status="Issues found"
        details="- Broken links: $file_broken_links\n- Relative links: $file_relative_links\n\n**Broken links:**\n\`\`\`\n$(echo -e "$broken_link_list" | sort | uniq)\n\`\`\`\n\n**Relative links:**\n\`\`\`\n$(echo -e "$relative_link_list" | sort | uniq)\n\`\`\`"
        
        add_to_report "$file" "$status" "$details"
        return 1
    fi
    
    # Make a backup of the file before making changes
    cp "$file" "${file}.bak"
    
    # Fix broken links first
    if [ $file_broken_links -gt 0 ]; then
        echo -e "${YELLOW}Fixing $file_broken_links broken links in $file${NC}"
        
        while IFS= read -r link; do
            if fix_broken_link "$file" "$link" 1; then
                ((file_fixed_links++))
                FIXED_LINKS=$((FIXED_LINKS + 1))
                file_fixed=$((file_fixed + 1))
            fi
        done < <(echo -e "$broken_link_list" | sort | uniq)
    fi
    
    # Fix relative links
    if [ $file_relative_links -gt 0 ]; then
        echo -e "${YELLOW}Fixing $file_relative_links relative links in $file${NC}"
        
        while IFS= read -r link; do
            if fix_relative_link "$file" "$link"; then
                ((file_fixed_links++))
                FIXED_LINKS=$((FIXED_LINKS + 1))
                file_fixed=$((file_fixed + 1))
            fi
        done < <(echo -e "$relative_link_list" | sort | uniq)
    fi
    
    # Show diff and ask to keep changes
    if [ -f "${file}.bak" ]; then
        if [ "$VERBOSE" = true ]; then
            echo -e "${PURPLE}Changes made:${NC}"
            diff -u "${file}.bak" "$file" || true
            echo ""
        fi
        
        # If in auto mode, auto-confirm
        if [ "$AUTO_MODE" = "auto" ]; then
            rm "${file}.bak"
            echo -e "${GREEN}Changes automatically saved.${NC}"
            status="Fixed links"
            details="- Issues found: $file_issues\n- Links fixed: $file_fixed_links"
        else
            echo -e "${BLUE}Does this look good?${NC}"
            echo "y - Keep all changes"
            echo "n - Discard all changes"
            echo ""
            read -p "Keep all changes? (y/n): " confirm
            
            if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
                rm "${file}.bak"
                echo -e "${GREEN}Changes saved.${NC}"
                status="Fixed links"
                details="- Issues found: $file_issues\n- Links fixed: $file_fixed_links"
            else
                mv "${file}.bak" "$file"
                echo -e "${YELLOW}Changes discarded.${NC}"
                status="Changes discarded"
                details="- Issues found: $file_issues\n- User chose to discard changes"
                file_fixed=0
            fi
        fi
    fi
    
    # Clean up
    rm -f "$FILE_MAP"
    
    if [ $file_fixed -lt $file_issues ]; then
        add_to_report "$file" "$status" "$details"
        return 1
    else
        add_to_report "$file" "$status" "$details"
        return 0
    fi
}

# Process all Markdown files in a directory
process_directory() {
    local dir="$1"
    local count=0
    local find_cmd="find \"$dir\" -type f -name \"*.md\""
    
    if [ "$RECURSIVE" = false ]; then
        find_cmd="find \"$dir\" -maxdepth 1 -type f -name \"*.md\""
    fi
    
    local files=$(eval "$find_cmd")
    local total=$(echo "$files" | wc -l)
    
    if [ -z "$files" ]; then
        echo -e "${YELLOW}No Markdown files found in $dir${NC}"
        return 0
    fi
    
    echo -e "${BLUE}Processing $total Markdown files in $dir...${NC}"
    
    for file in $files; do
        count=$((count + 1))
        
        if [ "$VERBOSE" = true ] || [ "$CHECK_ONLY" = false ]; then
            echo -e "${CYAN}[$count/$total] Processing:${NC} $file"
        fi
        
        if ! process_file "$file"; then
            EXIT_CODE=1
        fi
    done
    
    if [ "$CHECK_ONLY" = true ]; then
        echo -e "${BLUE}Validation completed with exit code ${EXIT_CODE}${NC}"
        if [ $BROKEN_LINKS -gt 0 ] || [ $RELATIVE_LINKS -gt 0 ]; then
            echo -e "${YELLOW}Found $((BROKEN_LINKS + RELATIVE_LINKS)) link issues in $TOTAL_FILES files:${NC}"
            echo -e "${YELLOW}- $BROKEN_LINKS broken links${NC}"
            echo -e "${YELLOW}- $RELATIVE_LINKS relative links that should be absolute${NC}"
        else
            echo -e "${GREEN}All links are valid!${NC}"
        fi
        
        if [ "$GENERATE_REPORT" = true ]; then
            echo -e "${GREEN}Report generated:${NC} $REPORT_FILE"
        fi
    else
        echo -e "${GREEN}Processing completed for $count files.${NC}"
        echo -e "${GREEN}- Successfully fixed: $FIXED_LINKS links${NC}"
        if [ $BROKEN_LINKS -gt 0 ] || [ $RELATIVE_LINKS -gt 0 ]; then
            echo -e "${YELLOW}- Remaining issues: $((BROKEN_LINKS + RELATIVE_LINKS - FIXED_LINKS))${NC}"
        fi
    fi
}

# Main execution
main() {
    parse_args "$@"
    
    # Initialize counters
    TOTAL_FILES=0
    TOTAL_LINKS=0
    BROKEN_LINKS=0
    RELATIVE_LINKS=0
    FIXED_LINKS=0
    
    # Initialize report
    init_report
    
    if [ "$CHECK_ONLY" = true ] || [ "$PROCESS_DIR" = true ]; then
        process_directory "$TARGET_DIR"
        TOTAL_FILES=$(echo "$TARGET_DIR" | grep -c "")
        update_report_counts
        exit $EXIT_CODE
    else
        TOTAL_FILES=1
        process_file "$FILE"
        update_report_counts
        exit $EXIT_CODE
    fi
}

main "$@" 