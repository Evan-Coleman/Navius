#!/bin/bash

# Script to add missing sections to a Markdown file
# Usage: ./add_sections.sh <markdown_file> [auto]
# If "auto" is passed as the second parameter, changes will be applied automatically without confirmation

set -e

# Check arguments
if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 <markdown_file> [auto]"
    echo "Example: $0 docs/guides/authentication.md"
    echo "Add 'auto' as second parameter to apply changes automatically without confirmation"
    exit 1
fi

FILE="$1"
AUTO_CONFIRM=""
if [ $# -eq 2 ] && [ "$2" = "auto" ]; then
    AUTO_CONFIRM="auto"
fi

# Verify the file exists and is a markdown file
if [ ! -f "$FILE" ] || [[ "$FILE" != *.md ]]; then
    echo "Error: $FILE is not a valid markdown file."
    exit 1
fi

# Determine document type based on path
get_document_type() {
    local file="$1"
    
    if [[ "$file" == *"/getting-started/"* ]]; then
        echo "getting-started"
    elif [[ "$file" == *"/guides/"* ]]; then
        echo "guide"
    elif [[ "$file" == *"/reference/"* ]]; then
        echo "reference"
    elif [[ "$file" == *"/contributing/"* ]]; then
        echo "contributing"
    elif [[ "$file" == *"/roadmaps/"* ]]; then
        echo "roadmap"
    elif [[ "$file" == *"/architecture/"* ]]; then
        echo "architecture"
    else
        echo "documentation"
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
    echo "✅ Added $section section to $file"
}

# Get appropriate links for related documents based on document type
get_related_doc_examples() {
    local file_type="$1"
    
    case "$file_type" in
        "guide")
            echo "- [Installation Guide](/docs/getting-started/installation.md) - How to install the application\n- [Development Workflow](/docs/guides/development/development-workflow.md) - Development best practices"
            ;;
        "getting-started")
            echo "- [Project Structure](/docs/getting-started/project-structure.md) - Overview of the codebase\n- [First Steps](/docs/getting-started/first-steps.md) - Getting started with the application"
            ;;
        "reference")
            echo "- [API Standards](/docs/reference/standards/api-standards.md) - API design guidelines\n- [Error Handling](/docs/reference/error-handling.md) - Error handling patterns"
            ;;
        "contributing")
            echo "- [Contributing Guide](/docs/contributing/contributing.md) - How to contribute to the project\n- [Development Setup](/docs/getting-started/development-setup.md) - Setting up your development environment"
            ;;
        "roadmap")
            echo "- [Project Structure Roadmap](/docs/roadmaps/completed/11_project_structure_future_improvements.md) - Future improvements\n- [Documentation Overhaul](/docs/roadmaps/12_document_overhaul.md) - Documentation plans"
            ;;
        "architecture")
            echo "- [Project Structure](/docs/architecture/project-structure.md) - Overall structure\n- [Module Dependencies](/docs/architecture/module-dependencies.md) - Dependencies between modules"
            ;;
        *)
            echo "- [Document 1](/docs/path/to/document1.md) - Brief description\n- [Document 2](/docs/path/to/document2.md) - Brief description"
            ;;
    esac
}

# Determine document type and title
doc_type=$(get_document_type "$FILE")
title=$(get_title "$FILE")
missing_sections=()

# We only care about adding the Related Documents section
if ! has_section "$FILE" "Related Documents"; then
    missing_sections+=("Related Documents")
fi

# If no missing sections, exit
if [ ${#missing_sections[@]} -eq 0 ]; then
    echo "No missing sections found in $FILE."
    exit 0
fi

# Make a backup
cp "$FILE" "${FILE}.bak"

# Add missing Related Documents section
for section in "${missing_sections[@]}"; do
    case "$section" in
        "Related Documents")
            related_examples=$(get_related_doc_examples "$doc_type")
            content="\n## Related Documents\n$related_examples\n"
            ;;
    esac
    
    add_section "$FILE" "$section" "$content"
done

# Show diff
echo ""
echo "Changes made:"
diff -u "${FILE}.bak" "$FILE" || true

# Ask user if they want to keep the changes unless auto confirmation is enabled
if [ "$AUTO_CONFIRM" = "auto" ]; then
    rm "${FILE}.bak"
    echo "Changes automatically saved."
else
    read -p "Keep these changes? (y/n): " confirm
    if [[ $confirm == [yY] || $confirm == [yY][eE][sS] ]]; then
        rm "${FILE}.bak"
        echo "Changes saved."
    else
        mv "${FILE}.bak" "$FILE"
        echo "Changes discarded."
    fi
fi 