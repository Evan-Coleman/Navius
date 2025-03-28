#!/bin/sh
# Script to add missing sections to Markdown files based on document type

SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

set_strict_mode

# Default configuration
TARGET_DIR="docs"
SINGLE_FILE=""
CHECK_ONLY=false
RECURSIVE=false
GENERATE_REPORT=false
AUTO_CONFIRM=false
VERBOSE=false
ADD_ALL_SECTIONS=false
CUSTOM_SECTIONS=""
REPORTS_DIR="target/reports/docs_validation"
REPORT_FILE="${REPORTS_DIR}/sections_report_$(get_today_date)_$(date '+%H%M%S').md"

print_usage() {
    echo "Usage: add_sections.sh [OPTIONS]"
    echo "Options:"
    echo "  --dir DIRECTORY           Process markdown files in specific directory"
    echo "  --file FILE               Process a single file only"
    echo "  --recursive, -r           Process directories recursively"
    echo "  --check-only              Only check for missing sections without making changes"
    echo "  --report                  Generate a detailed report of validation results"
    echo "  --auto                    Apply changes automatically without confirmation"
    echo "  --verbose, -v             Show detailed information about each file"
    echo "  --add-all                 Add all possible sections appropriate for document type"
    echo "  --sections \"sec1,sec2\"    Specify custom sections to add (comma-separated)"
    echo "  --help                    Display this help message"
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
        --add-all)
            ADD_ALL_SECTIONS=true
            shift
            ;;
        --sections)
            if [ -z "$2" ] || [ "${2:0:1}" = "-" ]; then
                log_error "Error: --sections requires a comma-separated list"
                print_usage
                exit 1
            fi
            CUSTOM_SECTIONS="$2"
            shift 2
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
        echo "# Section Validation Report"
        echo "Generated on: $(date)"
        echo 
        echo "## Summary"
        echo 
        echo "| Metric | Count |"
        echo "|--------|-------|"
        echo "| Total files | 0 |"
        echo "| Files with missing sections | 0 |"
        echo "| Total missing sections | 0 |"
        echo "| Fixed files | 0 |"
        echo 
        echo "## Detailed Results"
        echo 
    } > "$REPORT_FILE"
fi

# Counters for reporting
TOTAL_FILES=0
FIXED_FILES=0
MISSING_SECTIONS_COUNT=0

# Get standard sections for document type
get_standard_sections() {
    doc_type="$1"
    
    # Common sections for all document types
    common_sections="Related Documents"
    
    if [ "$ADD_ALL_SECTIONS" = true ]; then
        case "$doc_type" in
            getting-started)
                echo "Overview Prerequisites Installation Configuration Usage Related Documents"
                ;;
            guide)
                echo "Overview Prerequisites Step-by-Step Guide Advanced Configuration Examples Troubleshooting Related Documents"
                ;;
            tutorial)
                echo "Overview Prerequisites Setup Steps Summary Next Steps Related Documents"
                ;;
            reference)
                echo "Overview API Reference Examples Configuration Options Related Documents"
                ;;
            architecture)
                echo "Overview Design Principles Components Data Flow Deployment Related Documents"
                ;;
            concept)
                echo "Overview Key Concepts Implementation Details Trade-offs Related Documents"
                ;;
            roadmap)
                echo "Overview Current State Target State Implementation Phases Dependencies Risks Progress Tracking Related Documents"
                ;;
            example)
                echo "Overview Code Example Explanation Usage Notes Related Documents"
                ;;
            faq)
                echo "Overview Frequently Asked Questions Related Documents"
                ;;
            standard)
                echo "Overview Rules Best Practices Exceptions Examples Related Documents"
                ;;
            misc)
                echo "Overview Content Related Documents"
                ;;
            *)
                echo "$common_sections"
                ;;
        esac
    else
        # If custom sections are provided, use them
        if [ -n "$CUSTOM_SECTIONS" ]; then
            echo "$CUSTOM_SECTIONS Related Documents"
        else
            # Otherwise only include the required Related Documents section
            echo "$common_sections"
        fi
    fi
}

# Check which sections exist in a file
check_existing_sections() {
    file="$1"
    
    # Read the file content
    content=$(cat "$file")
    
    # Check for each section by looking for "## Section Name" pattern
    existing_sections=""
    
    # Get all level 2 headings (## Heading)
    headings=$(echo "$content" | grep -E "^## [A-Za-z0-9 ]+" | sed 's/^## //')
    
    # Convert to space-separated list
    for heading in $headings; do
        if [ -z "$existing_sections" ]; then
            existing_sections="$heading"
        else
            existing_sections="$existing_sections $heading"
        fi
    done
    
    echo "$existing_sections"
}

# Get missing sections
get_missing_sections() {
    file="$1"
    
    # Determine document type
    doc_type=$(get_document_type "$file")
    
    # Get standard sections for this document type
    standard_sections=$(get_standard_sections "$doc_type")
    
    # Get existing sections
    existing_sections=$(check_existing_sections "$file")
    
    # Find missing sections
    missing_sections=""
    
    for section in $standard_sections; do
        # Check if section exists
        if ! echo "$existing_sections" | grep -q -w "$section"; then
            if [ -z "$missing_sections" ]; then
                missing_sections="$section"
            else
                missing_sections="$missing_sections $section"
            fi
        fi
    done
    
    echo "$missing_sections"
}

# Generate section content
generate_section_content() {
    section="$1"
    doc_type="$2"
    file="$3"
    
    case "$section" in
        "Related Documents")
            # Generate list of related documents based on document type and path
            related_content=$(generate_related_documents "$file" "$doc_type")
            
            echo "## Related Documents\n\nThe following documents may be useful for additional context:\n\n$related_content\n"
            ;;
        "Overview")
            echo "## Overview\n\nTODO: Add a brief overview of what this document covers.\n"
            ;;
        "Prerequisites")
            echo "## Prerequisites\n\nBefore proceeding, ensure you have the following:\n\n- Requirement 1\n- Requirement 2\n- Requirement 3\n"
            ;;
        "Installation")
            echo "## Installation\n\nFollow these steps to install:\n\n\`\`\`bash\n# Example installation command\n\`\`\`\n"
            ;;
        "Configuration")
            echo "## Configuration\n\nConfigure the settings as follows:\n\n\`\`\`yaml\n# Example configuration\nkey: value\n\`\`\`\n"
            ;;
        "Usage")
            echo "## Usage\n\nUse the feature as follows:\n\n\`\`\`rust\n// Example code\n\`\`\`\n"
            ;;
        "Examples")
            echo "## Examples\n\nHere are some examples of common use cases:\n\n### Example 1\n\n\`\`\`rust\n// Code example 1\n\`\`\`\n\n### Example 2\n\n\`\`\`rust\n// Code example 2\n\`\`\`\n"
            ;;
        "Step-by-Step Guide")
            echo "## Step-by-Step Guide\n\n1. First step\n2. Second step\n3. Third step\n"
            ;;
        "Advanced Configuration")
            echo "## Advanced Configuration\n\nFor advanced use cases, you can configure additional options:\n\n- Option 1: Description\n- Option 2: Description\n"
            ;;
        "Troubleshooting")
            echo "## Troubleshooting\n\n### Common Issue 1\n\n**Problem:** Description of the problem\n\n**Solution:** How to fix it\n\n### Common Issue 2\n\n**Problem:** Description of the problem\n\n**Solution:** How to fix it\n"
            ;;
        "Setup")
            echo "## Setup\n\nPrepare your environment:\n\n\`\`\`bash\n# Setup commands\n\`\`\`\n"
            ;;
        "Steps")
            echo "## Steps\n\n1. First step with detailed explanation\n2. Second step with detailed explanation\n3. Third step with detailed explanation\n"
            ;;
        "Summary")
            echo "## Summary\n\nIn this document, we covered:\n\n- Key point 1\n- Key point 2\n- Key point 3\n"
            ;;
        "Next Steps")
            echo "## Next Steps\n\nAfter completing this tutorial, you might want to explore:\n\n- Related topic 1\n- Related topic 2\n- Advanced techniques\n"
            ;;
        "API Reference")
            echo "## API Reference\n\n### Function/Endpoint 1\n\n**Description:** What it does\n\n**Parameters:**\n- param1: Description\n- param2: Description\n\n**Returns:** What it returns\n\n### Function/Endpoint 2\n\n**Description:** What it does\n\n**Parameters:**\n- param1: Description\n- param2: Description\n\n**Returns:** What it returns\n"
            ;;
        "Design Principles")
            echo "## Design Principles\n\nThis component is designed with these principles in mind:\n\n1. Principle 1: Explanation\n2. Principle 2: Explanation\n3. Principle 3: Explanation\n"
            ;;
        "Components")
            echo "## Components\n\nThe system consists of these main components:\n\n### Component 1\n\nPurpose and responsibility\n\n### Component 2\n\nPurpose and responsibility\n"
            ;;
        "Data Flow")
            echo "## Data Flow\n\nData flows through the system as follows:\n\n1. Input enters through...\n2. Processing happens in...\n3. Output is delivered via...\n\n```mermaid\ngraph LR\n    A[Input] --> B[Process]\n    B --> C[Output]\n```\n"
            ;;
        "Deployment")
            echo "## Deployment\n\nThis component is deployed using:\n\n- Infrastructure requirements\n- Deployment process\n- Configuration details\n"
            ;;
        "Key Concepts")
            echo "## Key Concepts\n\n### Concept 1\n\nExplanation of the concept and its importance\n\n### Concept 2\n\nExplanation of the concept and its importance\n"
            ;;
        "Implementation Details")
            echo "## Implementation Details\n\nThe implementation has these notable characteristics:\n\n- Technical detail 1\n- Technical detail 2\n- Technical detail 3\n"
            ;;
        "Trade-offs")
            echo "## Trade-offs\n\nThis approach has the following trade-offs:\n\n### Advantages\n\n- Advantage 1\n- Advantage 2\n\n### Disadvantages\n\n- Disadvantage 1\n- Disadvantage 2\n"
            ;;
        "Current State")
            echo "## Current State\n\nThe current implementation has these characteristics:\n\n- Feature 1: Status\n- Feature 2: Status\n- Feature 3: Status\n"
            ;;
        "Target State")
            echo "## Target State\n\nAfter implementation, the system will have:\n\n- Capability 1\n- Capability 2\n- Capability 3\n"
            ;;
        "Implementation Phases")
            echo "## Implementation Phases\n\n### Phase 1: Title\n\n- Task 1\n- Task 2\n- Task 3\n\n### Phase 2: Title\n\n- Task 1\n- Task 2\n- Task 3\n"
            ;;
        "Dependencies")
            echo "## Dependencies\n\nThis implementation depends on:\n\n- Dependency 1: Reason\n- Dependency 2: Reason\n- Dependency 3: Reason\n"
            ;;
        "Risks")
            echo "## Risks\n\n| Risk | Impact | Likelihood | Mitigation |\n|------|--------|------------|------------|\n| Risk 1 | High/Medium/Low | High/Medium/Low | How to mitigate |\n| Risk 2 | High/Medium/Low | High/Medium/Low | How to mitigate |\n"
            ;;
        "Progress Tracking")
            echo "## Progress Tracking\n\n| Task | Status | Notes |\n|------|--------|-------|\n| Task 1 | Not Started/In Progress/Complete | Additional details |\n| Task 2 | Not Started/In Progress/Complete | Additional details |\n"
            ;;
        "Code Example")
            echo "## Code Example\n\n\`\`\`rust\n// Example code demonstrating the concept\n\`\`\`\n"
            ;;
        "Explanation")
            echo "## Explanation\n\nThis example works as follows:\n\n1. First, it initializes...\n2. Then, it processes...\n3. Finally, it outputs...\n"
            ;;
        "Usage Notes")
            echo "## Usage Notes\n\nWhen using this example, keep in mind:\n\n- Important note 1\n- Important note 2\n- Important note 3\n"
            ;;
        "Frequently Asked Questions")
            echo "## Frequently Asked Questions\n\n### Question 1?\n\nAnswer to question 1.\n\n### Question 2?\n\nAnswer to question 2.\n\n### Question 3?\n\nAnswer to question 3.\n"
            ;;
        "Rules")
            echo "## Rules\n\n1. Rule 1: Explanation\n2. Rule 2: Explanation\n3. Rule 3: Explanation\n"
            ;;
        "Best Practices")
            echo "## Best Practices\n\n- Practice 1: Rationale\n- Practice 2: Rationale\n- Practice 3: Rationale\n"
            ;;
        "Exceptions")
            echo "## Exceptions\n\nThese standards have the following exceptions:\n\n- Exception 1: When and why\n- Exception 2: When and why\n"
            ;;
        "Content")
            echo "## Content\n\nTODO: Add relevant content for this document.\n"
            ;;
        *)
            echo "## $section\n\nTODO: Add content for this section.\n"
            ;;
    esac
}

# Generate related documents based on document type and path
generate_related_documents() {
    file="$1"
    doc_type="$2"
    
    related_content=""
    
    # Common related documents based on document type
    case "$doc_type" in
        getting-started)
            related_content="- [Installation](/docs/getting-started/installation.md)\n- [Configuration](/docs/getting-started/configuration.md)"
            ;;
        guide)
            related_content="- [Getting Started Guide](/docs/getting-started/overview.md)\n- [API Reference](/docs/reference/api-reference.md)"
            ;;
        tutorial)
            related_content="- [Getting Started Guide](/docs/getting-started/overview.md)\n- [Examples](/docs/examples/README.md)"
            ;;
        reference)
            related_content="- [API Overview](/docs/reference/api-overview.md)\n- [Configuration Reference](/docs/reference/configuration.md)"
            ;;
        architecture)
            related_content="- [System Overview](/docs/architecture/overview.md)\n- [Component Design](/docs/architecture/components.md)"
            ;;
        roadmap)
            related_content="- [Project Roadmap](/docs/roadmaps/README.md)\n- [Contribution Guidelines](/docs/contributing/how-to-contribute.md)"
            ;;
        *)
            related_content="- [Documentation Overview](/docs/README.md)\n- [Getting Started](/docs/getting-started/overview.md)"
            ;;
    esac
    
    # Add dynamically discovered related documents
    # Find markdown files that might be related based on similar path
    dir_path=$(dirname "$file")
    file_basename=$(basename "$file" .md)
    
    # Find files in the same directory
    related_files=$(find "$dir_path" -maxdepth 1 -type f -name "*.md" | grep -v "$file" | head -3)
    
    for related_file in $related_files; do
        related_basename=$(basename "$related_file" .md)
        # Create relative path from docs directory
        related_path=$(echo "$related_file" | sed "s|^docs|/docs|")
        related_content="$related_content\n- [$related_basename]($related_path)"
    done
    
    echo "$related_content"
}

# Add missing sections to a file
add_sections_to_file() {
    file="$1"
    
    if [ ! -f "$file" ]; then
        log_error "File does not exist: $file"
        return 1
    fi
    
    # Determine document type
    doc_type=$(get_document_type "$file")
    
    if [ "$VERBOSE" = true ]; then
        log_info "Document type: $doc_type"
    fi
    
    # Get missing sections
    missing_sections=$(get_missing_sections "$file")
    missing_count=$(echo "$missing_sections" | wc -w)
    
    if [ -z "$missing_sections" ]; then
        if [ "$VERBOSE" = true ]; then
            log_success "No missing sections in $file"
        fi
        return 0
    fi
    
    # In check-only mode, just report missing sections
    if [ "$CHECK_ONLY" = true ]; then
        log_warning "Missing sections in $file: $missing_sections"
        MISSING_SECTIONS_COUNT=$((MISSING_SECTIONS_COUNT + missing_count))
        
        # Add to report
        if [ "$GENERATE_REPORT" = true ]; then
            add_to_report "$file" "Missing sections" "Missing sections: $missing_sections"
        fi
        
        return 1
    fi
    
    # Create a temporary file
    temp_file=$(mktemp)
    
    # Add content up to frontmatter, if any
    if grep -q "^---" "$file"; then
        # File has frontmatter
        front_part=$(sed -n '1,/^---$/p' "$file" | head -1)
        
        if [ "$front_part" = "---" ]; then
            # Extract frontmatter and content separately
            frontmatter=$(extract_frontmatter "$file")
            content=$(get_content_without_frontmatter "$file")
            
            # Write frontmatter to temp file
            echo "---" > "$temp_file"
            echo "$frontmatter" >> "$temp_file"
            
            # Update last_updated field in frontmatter
            if ! echo "$frontmatter" | grep -q "^last_updated:"; then
                echo "last_updated: $(get_today_date)" >> "$temp_file"
            fi
            
            echo "---" >> "$temp_file"
            
            # Write content to temp file
            echo "$content" >> "$temp_file"
        else
            # No frontmatter, copy the entire file
            cat "$file" > "$temp_file"
        fi
    else
        # No frontmatter, copy the entire file
        cat "$file" > "$temp_file"
    fi
    
    # Prepare section content to add
    sections_to_add=""
    for section in $missing_sections; do
        section_content=$(generate_section_content "$section" "$doc_type" "$file")
        sections_to_add="${sections_to_add}${section_content}\n"
    done
    
    # Remove trailing newline
    sections_to_add=$(echo "$sections_to_add" | sed '$s/\\n$//')
    
    # Add the new sections to the end of the file
    echo "" >> "$temp_file"
    echo -e "$sections_to_add" >> "$temp_file"
    
    # Show diff and ask for confirmation
    if [ "$AUTO_CONFIRM" = true ]; then
        # Automatically apply changes
        mv "$temp_file" "$file"
        log_success "Automatically added missing sections to $file: $missing_sections"
        FIXED_FILES=$((FIXED_FILES + 1))
        
        # Add to report
        if [ "$GENERATE_REPORT" = true ]; then
            add_to_report "$file" "Fixed" "Added sections: $missing_sections"
        fi
    else
        # Show diff and ask for confirmation
        log_info "Missing sections in $file: $missing_sections"
        log_info "Changes to be applied:"
        diff -u "$file" "$temp_file" | grep -v "^---" | grep -v "^+++"
        
        printf "Apply these changes? [y/N]: "
        read -r confirm
        
        if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
            mv "$temp_file" "$file"
            log_success "Added missing sections to $file: $missing_sections"
            FIXED_FILES=$((FIXED_FILES + 1))
            
            # Add to report
            if [ "$GENERATE_REPORT" = true ]; then
                add_to_report "$file" "Fixed" "Added sections: $missing_sections"
            fi
        else
            rm "$temp_file"
            log_info "Skipped adding sections to $file"
            
            # Add to report
            if [ "$GENERATE_REPORT" = true ]; then
                add_to_report "$file" "Skipped" "User chose not to add sections: $missing_sections"
            fi
        fi
    fi
    
    # Update missing sections count
    MISSING_SECTIONS_COUNT=$((MISSING_SECTIONS_COUNT + missing_count))
    
    return 0
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
    
    # Process each file
    for file in $file_list; do
        TOTAL_FILES=$((TOTAL_FILES + 1))
        
        # Process the file
        add_sections_to_file "$file"
    done
    
    # Output summary
    log_info "Directory $dir summary:"
    log_info "  Total files: $TOTAL_FILES"
    log_info "  Files with missing sections: $((TOTAL_FILES - FIXED_FILES))"
    log_info "  Total missing sections: $MISSING_SECTIONS_COUNT"
    
    if [ $FIXED_FILES -gt 0 ]; then
        log_success "  Fixed files: $FIXED_FILES"
    fi
    
    # Update report
    if [ "$GENERATE_REPORT" = true ]; then
        update_report_counts
    fi
    
    # Return error code if files need fixing
    if [ $MISSING_SECTIONS_COUNT -gt 0 ] && [ $FIXED_FILES -lt $TOTAL_FILES ]; then
        return 1
    else
        return 0
    fi
}

# Main execution
exit_code=0

if [ -n "$SINGLE_FILE" ]; then
    TOTAL_FILES=1
    add_sections_to_file "$SINGLE_FILE"
    exit_code=$?
else
    process_directory "$TARGET_DIR" "$RECURSIVE"
    exit_code=$?
    
    if [ "$CHECK_ONLY" = true ]; then
        if [ $MISSING_SECTIONS_COUNT -gt 0 ]; then
            log_warning "Found $MISSING_SECTIONS_COUNT missing sections in $((TOTAL_FILES - FIXED_FILES)) files"
        else
            log_success "All files have required sections!"
        fi
    fi
fi

# Output report location if generated
if [ "$GENERATE_REPORT" = true ]; then
    log_success "Section validation report generated: $REPORT_FILE"
fi

exit $exit_code 