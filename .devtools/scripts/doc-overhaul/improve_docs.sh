#!/bin/bash

# Documentation Improvement Script
# This is a guided process to improve documentation one file at a time

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}    $1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Welcome message
clear
print_header "Documentation Improvement Process"
echo "This script will help you improve your documentation by addressing common issues."
echo "You can either improve a single file through a step-by-step process,"
echo "or batch process multiple files with the same type of issue."
echo ""
echo "Options:"
echo "1. Complete process for a single file:"
echo "   - Fix frontmatter"
echo "   - Add Related Documents"
echo "   - Fix broken and relative links"
echo ""
echo "2. Batch processing options:"
echo "   - Add frontmatter to ALL files missing it"
echo "   - Add Related Documents to ALL files missing it"
echo "   - Fix broken links in ALL files"
echo "   - Fix relative links in ALL files"
echo ""

# Step 1: Run validation
clear
print_header "Step 1: Validating Documentation"
echo "Running validation to identify issues..."
./scripts/doc-overhaul/detailed_validation.sh

# Step 2: Select a file
clear
print_header "Step 2: Select a File to Improve"
echo "Please choose a file to improve based on the validation results."
echo "You can either enter a specific file path or batch process files with specific issues."
echo ""
echo "Options:"
echo "1. Enter a file path manually (complete process)"
echo "2. Process ALL files missing frontmatter"
echo "3. Process ALL files missing Related Documents sections"
echo "4. Process ALL files with broken links"
echo "5. Process ALL files with relative links"
echo "6. Exit"
echo ""

# Set the report location
REPORTS_DIR="target/reports/docs_validation"
REPORT_FILE="$REPORTS_DIR/doc_validation_report.md"

read -p "Enter your choice (1-6): " file_choice

case $file_choice in
    1)
        read -p "Enter the file path (e.g., docs/guides/authentication.md): " target_file
        ;;
    2)
        # Get files missing frontmatter from validation report
        files=$(grep -A 100 "^## Files Missing Frontmatter" "$REPORT_FILE" | grep -B 100 "^##" | grep "^\- \`" | sed 's/^\- \`//' | sed 's/\`$//' | grep -v "^##")
        
        if [ -z "$files" ]; then
            print_warning "No files found missing frontmatter."
            exit 0
        fi
        
        echo "Files missing frontmatter:"
        counter=1
        total_files=$(echo "$files" | wc -l | xargs)
        echo "Found $total_files files missing frontmatter. Processing all files..."
        
        # Process all files missing frontmatter
        while IFS= read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Only add frontmatter, don't check for missing sections
                ./scripts/doc-overhaul/fix_frontmatter.sh "$file" auto
                print_success "Added frontmatter to $file"
            else
                print_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done <<< "$files"
        
        print_success "Completed adding frontmatter to all $total_files files!"
        exit 0
        ;;
    3)
        # Get files missing Related Documents section from validation report
        files=$(grep -A 100 "^## Files Missing 'Related Documents' Section" "$REPORT_FILE" | grep -B 100 "^##" | grep "^\- \`" | sed 's/^\- \`//' | sed 's/\`$//' | grep -v "^##")
        
        if [ -z "$files" ]; then
            print_warning "No files found missing sections."
            exit 0
        fi
        
        echo "Files missing Related Documents section:"
        counter=1
        total_files=$(echo "$files" | wc -l | xargs)
        echo "Found $total_files files missing Related Documents section. Processing all files..."
        
        # Process all files missing Related Documents section
        while IFS= read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Only add missing sections, don't check for frontmatter
                ./scripts/doc-overhaul/add_sections.sh "$file" auto
                print_success "Added Related Documents section to $file"
            else
                print_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done <<< "$files"
        
        print_success "Completed adding Related Documents section to all $total_files files!"
        exit 0
        ;;
    4)
        # Get files with broken links from validation report
        files=$(grep -A 100 "^## Broken Links" "$REPORT_FILE" | grep -B 100 "^##" | grep "^|" | grep -v "^| File " | grep -v "^|-" | awk -F'|' '{print $2}' | sed 's/^ *//' | sed 's/ *$//' | sort | uniq | grep -v "^$")
        
        if [ -z "$files" ]; then
            print_warning "No files found with broken links."
            exit 0
        fi
        
        echo "Files with broken links:"
        counter=1
        total_files=$(echo "$files" | wc -l | xargs)
        echo "Found $total_files files with broken links. Processing all files..."
        
        # Process all files with broken links
        while IFS= read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Fix broken links
                ./scripts/doc-overhaul/fix_links.sh "$file" auto
                print_success "Fixed links in $file"
            else
                print_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done <<< "$files"
        
        print_success "Completed fixing links in all $total_files files!"
        exit 0
        ;;
    5)
        # Get files with relative links from validation report
        files=$(grep -A 100 "^## Relative Links" "$REPORT_FILE" | grep -B 100 "^##" | grep "^|" | grep -v "^| File " | grep -v "^|-" | awk -F'|' '{print $2}' | sed 's/^ *//' | sed 's/ *$//' | sort | uniq | grep -v "^$")
        
        if [ -z "$files" ]; then
            print_warning "No files found with relative links."
            exit 0
        fi
        
        echo "Files with relative links:"
        counter=1
        total_files=$(echo "$files" | wc -l | xargs)
        echo "Found $total_files files with relative links. Processing all files..."
        
        # Process all files with relative links
        while IFS= read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Fix relative links
                ./scripts/doc-overhaul/fix_links.sh "$file" auto
                print_success "Fixed links in $file"
            else
                print_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done <<< "$files"
        
        print_success "Completed fixing links in all $total_files files!"
        exit 0
        ;;
    6)
        echo "Exiting."
        exit 0
        ;;
    *)
        print_error "Invalid choice. Exiting."
        exit 1
        ;;
esac

# Verify target file exists
if [ "$file_choice" = "1" ]; then
    if [ ! -f "$target_file" ]; then
        print_error "File not found: $target_file"
        exit 1
    fi
    
    print_success "Selected file: $target_file"
    echo ""
    
    # Step 3: Fix frontmatter and add Related Documents
    clear
    print_header "Step 3: Fix Frontmatter and Add Related Documents"
    echo "Checking if frontmatter and Related Documents need to be added..."
    
    if head -n 20 "$target_file" | grep -q "^---" && head -n 20 "$target_file" | grep -q "^title:"; then
        print_success "File already has frontmatter."
    else
        echo "File is missing frontmatter. Adding it now..."
        ./scripts/doc-overhaul/fix_frontmatter.sh "$target_file"
    fi
    
    echo ""
    echo "Now checking if Related Documents section needs to be added..."
    ./scripts/doc-overhaul/add_sections.sh "$target_file"
    
    # Step 4: Fix broken links
    clear
    print_header "Step 4: Fix Broken and Relative Links"
    echo "Checking for broken and relative links..."
    ./scripts/doc-overhaul/fix_links.sh "$target_file"
    
    # If broken links were found, the fix_links.sh script would have prompted for user input
    # No need to pause here if there were no issues
    
    # Final message
    clear
    print_header "Documentation Improvement Complete"
    echo "You've successfully improved the file: $target_file"
    echo "To continue improving more files, run this script again."
    echo ""
    echo "Next steps:"
    echo "1. Run './scripts/doc-overhaul/improve_docs.sh' to improve another file"
    echo "2. Run './scripts/doc-overhaul/detailed_validation.sh' to see remaining issues"
    echo ""
    print_success "Happy documenting!"
fi 