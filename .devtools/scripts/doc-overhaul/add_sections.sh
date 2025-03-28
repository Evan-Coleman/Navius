#!/bin/bash

# Script to add missing sections to Markdown files
# 
# IMPORTANT: This script enforces the document standards defined in:
# - /docs/roadmaps/30_documentation-reorganization-roadmap.md
# - /docs/roadmaps/30_documentation-reorganization-instructions.md
# - /docs/reference/standards/documentation-standards.md
#
# The section standards implemented here should always be kept in sync with those documents.
#
# Usage: 
#   Single file: ./add_sections.sh <markdown_file> [auto]
#   Directory: ./add_sections.sh --dir <directory> [--recursive]
#   Help: ./add_sections.sh --help

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPORT_FILE="target/reports/docs_validation/sections_report_$(date '+%Y%m%d_%H%M%S').md"
PROCESS_DIR=false
TARGET_DIR="docs"
RECURSIVE=false
GENERATE_REPORT=false
AUTO_MODE=""
VERBOSE=false
CHECK_ONLY=false
ADD_ALL_SECTIONS=false
CUSTOM_SECTIONS=""
EXIT_CODE=0
CURRENT_DATE=$(date '+%B %d, %Y')

# Terminal colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters for reporting
TOTAL_FILES=0
FIXED_FILES=0
MISSING_SECTIONS_COUNT=0

# Display help message
show_help() {
    echo -e "${BLUE}Section Adding Tool${NC}"
    echo -e "This script adds missing standard sections to Markdown files."
    echo
    echo -e "${CYAN}Usage:${NC}"
    echo "  ./add_sections.sh <markdown_file> [auto]       # Process a single file"
    echo "  ./add_sections.sh --dir <directory>            # Process all markdown files in a directory" 
    echo "  ./add_sections.sh --check-only                 # Check for missing sections without making changes"
    echo "  ./add_sections.sh --help                       # Display this help message"
    echo
    echo -e "${CYAN}Options:${NC}"
    echo "  auto                  Apply changes automatically without confirmation"
    echo "  --dir <directory>     Specify the directory to process (default: docs)"
    echo "  --recursive, -r       Process directories recursively"
    echo "  --check-only          Only check for missing sections without making changes"
    echo "  --report              Generate a detailed report of validation results"
    echo "  --verbose, -v         Show more detailed information during processing"
    echo "  --add-all             Add all possible sections appropriate for document type"
    echo "  --sections \"sec1,sec2\"  Specify custom sections to add (comma-separated)"
    echo "  --help, -h            Display this help message"
    echo
    echo -e "${CYAN}Examples:${NC}"
    echo "  ./add_sections.sh docs/guides/authentication.md"
    echo "  ./add_sections.sh --dir docs/guides --recursive"
    echo "  ./add_sections.sh --check-only --dir docs --report"
    echo "  ./add_sections.sh --sections \"Prerequisites,Troubleshooting\" docs/guides/setup.md"
    echo
    echo -e "${CYAN}Integration:${NC}"
    echo "  This tool works with other documentation validation tools:"
    echo "  - generate_report.sh: Comprehensive documentation quality report"
    echo "  - fix_frontmatter.sh: Fix or validate document frontmatter"
    echo "  - fix_links.sh: Fix broken internal links"
    echo "  - comprehensive_test.sh: In-depth documentation analysis"
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
            --add-all)
                ADD_ALL_SECTIONS=true
                shift
                ;;
            --sections)
                if [ -n "$2" ] && [ "${2:0:1}" != "-" ]; then
                    CUSTOM_SECTIONS="$2"
                    shift 2
                else
                    echo -e "${RED}Error: Argument for $1 is missing${NC}" >&2
                    exit 1
                fi
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
        echo "# Section Validation Report" > "$REPORT_FILE"
        echo "Generated on: $CURRENT_DATE" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Summary" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "| Metric | Count |" >> "$REPORT_FILE"
        echo "|--------|-------|" >> "$REPORT_FILE"
        echo "| Total files | 0 |" >> "$REPORT_FILE"
        echo "| Files with missing sections | 0 |" >> "$REPORT_FILE"
        echo "| Total missing sections | 0 |" >> "$REPORT_FILE"
        echo "| Fixed files | 0 |" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
        echo "## Detailed Results" >> "$REPORT_FILE"
        echo >> "$REPORT_FILE"
    fi
}

# Update report counts
update_report_counts() {
    if [ "$GENERATE_REPORT" = true ]; then
        local MISSING_FILES=$(($TOTAL_FILES - $FIXED_FILES))
        sed -i.bak "s/| Total files | [0-9]* |/| Total files | $TOTAL_FILES |/g" "$REPORT_FILE"
        sed -i.bak "s/| Files with missing sections | [0-9]* |/| Files with missing sections | $MISSING_FILES |/g" "$REPORT_FILE"
        sed -i.bak "s/| Total missing sections | [0-9]* |/| Total missing sections | $MISSING_SECTIONS_COUNT |/g" "$REPORT_FILE"
        sed -i.bak "s/| Fixed files | [0-9]* |/| Fixed files | $FIXED_FILES |/g" "$REPORT_FILE"
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

# Determine document type based on path
get_document_type() {
    local file="$1"
    
    # Support for both old and new directory structure
    if [[ "$file" =~ (getting-started|01_getting_started) ]]; then
        echo "getting-started"
    elif [[ "$file" =~ (guides|04_guides) ]]; then
        echo "guide"
    elif [[ "$file" =~ (reference|05_reference) ]]; then
        echo "reference"
    elif [[ "$file" =~ (contributing|03_contributing) ]]; then
        echo "contributing"
    elif [[ "$file" =~ (roadmaps|98_roadmaps) ]]; then
        echo "roadmap" 
    elif [[ "$file" =~ (architecture|05_reference/architecture) ]]; then
        echo "architecture"
    elif [[ "$file" =~ (examples|02_examples) ]]; then
        echo "example"
    elif [[ "$file" =~ 99_misc ]]; then
        echo "misc"
    else
        # Try to determine from frontmatter
        local category=$(grep -n "^category:" "$file" | head -1 | sed 's/^[0-9]*:category: *//' | sed 's/^"//' | sed 's/"$//')
        if [ -n "$category" ]; then
            case "$category" in
                "getting-started") echo "getting-started" ;;
                "guides"|"guide") echo "guide" ;;
                "reference") echo "reference" ;;
                "contributing") echo "contributing" ;;
                "roadmap"|"roadmaps") echo "roadmap" ;;
                "architecture") echo "architecture" ;;
                "example"|"examples") echo "example" ;;
                "misc") echo "misc" ;;
                *) echo "documentation" ;;
            esac
        else
            echo "documentation"
        fi
    fi
}

# Get the title from the file
get_title() {
    local file="$1"
    
    # First try to get the title from frontmatter
    title=$(grep -n "^title:" "$file" | head -1 | sed 's/^[0-9]*:title: *//' | sed 's/^"//' | sed 's/"$//')
    
    # If no title in frontmatter, try to get it from first heading
    if [ -z "$title" ]; then
        title=$(grep -m 1 "^# " "$file" | sed 's/^# //')
    fi
    
    # If still no title, use the filename
    if [ -z "$title" ]; then
        filename=$(basename "$file" .md)
        # Convert kebab-case or snake_case to Title Case
        title=$(echo "$filename" | sed 's/[-_]/ /g' | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) substr($i,2)} 1')
    fi
    
    echo "$title"
}

# Check if a file has a section
has_section() {
    local file="$1"
    local section="$2"
    
    grep -q "^## $section\|^# $section" "$file"
    return $?
}

# Add a section to a file
add_section() {
    local file="$1"
    local section="$2"
    local content="$3"
    
    # If the file does not end with a newline, add one
    if [ -s "$file" ] && [ "$(tail -c 1 "$file" | xxd -p)" != "0a" ]; then
        echo "" >> "$file"
    fi
    
    # Add the section
    echo -e "$content" >> "$file"
    if [ "$VERBOSE" = true ]; then
        echo -e "${GREEN}✅ Added $section section to $file${NC}"
    fi
}

# Check and update last_updated field in frontmatter
update_last_updated() {
    local file="$1"
    
    # Check if frontmatter exists and has last_updated field
    if grep -q "^---" "$file" && grep -q "^last_updated:" "$file"; then
        # Update last_updated field with current date
        sed -i.bak "s/^last_updated:.*$/last_updated: $CURRENT_DATE/" "$file"
        rm "${file}.bak"
        if [ "$VERBOSE" = true ]; then
            echo -e "${GREEN}✅ Updated last_updated to $CURRENT_DATE in $file${NC}"
        fi
    fi
}

# Get appropriate links for related documents based on document type
get_related_doc_examples() {
    local file_type="$1"
    
    case "$file_type" in
        "guide")
            echo "- [Installation Guide](/docs/01_getting_started/installation.md) - How to install the application\n- [Development Workflow](/docs/04_guides/development/development-workflow.md) - Development best practices"
            ;;
        "getting-started")
            echo "- [Project Structure](/docs/01_getting_started/project-structure.md) - Overview of the codebase\n- [First Steps](/docs/01_getting_started/first-steps.md) - Getting started with the application"
            ;;
        "reference")
            echo "- [API Standards](/docs/05_reference/standards/api-standards.md) - API design guidelines\n- [Error Handling](/docs/05_reference/error-handling.md) - Error handling patterns"
            ;;
        "contributing")
            echo "- [Contributing Guide](/docs/03_contributing/contributing.md) - How to contribute to the project\n- [Development Setup](/docs/01_getting_started/development-setup.md) - Setting up your development environment"
            ;;
        "roadmap")
            echo "- [Project Structure Roadmap](/docs/98_roadmaps/completed/11_project_structure_future_improvements.md) - Future improvements\n- [Documentation Reorganization](/docs/98_roadmaps/30_documentation-reorganization-roadmap.md) - Documentation plans"
            ;;
        "architecture")
            echo "- [Project Structure](/docs/05_reference/architecture/project-structure.md) - Overall structure\n- [Module Dependencies](/docs/05_reference/architecture/module-dependencies.md) - Dependencies between modules"
            ;;
        "example")
            echo "- [Basic Usage](/docs/02_examples/basic-usage.md) - Simple usage examples\n- [Advanced Patterns](/docs/02_examples/advanced-patterns.md) - Advanced usage patterns"
            ;;
        "misc")
            echo "- [Documentation Standards](/docs/05_reference/standards/documentation-standards.md) - Documentation formatting and writing style guidelines\n- [Documentation Reorganization Roadmap](/docs/98_roadmaps/30_documentation-reorganization-roadmap.md) - Strategic plan for restructuring documentation"
            ;;
        *)
            echo "- [Documentation Standards](/docs/05_reference/standards/documentation-standards.md) - Documentation formatting and writing style guidelines\n- [Related Document](/docs/path/to/related-document.md) - Brief description"
            ;;
    esac
}

# Get content for standard sections based on document type
get_section_content() {
    local section="$1"
    local doc_type="$2"
    local title="$3"
    
    case "$section" in
        "Overview")
            echo "\n## Overview\n\nBrief introduction to ${title}.\n"
            ;;
        "Prerequisites")
            echo "\n## Prerequisites\n\nBefore you begin, ensure you have the following:\n\n- Navius development environment set up\n- Rust installed (version 1.70 or higher)\n- Basic understanding of Rust and Axum framework\n"
            ;;
        "Installation")
            if [[ "$doc_type" == "getting-started" ]]; then
                echo "\n## Installation\n\n\`\`\`bash\n# Clone the repository\ngit clone https://github.com/your-org/navius.git\ncd navius\n\n# Install dependencies\ncargo build\n\`\`\`\n"
            fi
            ;;
        "Usage")
            if [[ "$doc_type" == "guide" || "$doc_type" == "example" ]]; then
                echo "\n## Usage\n\nBasic usage examples:\n\n\`\`\`rust\n// Example code for using this feature\nuse navius::core::feature;\n\nfn main() {\n    let result = feature::process();\n    println!(\"Result: {:?}\", result);\n}\n\`\`\`\n"
            fi
            ;;
        "Configuration")
            if [[ "$doc_type" == "guide" || "$doc_type" == "reference" ]]; then
                echo "\n## Configuration\n\nConfiguration options and examples:\n\n\`\`\`yaml\n# Configuration example\nfeature:\n  enabled: true\n  options:\n    timeout: 30\n    retry: 3\n\`\`\`\n"
            fi
            ;;
        "Examples")
            if [[ "$doc_type" != "example" ]]; then
                echo "\n## Examples\n\nCode examples demonstrating key functionality:\n\n\`\`\`rust\n// Basic example\nuse navius::core::feature;\n\n// Advanced usage example\nfeature::configure(Config {\n    timeout: 30,\n    retry: true,\n});\n\`\`\`\n"
            fi
            ;;
        "Troubleshooting")
            echo "\n## Troubleshooting\n\nCommon issues and solutions:\n\n### Common Issue 1\n\nDescription of the issue.\n\n**Solution**: Steps to resolve the issue.\n\n### Common Issue 2\n\nDescription of another common issue.\n\n**Solution**: Steps to resolve the issue.\n"
            ;;
        "Implementation Details")
            if [[ "$doc_type" == "reference" || "$doc_type" == "architecture" ]]; then
                echo "\n## Implementation Details\n\nTechnical details about how this feature is implemented.\n\n```mermaid\nflowchart TD\n    A[Client] -->|Request| B(API)\n    B -->|Process| C{Logic}\n    C -->|Success| D[Response]\n    C -->|Error| E[Error Handler]\n```\n"
            fi
            ;;
        "Related Documents")
            related_examples=$(get_related_doc_examples "$doc_type")
            echo "\n## Related Documents\n\n$related_examples\n"
            ;;
    esac
}

# Get list of appropriate sections based on document type
get_sections_for_type() {
    local doc_type="$1"
    
    # All document types should have Overview and Related Documents
    sections=("Overview" "Related Documents")
    
    # If add-all is enabled, add all relevant sections
    if [ "$ADD_ALL_SECTIONS" = true ]; then
        case "$doc_type" in
            "getting-started")
                sections=("Overview" "Prerequisites" "Installation" "Usage" "Troubleshooting" "Related Documents")
                ;;
            "guide")
                sections=("Overview" "Prerequisites" "Usage" "Configuration" "Examples" "Troubleshooting" "Related Documents")
                ;;
            "reference")
                sections=("Overview" "Configuration" "Examples" "Implementation Details" "Related Documents")
                ;;
            "example")
                sections=("Overview" "Prerequisites" "Usage" "Related Documents")
                ;;
            "contributing")
                sections=("Overview" "Prerequisites" "Related Documents")
                ;;
            "architecture")
                sections=("Overview" "Implementation Details" "Related Documents")
                ;;
            "roadmap")
                sections=("Overview" "Current State" "Target State" "Implementation Phases" "Success Criteria" "Related Documents")
                ;;
            "misc")
                sections=("Overview" "Related Documents")
                ;;
            *)
                sections=("Overview" "Related Documents")
                ;;
        esac
    fi
    
    # If custom sections are specified, use those instead
    if [ -n "$CUSTOM_SECTIONS" ]; then
        IFS=',' read -ra sections <<< "$CUSTOM_SECTIONS"
    fi
    
    echo "${sections[@]}"
}

# Process a single file
process_file() {
    local file="$1"
    local status=""
    local details=""
    local missing_sections_list=""
    local file_missing_count=0
    
    # Determine document type and title
    doc_type=$(get_document_type "$file")
    title=$(get_title "$file")
    
    # Get appropriate sections for this document type
    mapfile -t sections < <(get_sections_for_type "$doc_type")
    
    # Check for missing sections
    missing_sections=()
    for section in "${sections[@]}"; do
        if ! has_section "$file" "$section"; then
            missing_sections+=("$section")
            missing_sections_list="${missing_sections_list}- $section\n"
            file_missing_count=$((file_missing_count + 1))
            MISSING_SECTIONS_COUNT=$((MISSING_SECTIONS_COUNT + 1))
        fi
    done
    
    # If no missing sections, exit
    if [ ${#missing_sections[@]} -eq 0 ]; then
        if [ "$VERBOSE" = true ]; then
            echo -e "${GREEN}✅ No missing sections found in $file.${NC}"
        fi
        status="Complete"
        FIXED_FILES=$((FIXED_FILES + 1))
        add_to_report "$file" "$status" "All required sections are present."
        return 0
    fi
    
    # In check-only mode, just report the missing sections
    if [ "$CHECK_ONLY" = true ]; then
        echo -e "${YELLOW}⚠️ Missing sections in $file:${NC}"
        echo -e "${missing_sections_list}"
        
        status="Missing sections"
        details="The following sections are missing:\n${missing_sections_list}"
        add_to_report "$file" "$status" "$details"
        return 1
    fi
    
    echo -e "${BLUE}Adding missing sections to $file...${NC}"
    
    # Make a backup
    cp "$file" "${file}.bak"
    
    # Add missing sections
    for section in "${missing_sections[@]}"; do
        content=$(get_section_content "$section" "$doc_type" "$title")
        add_section "$file" "$section" "$content"
    done
    
    # Update last_updated field in frontmatter if we made changes
    update_last_updated "$file"
    
    # Show diff
    if [ "$VERBOSE" = true ]; then
        echo -e "${PURPLE}Changes made:${NC}"
        diff -u "${file}.bak" "$file" || true
        echo ""
    fi
    
    # Auto-confirm or ask for confirmation
    if [ "$AUTO_MODE" = "auto" ]; then
        rm "${file}.bak"
        echo -e "${GREEN}Changes automatically saved.${NC}"
        status="Fixed"
        details="Added the following sections:\n${missing_sections_list}"
        FIXED_FILES=$((FIXED_FILES + 1))
    else
        echo -e "${BLUE}Does this look good?${NC}"
        echo "y - Keep all changes"
        echo "n - Discard all changes"
        echo ""
        read -p "Keep these changes? (y/n): " confirm
        
        if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
            rm "${file}.bak"
            echo -e "${GREEN}Changes saved.${NC}"
            status="Fixed"
            details="Added the following sections:\n${missing_sections_list}"
            FIXED_FILES=$((FIXED_FILES + 1))
        else
            mv "${file}.bak" "$file"
            echo -e "${YELLOW}Changes discarded.${NC}"
            status="Changes discarded"
            details="The following sections were proposed but changes were discarded:\n${missing_sections_list}"
        fi
    fi
    
    add_to_report "$file" "$status" "$details"
    return 0
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
    total=$(echo "$total" | tr -d '[:space:]')
    
    if [ -z "$files" ]; then
        echo -e "${YELLOW}No Markdown files found in $dir${NC}"
        return 0
    fi
    
    echo -e "${BLUE}Processing $total Markdown files in $dir...${NC}"
    
    for file in $files; do
        count=$((count + 1))
        TOTAL_FILES=$((TOTAL_FILES + 1))
        
        if [ "$VERBOSE" = true ] || [ "$CHECK_ONLY" = false ]; then
            echo -e "${CYAN}[$count/$total] Processing:${NC} $file"
        fi
        
        if ! process_file "$file"; then
            EXIT_CODE=1
        fi
    done
    
    if [ "$CHECK_ONLY" = true ]; then
        echo -e "${BLUE}Section checking completed with exit code ${EXIT_CODE}${NC}"
        echo -e "${YELLOW}Found $MISSING_SECTIONS_COUNT missing sections in $((TOTAL_FILES - FIXED_FILES)) files${NC}"
        
        if [ "$GENERATE_REPORT" = true ]; then
            echo -e "${GREEN}Report generated:${NC} $REPORT_FILE"
        fi
    else
        echo -e "${GREEN}Processing completed for $count files.${NC}"
        echo -e "${GREEN}- Successfully processed: $FIXED_FILES files${NC}"
        if [ $MISSING_SECTIONS_COUNT -gt 0 ]; then
            echo -e "${YELLOW}- Added $MISSING_SECTIONS_COUNT sections total${NC}"
        fi
        
        if [ "$GENERATE_REPORT" = true ]; then
            echo -e "${GREEN}Report generated:${NC} $REPORT_FILE"
        fi
    fi
}

# Main execution
main() {
    parse_args "$@"
    
    # Initialize report
    init_report
    
    # Print date for logging purposes
    if [ "$VERBOSE" = true ]; then
        echo -e "${BLUE}Current date: $CURRENT_DATE${NC}"
    fi
    
    if [ "$CHECK_ONLY" = true ] || [ "$PROCESS_DIR" = true ]; then
        process_directory "$TARGET_DIR"
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