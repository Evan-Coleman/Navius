#!/bin/bash

# Script to add or fix frontmatter in Markdown files
# Usage: 
#   Single file: ./fix_frontmatter.sh <markdown_file> [auto]
#   Directory: ./fix_frontmatter.sh --dir <directory> [--recursive]
#   Validation: ./fix_frontmatter.sh --validate-all [--dir <directory>] [--report]
#   Help: ./fix_frontmatter.sh --help

set -e

TODAY_DATE=$(date "+%Y-%m-%d")
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="docs"
REPORT_FILE="target/reports/docs_validation/frontmatter_validation_$(date '+%Y%m%d_%H%M%S').md"
VALIDATE_ONLY=false
PROCESS_DIR=false
RECURSIVE=false
GENERATE_REPORT=false
AUTO_CONFIRM=""
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
    echo -e "${BLUE}Frontmatter Fixing Tool${NC}"
    echo -e "This script adds or fixes frontmatter in Markdown files."
    echo
    echo -e "${CYAN}Usage:${NC}"
    echo "  ./fix_frontmatter.sh <markdown_file> [auto]       # Process a single file"
    echo "  ./fix_frontmatter.sh --dir <directory>            # Process all markdown files in a directory" 
    echo "  ./fix_frontmatter.sh --validate-all               # Validate all markdown files (no changes)"
    echo "  ./fix_frontmatter.sh --help                       # Display this help message"
    echo
    echo -e "${CYAN}Options:${NC}"
    echo "  auto                  Apply changes automatically without confirmation"
    echo "  --dir <directory>     Specify the directory to process (default: docs)"
    echo "  --recursive, -r       Process directories recursively"
    echo "  --validate-all        Only validate frontmatter without making changes"
    echo "  --report              Generate a detailed report of validation results"
    echo "  --verbose, -v         Show more detailed information during processing"
    echo "  --help, -h            Display this help message"
    echo
    echo -e "${CYAN}Examples:${NC}"
    echo "  ./fix_frontmatter.sh docs/guides/authentication.md"
    echo "  ./fix_frontmatter.sh --dir docs/guides --recursive"
    echo "  ./fix_frontmatter.sh --validate-all --dir docs --report"
    echo
    echo -e "${CYAN}Integration:${NC}"
    echo "  This tool can be used with other documentation validation tools:"
    echo "  - generate_report.sh: Comprehensive documentation quality report"
    echo "  - comprehensive_test.sh: In-depth documentation analysis"
    echo "  - fix_links.sh: Fix broken links in documentation" 
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
            --validate-all)
                VALIDATE_ONLY=true
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
                AUTO_CONFIRM="auto"
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
    if [ "$VALIDATE_ONLY" = true ] && [ -n "$FILE" ]; then
        echo -e "${RED}Error: Cannot specify a file with --validate-all. Use --dir instead.${NC}" >&2
        exit 1
    fi

    if [ "$PROCESS_DIR" = true ] && [ -n "$FILE" ]; then
        echo -e "${RED}Error: Cannot specify both a file and --dir${NC}" >&2
        exit 1
    fi

    if [ -z "$FILE" ] && [ "$PROCESS_DIR" = false ] && [ "$VALIDATE_ONLY" = false ]; then
        echo -e "${RED}Error: No file or directory specified${NC}" >&2
        show_help
        exit 1
    fi

    if [ "$VALIDATE_ONLY" = true ]; then
        PROCESS_DIR=true
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
        echo "# Frontmatter Validation Report" > "$REPORT_FILE"
        echo "Generated on: $(date)" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Summary" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "| Metric | Count |" >> "$REPORT_FILE"
        echo "|--------|-------|" >> "$REPORT_FILE"
        echo "| Total files | 0 |" >> "$REPORT_FILE"
        echo "| Files with frontmatter | 0 |" >> "$REPORT_FILE"
        echo "| Files without frontmatter | 0 |" >> "$REPORT_FILE"
        echo "| Files with complete frontmatter | 0 |" >> "$REPORT_FILE"
        echo "| Files with incomplete frontmatter | 0 |" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Detailed Results" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
    fi
}

# Update report counts
update_report_counts() {
    if [ "$GENERATE_REPORT" = true ]; then
        sed -i.bak "s/| Total files | [0-9]* |/| Total files | $TOTAL_FILES |/g" "$REPORT_FILE"
        sed -i.bak "s/| Files with frontmatter | [0-9]* |/| Files with frontmatter | $WITH_FRONTMATTER |/g" "$REPORT_FILE"
        sed -i.bak "s/| Files without frontmatter | [0-9]* |/| Files without frontmatter | $WITHOUT_FRONTMATTER |/g" "$REPORT_FILE"
        sed -i.bak "s/| Files with complete frontmatter | [0-9]* |/| Files with complete frontmatter | $COMPLETE_FRONTMATTER |/g" "$REPORT_FILE"
        sed -i.bak "s/| Files with incomplete frontmatter | [0-9]* |/| Files with incomplete frontmatter | $INCOMPLETE_FRONTMATTER |/g" "$REPORT_FILE"
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

# Check if a file has frontmatter
has_frontmatter() {
    local file="$1"
    head -n 20 "$file" | grep -q "^---" && head -n 20 "$file" | grep -q "^title:"
    return $?
}

# Check if frontmatter is complete
has_complete_frontmatter() {
    local file="$1"
    local result=0
    
    # Check for required frontmatter fields
    head -n 40 "$file" | grep -q "^title:" || result=1
    head -n 40 "$file" | grep -q "^description:" || result=1
    head -n 40 "$file" | grep -q "^category:" || result=1
    head -n 40 "$file" | grep -q "^tags:" || result=1
    head -n 40 "$file" | grep -q "^last_updated:" || result=1
    
    return $result
}

# Extract title from first heading in the file
extract_title() {
    local file="$1"
    # First try to get the title from the first heading
    title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
    
    # If title is empty, use the filename
    if [ -z "$title" ]; then
        filename=$(basename "$file" .md)
        # Convert kebab-case or snake_case to Title Case
        title=$(echo "$filename" | sed 's/[-_]/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
    fi
    
    echo "$title"
}

# Determine document type based on path
get_document_type() {
    local file="$1"
    
    if [[ "$file" == *"/01_getting_started/"* || "$file" == *"/getting-started/"* ]]; then
        echo "getting-started"
    elif [[ "$file" == *"/02_examples/"* || "$file" == *"/examples/"* ]]; then
        echo "examples"
    elif [[ "$file" == *"/03_contributing/"* || "$file" == *"/contributing/"* ]]; then
        echo "contributing"
    elif [[ "$file" == *"/04_guides/"* || "$file" == *"/guides/"* ]]; then
        echo "guides"
    elif [[ "$file" == *"/05_reference/"* || "$file" == *"/reference/"* ]]; then
        echo "reference"
    elif [[ "$file" == *"/roadmaps/"* || "$file" == *"/98_roadmaps/"* ]]; then
        echo "roadmap"
    elif [[ "$file" == *"/architecture/"* ]]; then
        echo "architecture"
    elif [[ "$file" == *"/99_misc/"* || "$file" == *"/misc/"* ]]; then
        echo "misc"
    else
        echo "documentation"
    fi
}

# Get related documents
get_related_docs() {
    local file="$1"
    local related_docs=""
    
    # Find markdown links [text](path.md)
    related_files=$(grep -o -E '\[[^\]]+\]\([^)]+\.md\)' "$file" | grep -o -E '\([^)]+\.md\)' | sed 's/^(//' | sed 's/)$//' | grep -v "README.md" | sort | uniq | head -5)
    
    for related in $related_files; do
        if [ -n "$related_docs" ]; then
            related_docs="${related_docs}
  - $related"
        else
            related_docs="  - $related"
        fi
    done
    
    echo "$related_docs"
}

# Get tags from file content
get_tags() {
    local file="$1"
    local tags=""
    
    # List of common tags to check for
    common_tags=("api" "architecture" "authentication" "aws" "caching" "database" "deployment" "development" "documentation" "error-handling" "installation" "integration" "performance" "postgres" "redis" "security" "testing" "guide" "tutorial" "reference" "configuration" "getting-started" "examples" "contributions" "roadmap" "standards" "organization" "migration" "workflow" "release")
    
    for tag in "${common_tags[@]}"; do
        if grep -iq "\b$tag\b" "$file"; then
            if [ -n "$tags" ]; then
                tags="${tags}
  - $tag"
            else
                tags="  - $tag"
            fi
        fi
    done
    
    # Always include a tag based on document type
    doc_type=$(get_document_type "$file")
    if ! echo "$tags" | grep -q "$doc_type"; then
        if [ -n "$tags" ]; then
            tags="${tags}
  - $doc_type"
        else
            tags="  - $doc_type"
        fi
    fi
    
    echo "$tags"
}

# Calculate reading time based on word count
calculate_reading_time() {
    local file="$1"
    local word_count=$(grep -v "^---" "$file" | grep -v "^$" | wc -w)
    # Average reading speed: 200 words per minute
    local reading_time=$((word_count / 200))
    if [ $reading_time -lt 1 ]; then
        reading_time=1
    fi
    echo "$reading_time"
}

# Process a single file
process_file() {
    local file="$1"
    local status=""
    local details=""
    
    if ! has_frontmatter "$file"; then
        if [ "$VALIDATE_ONLY" = true ]; then
            status="Missing frontmatter"
            details="This file has no frontmatter and needs to be processed."
            if [ "$VERBOSE" = true ]; then
                echo -e "${RED}❌ Missing frontmatter: $file${NC}"
            fi
            WITHOUT_FRONTMATTER=$((WITHOUT_FRONTMATTER + 1))
            add_to_report "$file" "$status" "$details"
            return 1
        else
            # Make a backup
            cp "$file" "${file}.bak"
            
            title=$(extract_title "$file")
            doc_type=$(get_document_type "$file")
            related_docs=$(get_related_docs "$file")
            tags=$(get_tags "$file")
            reading_time=$(calculate_reading_time "$file")
            
            # Create description from first paragraph after heading
            description=$(sed -n '/^# /,/^$/p' "$file" | tail -n +2 | grep -v "^$" | head -n 1 | sed 's/^## //')
            # If description is empty or too long, create a generic one
            if [ -z "$description" ] || [ ${#description} -gt 120 ]; then
                description="Documentation about $title"
            fi
            
            # Create frontmatter
            frontmatter="---
title: \"$title\"
description: \"$description\"
category: $doc_type
tags:
$tags"

            if [ -n "$related_docs" ]; then
                frontmatter="${frontmatter}
related:
$related_docs"
            fi
            
            frontmatter="${frontmatter}
last_updated: $TODAY_DATE
version: 1.0
reading_time: $reading_time min
---
"
            
            # Add frontmatter to the file
            echo -e "${frontmatter}$(cat "$file")" > "$file"
            
            echo -e "${GREEN}✅ Added frontmatter to $file${NC}"
            if [ "$VERBOSE" = true ]; then
                echo -e "   ${CYAN}Title:${NC} $title"
                echo -e "   ${CYAN}Type:${NC} $doc_type"
                echo -e "   ${CYAN}Description:${NC} $description"
            fi
            
            # Show diff of changes
            if [ "$VERBOSE" = true ]; then
                echo ""
                echo -e "${PURPLE}Changes made:${NC}"
                diff -u "${file}.bak" "$file" || true
                echo ""
            fi
            
            # Ask user if they want to keep the changes unless auto confirmation is enabled
            if [ "$AUTO_CONFIRM" = "auto" ]; then
                rm "${file}.bak"
                echo "Changes automatically saved."
            else
                read -p "Keep these changes? (y/n): " confirm
                if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
                    rm "${file}.bak"
                    echo "Changes saved."
                    status="Frontmatter added"
                    details="Added complete frontmatter with title, description, category, tags, and last updated date."
                else
                    mv "${file}.bak" "$file"
                    echo "Changes discarded."
                    status="Processing skipped"
                    details="User chose to skip processing this file."
                fi
            fi
            
            WITHOUT_FRONTMATTER=$((WITHOUT_FRONTMATTER + 1))
            add_to_report "$file" "$status" "$details"
            return 0
        fi
    else
        WITH_FRONTMATTER=$((WITH_FRONTMATTER + 1))
        
        if has_complete_frontmatter "$file"; then
            COMPLETE_FRONTMATTER=$((COMPLETE_FRONTMATTER + 1))
            if [ "$VERBOSE" = true ]; then
                echo -e "${GREEN}✓ Complete frontmatter: $file${NC}"
            fi
            status="Complete frontmatter"
            details="This file has all required frontmatter fields."
            add_to_report "$file" "$status" "$details"
            return 0
        else
            INCOMPLETE_FRONTMATTER=$((INCOMPLETE_FRONTMATTER + 1))
            if [ "$VALIDATE_ONLY" = true ]; then
                echo -e "${YELLOW}⚠️ Incomplete frontmatter: $file${NC}"
                status="Incomplete frontmatter"
                details="This file is missing one or more required frontmatter fields (title, description, category, tags, last_updated)."
                add_to_report "$file" "$status" "$details"
                return 1
            else
                echo -e "${YELLOW}⚠️ Incomplete frontmatter detected in $file${NC}"
                echo -e "${CYAN}Not yet implemented:${NC} Updating existing frontmatter will be available in a future version."
                status="Incomplete frontmatter"
                details="This file is missing one or more required frontmatter fields. Updating existing frontmatter is not yet implemented."
                add_to_report "$file" "$status" "$details"
                return 1
            fi
        fi
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
        if [ "$VERBOSE" = true ] || [ "$VALIDATE_ONLY" = false ]; then
            echo -e "${CYAN}[$count/$total] Processing:${NC} $file"
        fi
        
        if ! process_file "$file"; then
            EXIT_CODE=1
        fi
    done
    
    if [ "$VALIDATE_ONLY" = true ]; then
        echo -e "${BLUE}Validation completed with ${EXIT_CODE}${NC}"
        if [ $WITHOUT_FRONTMATTER -gt 0 ] || [ $INCOMPLETE_FRONTMATTER -gt 0 ]; then
            echo -e "${YELLOW}Found issues in $((WITHOUT_FRONTMATTER + INCOMPLETE_FRONTMATTER)) of $TOTAL_FILES files:${NC}"
            echo -e "${YELLOW}- $WITHOUT_FRONTMATTER files without frontmatter${NC}"
            echo -e "${YELLOW}- $INCOMPLETE_FRONTMATTER files with incomplete frontmatter${NC}"
        else
            echo -e "${GREEN}All files have complete frontmatter!${NC}"
        fi
        
        if [ "$GENERATE_REPORT" = true ]; then
            echo -e "${GREEN}Report generated:${NC} $REPORT_FILE"
        fi
    else
        echo -e "${GREEN}Processing completed for $count files.${NC}"
        echo -e "${GREEN}- Successfully processed: $((count - WITHOUT_FRONTMATTER - INCOMPLETE_FRONTMATTER))${NC}"
        if [ $WITHOUT_FRONTMATTER -gt 0 ] || [ $INCOMPLETE_FRONTMATTER -gt 0 ]; then
            echo -e "${YELLOW}- Files with issues: $((WITHOUT_FRONTMATTER + INCOMPLETE_FRONTMATTER))${NC}"
        fi
    fi
}

# Main execution
main() {
    parse_args "$@"
    
    # Initialize counters
    TOTAL_FILES=0
    WITH_FRONTMATTER=0
    WITHOUT_FRONTMATTER=0
    COMPLETE_FRONTMATTER=0
    INCOMPLETE_FRONTMATTER=0
    
    # Initialize report
    init_report
    
    if [ "$VALIDATE_ONLY" = true ]; then
        echo -e "${BLUE}Validating frontmatter in Markdown files...${NC}"
        process_directory "$TARGET_DIR"
        TOTAL_FILES=$((WITH_FRONTMATTER + WITHOUT_FRONTMATTER))
        update_report_counts
        exit $EXIT_CODE
    elif [ "$PROCESS_DIR" = true ]; then
        process_directory "$TARGET_DIR"
        TOTAL_FILES=$((WITH_FRONTMATTER + WITHOUT_FRONTMATTER))
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