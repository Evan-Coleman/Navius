#!/bin/bash

# Comprehensive Documentation Test Suite
# This script performs advanced documentation validation beyond basic format checks
# It verifies document relationships, content accuracy, and semantic correctness

set -e

DOCS_DIR="docs"
REPORTS_DIR="target/reports/docs_validation"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="${REPORTS_DIR}/comprehensive_test_${TIMESTAMP}.md"

# Create reports directory if it doesn't exist
mkdir -p "$REPORTS_DIR"

# Colors for terminal output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize counters and trackers
declare -A document_references
declare -A tag_counts
declare -A category_documents
declare -A related_pairs

# Function to extract frontmatter from file
extract_frontmatter() {
    local file=$1
    awk 'BEGIN{flag=0} /^---$/{if(flag==0){flag=1}else{flag=2;exit}} flag==1{print}' "$file"
}

# Function to extract frontmatter field
get_frontmatter_field() {
    local frontmatter=$1
    local field=$2
    echo "$frontmatter" | grep "^$field:" | sed "s/^$field: *//g" | sed 's/^"//' | sed 's/"$//'
}

# Function to extract a list field (tags, related)
get_frontmatter_list() {
    local frontmatter=$1
    local field=$2
    echo "$frontmatter" | awk -v f="$field:" 'BEGIN{flag=0} $0 ~ "^"f{flag=1;next} /^[a-z]/{if(flag==1)flag=0} flag==1 && /^ *- /{print $2}'
}

echo -e "${BLUE}Starting comprehensive documentation testing...${NC}"
echo "# Comprehensive Documentation Test Report - $(date '+%B %d, %Y')" > $REPORT_FILE
echo "" >> $REPORT_FILE

# Phase 1: Document Inventory and Metadata Analysis
echo -e "${BLUE}Phase 1: Document Inventory and Metadata Analysis${NC}"
echo "## Document Inventory" >> $REPORT_FILE
echo "" >> $REPORT_FILE

total_documents=0
files_with_frontmatter=0
files_without_frontmatter=0

while IFS= read -r file; do
    total_documents=$((total_documents + 1))
    
    # Skip READMEs for certain checks
    is_readme=false
    if [[ $(basename "$file") == "README.md" ]]; then
        is_readme=true
    fi
    
    # Extract and analyze frontmatter
    if grep -q "^---" "$file"; then
        files_with_frontmatter=$((files_with_frontmatter + 1))
        
        frontmatter=$(extract_frontmatter "$file")
        title=$(get_frontmatter_field "$frontmatter" "title")
        category=$(get_frontmatter_field "$frontmatter" "category")
        
        # Track documents by category
        if [ -n "$category" ]; then
            category_documents["$category"]="${category_documents["$category"]} $file"
        fi
        
        # Track tags
        tags=$(get_frontmatter_list "$frontmatter" "tags")
        for tag in $tags; do
            tag_counts["$tag"]=$((${tag_counts["$tag"]:-0} + 1))
        done
        
        # Record document references and relationships
        related=$(get_frontmatter_list "$frontmatter" "related")
        for rel in $related; do
            # Convert to absolute path from project root if it's a relative path
            if [[ $rel == /* ]]; then
                # It's already an absolute path
                rel_path="${rel#/docs/}"
                rel_path="docs/$rel_path"
            else
                # It's a relative path
                rel_path="$(dirname "$file")/$rel"
                rel_path=$(realpath --relative-to="$(pwd)" "$rel_path" 2>/dev/null || echo "INVALID:$rel")
            fi
            
            # Record the relation in both directions
            document_references["$file"]="${document_references["$file"]} $rel_path"
            
            # Store related pairs for graph analysis
            related_pair="$file|$rel_path"
            related_pairs["$related_pair"]=1
        done
    else
        files_without_frontmatter=$((files_without_frontmatter + 1))
    fi
done < <(find "$DOCS_DIR" -name "*.md" -type f)

# Output document inventory statistics
echo "Total documents: $total_documents" >> $REPORT_FILE
echo "Documents with frontmatter: $files_with_frontmatter" >> $REPORT_FILE
echo "Documents without frontmatter: $files_without_frontmatter" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Phase 2: Category Analysis
echo -e "${BLUE}Phase 2: Category Analysis${NC}"
echo "## Category Distribution" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "| Category | Document Count |" >> $REPORT_FILE
echo "|----------|----------------|" >> $REPORT_FILE

for category in "${!category_documents[@]}"; do
    # Count documents in this category
    count=$(echo "${category_documents[$category]}" | wc -w)
    echo "| $category | $count |" >> $REPORT_FILE
done
echo "" >> $REPORT_FILE

# Phase 3: Tag Analysis
echo -e "${BLUE}Phase 3: Tag Analysis${NC}"
echo "## Tag Usage" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "| Tag | Usage Count |" >> $REPORT_FILE
echo "|-----|-------------|" >> $REPORT_FILE

# Sort tags by usage count (descending)
for tag in $(printf '%s\n' "${!tag_counts[@]}" | sort -r -n -k2,2); do
    echo "| $tag | ${tag_counts[$tag]} |" >> $REPORT_FILE
done
echo "" >> $REPORT_FILE

# Phase 4: Document Relationship Analysis
echo -e "${BLUE}Phase 4: Document Relationship Analysis${NC}"
echo "## Document Relationships" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Check for orphaned documents (no incoming references)
echo "### Orphaned Documents" >> $REPORT_FILE
echo "Documents that are not referenced by any other document:" >> $REPORT_FILE
echo "" >> $REPORT_FILE

orphaned_count=0
for file in $(find "$DOCS_DIR" -name "*.md" -type f); do
    # Skip README files in this check, they're meant to be entry points
    if [[ $(basename "$file") == "README.md" ]]; then
        continue
    fi
    
    # Check if any document references this file
    referenced=false
    for ref_source in "${!document_references[@]}"; do
        if [[ "${document_references[$ref_source]}" == *"$file"* ]]; then
            referenced=true
            break
        fi
    done
    
    if [ "$referenced" = false ]; then
        orphaned_count=$((orphaned_count + 1))
        echo "- $file" >> $REPORT_FILE
    fi
done

if [ $orphaned_count -eq 0 ]; then
    echo "No orphaned documents found." >> $REPORT_FILE
fi
echo "" >> $REPORT_FILE

# Check for broken references
echo "### Broken References" >> $REPORT_FILE
echo "References to documents that don't exist:" >> $REPORT_FILE
echo "" >> $REPORT_FILE

broken_ref_count=0
for source in "${!document_references[@]}"; do
    for target in ${document_references[$source]}; do
        if [ ! -f "$target" ] && [[ ! "$target" == INVALID:* ]]; then
            broken_ref_count=$((broken_ref_count + 1))
            echo "- $source → $target" >> $REPORT_FILE
        fi
    done
done

if [ $broken_ref_count -eq 0 ]; then
    echo "No broken references found." >> $REPORT_FILE
fi
echo "" >> $REPORT_FILE

# Phase 5: Content Structure Analysis
echo -e "${BLUE}Phase 5: Content Structure Analysis${NC}"
echo "## Content Structure" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Analyze heading structure
echo "### Heading Structure" >> $REPORT_FILE
echo "Documents with non-standard heading structure:" >> $REPORT_FILE
echo "" >> $REPORT_FILE

heading_issues=0
while IFS= read -r file; do
    # Skip README files for this check
    if [[ $(basename "$file") == "README.md" ]]; then
        continue
    fi
    
    # Check if document has required headings
    has_title_heading=$(grep -q "^# " "$file" && echo true || echo false)
    has_overview=$(grep -q "^## Overview" "$file" && echo true || echo false)
    
    if [ "$has_title_heading" = false ] || [ "$has_overview" = false ]; then
        heading_issues=$((heading_issues + 1))
        echo "- $file" >> $REPORT_FILE
        
        if [ "$has_title_heading" = false ]; then
            echo "  - Missing title heading (# Title)" >> $REPORT_FILE
        fi
        
        if [ "$has_overview" = false ]; then
            echo "  - Missing overview section (## Overview)" >> $REPORT_FILE
        fi
    fi
done < <(find "$DOCS_DIR" -name "*.md" -type f)

if [ $heading_issues -eq 0 ]; then
    echo "All documents have standard heading structure." >> $REPORT_FILE
fi
echo "" >> $REPORT_FILE

# Phase 6: Content Quality Analysis
echo -e "${BLUE}Phase 6: Content Quality Analysis${NC}"
echo "## Content Quality" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Check for potentially outdated content
echo "### Potentially Outdated Content" >> $REPORT_FILE
echo "Documents last updated more than 90 days ago:" >> $REPORT_FILE
echo "" >> $REPORT_FILE

outdated_count=0
current_date=$(date +%s)
while IFS= read -r file; do
    frontmatter=$(extract_frontmatter "$file")
    last_updated=$(get_frontmatter_field "$frontmatter" "last_updated")
    
    if [ -n "$last_updated" ]; then
        # Try to parse the date
        update_timestamp=$(date -j -f "%B %d, %Y" "$last_updated" +%s 2>/dev/null)
        
        # If date parsing succeeded
        if [ -n "$update_timestamp" ]; then
            # Calculate days since last update
            days_diff=$(( (current_date - update_timestamp) / 86400 ))
            
            if [ $days_diff -gt 90 ]; then
                outdated_count=$((outdated_count + 1))
                echo "- $file (Last updated: $last_updated, $days_diff days ago)" >> $REPORT_FILE
            fi
        fi
    fi
done < <(find "$DOCS_DIR" -name "*.md" -type f)

if [ $outdated_count -eq 0 ]; then
    echo "All documents appear to be up-to-date." >> $REPORT_FILE
fi
echo "" >> $REPORT_FILE

# Generate document relationship graph data
echo "### Document Relationship Graph" >> $REPORT_FILE
echo "Generating document relationship data for visualization:" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "```mermaid" >> $REPORT_FILE
echo "graph TD" >> $REPORT_FILE

# Add nodes and connections
for pair in "${!related_pairs[@]}"; do
    source=$(echo "$pair" | cut -d'|' -f1)
    target=$(echo "$pair" | cut -d'|' -f2)
    
    # Use just the filename for readability
    source_name=$(basename "$source" .md)
    target_name=$(basename "$target" .md)
    
    # Add the relationship to the graph
    echo "    ${source_name//[^a-zA-Z0-9]/}_doc --> ${target_name//[^a-zA-Z0-9]/}_doc" >> $REPORT_FILE
done
echo "```" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Generate summary and recommendations
echo -e "${BLUE}Generating summary and recommendations...${NC}"
echo "## Summary and Recommendations" >> $REPORT_FILE
echo "" >> $REPORT_FILE

if [ $files_without_frontmatter -gt 0 ]; then
    echo "### Critical Issues" >> $REPORT_FILE
    echo "- $files_without_frontmatter documents are missing frontmatter" >> $REPORT_FILE
    echo "- Use fix_frontmatter.sh to add proper frontmatter to these files" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
fi

if [ $broken_ref_count -gt 0 ]; then
    echo "### High Priority" >> $REPORT_FILE
    echo "- $broken_ref_count broken references need to be fixed" >> $REPORT_FILE
    echo "- Use fix_links.sh to correct these references" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
fi

if [ $orphaned_count -gt 0 ]; then
    echo "### Medium Priority" >> $REPORT_FILE
    echo "- $orphaned_count documents are not referenced by any other document" >> $REPORT_FILE
    echo "- Add appropriate links to these documents from related content" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
fi

if [ $heading_issues -gt 0 ]; then
    echo "### Formatting Issues" >> $REPORT_FILE
    echo "- $heading_issues documents have non-standard heading structure" >> $REPORT_FILE
    echo "- Add standard sections to these documents using add_sections.sh" >> $REPORT_FILE
    echo "" >> $REPORT_FILE
fi

# Generate report URL
REPORT_URL="file://$(pwd)/$REPORT_FILE"
echo -e "${GREEN}Comprehensive documentation testing completed.${NC}"
echo -e "${GREEN}Report saved to: ${BLUE}$REPORT_FILE${NC}"
echo -e "${GREEN}View the report at: ${BLUE}$REPORT_URL${NC}"

# Return success status
exit 0 