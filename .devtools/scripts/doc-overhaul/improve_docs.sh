#!/bin/sh
# Documentation Improvement Script
# This is a guided process to improve documentation one file at a time

SCRIPT_DIR="$(dirname "$0")"
. "$SCRIPT_DIR/shell_utils.sh"

set_strict_mode

# Configuration
REPORTS_DIR="target/reports/docs_validation"
REPORT_FILE="$REPORTS_DIR/doc_validation_report.md"
QUALITY_REPORT="$REPORTS_DIR/documentation_quality_report_$(get_today_date).md"

# Create necessary directories
ensure_dir "$REPORTS_DIR"

# Welcome message
clear
log_header "Documentation Improvement Process"
log_info "Current date: $(get_today_date)"
echo "This script will help you improve your documentation by addressing common issues."
echo "You can either improve a single file through a step-by-step process,"
echo "or batch process multiple files with the same type of issue."
echo ""
echo "Options:"
echo "1. Complete process for a single file:"
echo "   - Fix frontmatter"
echo "   - Add missing sections"
echo "   - Fix broken and relative links"
echo "   - Validate code examples"
echo ""
echo "2. Batch processing options:"
echo "   - Add frontmatter to ALL files missing it"
echo "   - Add standard sections to ALL files missing them"
echo "   - Fix broken links in ALL files"
echo "   - Fix relative links in ALL files"
echo "   - Update last_updated field to current date"
echo ""
echo "3. Quality assessment:"
echo "   - Generate comprehensive quality report"
echo "   - Check readability metrics"
echo "   - Validate code examples"
echo "   - Create document relationship visualization"
echo ""

# Step 1: Run validation
clear
log_header "Step 1: Validating Documentation"
log_info "Current date: $(get_today_date)"
echo "Running validation to identify issues..."
"$SCRIPT_DIR/generate_report.sh" --skip-linting

# Step 2: Select a file
clear
log_header "Step 2: Select a File or Process to Improve"
log_info "Current date: $(get_today_date)"
echo "Please choose a file to improve based on the validation results."
echo "You can either enter a specific file path or batch process files with specific issues."
echo ""
echo "Options:"
echo "1. Enter a file path manually (complete process)"
echo "2. Process ALL files missing frontmatter"
echo "3. Process ALL files missing required sections"
echo "4. Process ALL files with broken links"
echo "5. Process ALL files with relative links" 
echo "6. Update last_updated field to current date ($(get_today_date)) in ALL files"
echo "7. Generate comprehensive quality report"
echo "8. Exit"
echo ""

read -p "Enter your choice (1-8): " file_choice

case $file_choice in
    1)
        read -p "Enter the file path (e.g., docs/04_guides/authentication.md): " target_file
        ;;
    2)
        # Get files missing frontmatter from validation report
        files=$(grep -A 100 "^## Files Missing Frontmatter" "$REPORT_FILE" | grep -B 100 "^##" | grep "^\- \`" | sed 's/^\- \`//' | sed 's/\`$//' | grep -v "^##")
        
        if [ -z "$files" ]; then
            log_warning "No files found missing frontmatter."
            exit 0
        fi
        
        echo "Files missing frontmatter:"
        counter=1
        total_files=$(echo "$files" | wc -l | tr -d ' ')
        echo "Found $total_files files missing frontmatter. Processing all files..."
        
        # Process all files missing frontmatter
        echo "$files" | while read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Only add frontmatter, don't check for missing sections
                "$SCRIPT_DIR/fix_frontmatter.sh" --file "$file" --auto
                log_success "Added frontmatter to $file"
            else
                log_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done
        
        log_success "Completed adding frontmatter to all $total_files files!"
        exit 0
        ;;
    3)
        # Get files missing required sections from validation report
        files=$(grep -A 100 "^## Files Missing Required Sections" "$REPORT_FILE" | grep -B 100 "^##" | grep "^\- \`" | sed 's/^\- \`//' | sed 's/\`$//' | grep -v "^##")
        
        if [ -z "$files" ]; then
            log_warning "No files found missing required sections."
            exit 0
        fi
        
        echo "Files missing required sections:"
        counter=1
        total_files=$(echo "$files" | wc -l | tr -d ' ')
        echo "Found $total_files files missing required sections. Processing all files..."
        
        # Process all files missing required sections
        echo "$files" | while read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Add missing sections with --add-all flag
                "$SCRIPT_DIR/add_sections.sh" --file "$file" --add-all --auto
                log_success "Added required sections to $file"
            else
                log_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done
        
        log_success "Completed adding required sections to all $total_files files!"
        exit 0
        ;;
    4)
        # Get files with broken links from validation report
        files=$(grep -A 100 "^## Broken Links" "$REPORT_FILE" | grep -B 100 "^##" | grep "^|" | grep -v "^| File " | grep -v "^|-" | awk -F'|' '{print $2}' | sed 's/^ *//' | sed 's/ *$//' | sort | uniq | grep -v "^$")
        
        if [ -z "$files" ]; then
            log_warning "No files found with broken links."
            exit 0
        fi
        
        echo "Files with broken links:"
        counter=1
        total_files=$(echo "$files" | wc -l | tr -d ' ')
        echo "Found $total_files files with broken links. Processing all files..."
        
        # Process all files with broken links
        echo "$files" | while read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Fix broken links
                "$SCRIPT_DIR/fix_links.sh" --file "$file" --auto
                log_success "Fixed links in $file"
            else
                log_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done
        
        log_success "Completed fixing links in all $total_files files!"
        exit 0
        ;;
    5)
        # Get files with relative links from validation report
        files=$(grep -A 100 "^## Relative Links" "$REPORT_FILE" | grep -B 100 "^##" | grep "^|" | grep -v "^| File " | grep -v "^|-" | awk -F'|' '{print $2}' | sed 's/^ *//' | sed 's/ *$//' | sort | uniq | grep -v "^$")
        
        if [ -z "$files" ]; then
            log_warning "No files found with relative links."
            exit 0
        fi
        
        echo "Files with relative links:"
        counter=1
        total_files=$(echo "$files" | wc -l | tr -d ' ')
        echo "Found $total_files files with relative links. Processing all files..."
        
        # Process all files with relative links
        echo "$files" | while read -r file; do
            echo ""
            echo "Processing $counter/$total_files: $file"
            if [ -f "$file" ]; then
                # Fix relative links
                "$SCRIPT_DIR/fix_links.sh" --file "$file" --auto
                log_success "Fixed links in $file"
            else
                log_error "File not found: $file"
            fi
            counter=$((counter + 1))
        done
        
        log_success "Completed fixing links in all $total_files files!"
        exit 0
        ;;
    6)
        # Update last_updated field in all markdown files
        read -p "Enter directory to process (default: docs): " update_dir
        update_dir=${update_dir:-docs}
        
        if [ ! -d "$update_dir" ]; then
            log_error "Directory not found: $update_dir"
            exit 1
        fi
        
        echo "Updating last_updated field to $(get_today_date) in all markdown files in $update_dir..."
        
        find "$update_dir" -type f -name "*.md" | while read -r file; do
            frontmatter=$(extract_frontmatter "$file")
            if [ -n "$frontmatter" ] && echo "$frontmatter" | grep -q "^last_updated:"; then
                # Create a temporary file
                temp_file=$(mktemp)
                
                # Extract frontmatter and content separately
                content=$(get_content_without_frontmatter "$file")
                
                # Update frontmatter
                updated_frontmatter=$(echo "$frontmatter" | sed "s/^last_updated:.*$/last_updated: $(get_today_date)/")
                
                # Write updated content to temp file
                echo "---" > "$temp_file"
                echo "$updated_frontmatter" >> "$temp_file"
                echo "---" >> "$temp_file"
                echo "$content" >> "$temp_file"
                
                # Replace original with updated file
                mv "$temp_file" "$file"
                
                log_success "Updated last_updated field in $file"
            fi
        done
        
        log_success "Completed updating last_updated field in all markdown files!"
        exit 0
        ;;
    7)
        # Generate comprehensive quality report
        clear
        log_header "Generating Comprehensive Quality Report"
        log_info "Current date: $(get_today_date)"
        echo "This will generate a detailed report of documentation quality..."
        echo ""
        echo "Options:"
        echo "1. Generate basic report (faster)"
        echo "2. Generate comprehensive report with visualization"
        echo "3. Go back to main menu"
        echo ""
        
        read -p "Enter your choice (1-3): " report_choice
        
        case $report_choice in
            1)
                "$SCRIPT_DIR/generate_report.sh"
                log_success "Generated basic quality report"
                ;;
            2)
                "$SCRIPT_DIR/generate_report.sh" --vis
                log_success "Generated comprehensive quality report with visualization"
                ;;
            3)
                exec "$0"
                ;;
            *)
                log_error "Invalid choice. Going back to main menu."
                exec "$0"
                ;;
        esac
        
        echo ""
        echo "Report generated: $QUALITY_REPORT"
        echo "Would you like to view the report now?"
        read -p "View report? (y/n): " view_report
        
        if [ "$view_report" = "y" ] || [ "$view_report" = "Y" ]; then
            if command -v less > /dev/null 2>&1; then
                less "$QUALITY_REPORT"
            else
                cat "$QUALITY_REPORT"
            fi
        fi
        
        exit 0
        ;;
    8)
        echo "Exiting."
        exit 0
        ;;
    *)
        log_error "Invalid choice. Exiting."
        exit 1
        ;;
esac

# Verify target file exists
if [ "$file_choice" = "1" ]; then
    if [ ! -f "$target_file" ]; then
        log_error "File not found: $target_file"
        exit 1
    fi
    
    log_success "Selected file: $target_file"
    echo ""
    
    # Step 3: Fix frontmatter and add sections
    clear
    log_header "Step 3: Fix Frontmatter and Add Required Sections"
    log_info "Current date: $(get_today_date)"
    echo "Checking if frontmatter and required sections need to be added..."
    
    # Check if file has frontmatter
    if has_frontmatter "$target_file"; then
        log_success "File already has frontmatter."
        
        # Check if last_updated field needs to be updated
        frontmatter=$(extract_frontmatter "$target_file")
        if echo "$frontmatter" | grep -q "^last_updated:"; then
            current_date=$(echo "$frontmatter" | grep "^last_updated:" | sed 's/^last_updated: *//')
            echo "Current last_updated: $current_date"
            read -p "Update last_updated to $(get_today_date)? (y/n): " update_date
            
            if [ "$update_date" = "y" ] || [ "$update_date" = "Y" ]; then
                # Create a temporary file
                temp_file=$(mktemp)
                
                # Extract content without frontmatter
                content=$(get_content_without_frontmatter "$target_file")
                
                # Update frontmatter
                updated_frontmatter=$(echo "$frontmatter" | sed "s/^last_updated:.*$/last_updated: $(get_today_date)/")
                
                # Write updated content to temp file
                echo "---" > "$temp_file"
                echo "$updated_frontmatter" >> "$temp_file"
                echo "---" >> "$temp_file"
                echo "$content" >> "$temp_file"
                
                # Replace original with updated file
                mv "$temp_file" "$target_file"
                
                log_success "Updated last_updated field to $(get_today_date)"
            fi
        else
            log_warning "File is missing last_updated field in frontmatter."
            echo "Adding this with the current date..."
            
            # Create a temporary file
            temp_file=$(mktemp)
            
            # Extract content without frontmatter
            content=$(get_content_without_frontmatter "$target_file")
            
            # Update frontmatter
            updated_frontmatter="$frontmatter\nlast_updated: $(get_today_date)"
            
            # Write updated content to temp file
            echo "---" > "$temp_file"
            echo -e "$updated_frontmatter" >> "$temp_file"
            echo "---" >> "$temp_file"
            echo "$content" >> "$temp_file"
            
            # Replace original with updated file
            mv "$temp_file" "$target_file"
            
            log_success "Added last_updated field with current date"
        fi
    else
        echo "File is missing frontmatter. Adding it now..."
        "$SCRIPT_DIR/fix_frontmatter.sh" --file "$target_file"
    fi
    
    echo ""
    echo "Now checking if required sections need to be added..."
    echo "Options:"
    echo "1. Add all recommended sections based on document type"
    echo "2. Only add missing required sections (Overview and Related Documents)"
    echo "3. Skip section addition"
    echo ""
    
    read -p "Enter your choice (1-3): " section_choice
    
    case $section_choice in
        1)
            "$SCRIPT_DIR/add_sections.sh" --file "$target_file" --add-all
            ;;
        2)
            "$SCRIPT_DIR/add_sections.sh" --file "$target_file"
            ;;
        3)
            log_warning "Skipping section addition"
            ;;
        *)
            log_error "Invalid choice. Defaulting to option 2."
            "$SCRIPT_DIR/add_sections.sh" --file "$target_file"
            ;;
    esac
    
    # Step 4: Fix broken links
    clear
    log_header "Step 4: Fix Broken and Relative Links"
    log_info "Current date: $(get_today_date)"
    echo "Checking for broken and relative links..."
    "$SCRIPT_DIR/fix_links.sh" --file "$target_file"
    
    # Step 5: Check document quality
    clear
    log_header "Step 5: Document Quality Assessment"
    log_info "Current date: $(get_today_date)"
    echo "Checking document quality..."
    "$SCRIPT_DIR/generate_report.sh" --file "$target_file"
    
    echo ""
    echo "Would you like to see detailed readability metrics for this file?"
    read -p "View readability metrics? (y/n): " view_metrics
    
    if [ "$view_metrics" = "y" ] || [ "$view_metrics" = "Y" ]; then
        # Extract file content without frontmatter for readability analysis
        content_without_frontmatter=$(get_content_without_frontmatter "$target_file")
        
        # Count words, sentences, paragraphs
        word_count=$(echo "$content_without_frontmatter" | wc -w | tr -d ' ')
        sentence_count=$(echo "$content_without_frontmatter" | grep -oE '(\.|!|\?)[ $]' | wc -l | tr -d ' ')
        paragraph_count=$(echo "$content_without_frontmatter" | grep -E '^$' | wc -l | tr -d ' ')
        
        # Calculate words per sentence
        if [ "$sentence_count" -gt 0 ]; then
            words_per_sentence=$(echo "scale=1; $word_count / $sentence_count" | bc)
        else
            words_per_sentence="N/A"
        fi
        
        echo ""
        log_info "Readability Metrics for $target_file:"
        echo "Word count: $word_count"
        echo "Sentence count: $sentence_count"
        echo "Paragraph count: $paragraph_count"
        echo "Words per sentence: $words_per_sentence"
        
        if [ "$words_per_sentence" != "N/A" ]; then
            if [ "$(echo "$words_per_sentence > 20" | bc)" -eq 1 ]; then
                log_warning "Average words per sentence is over 20, which may reduce readability"
                echo "Consider breaking long sentences into shorter ones for better readability."
            else
                log_success "Sentence length is within recommended range for good readability"
            fi
        fi
    fi
    
    # Final message
    clear
    log_header "Documentation Improvement Complete"
    log_info "Current date: $(get_today_date)"
    echo "You've successfully improved the file: $target_file"
    echo "To continue improving more files, run this script again."
    echo ""
    echo "Next steps:"
    echo "1. Run '$SCRIPT_DIR/improve_docs.sh' to improve another file"
    echo "2. Run '$SCRIPT_DIR/generate_report.sh' to see remaining issues"
    echo "3. Add your improved document to version control"
    echo ""
    log_success "Happy documenting!"
fi 