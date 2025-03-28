#!/bin/bash

# Simple script to analyze documentation and generate CSV inventory
# Output: CSV with filename, word count, section count, and tag count

OUTPUT_DIR="target/reports/doc_migration"
CSV_FILE="${OUTPUT_DIR}/doc_inventory_$(date +%Y%m%d_%H%M%S).csv"

mkdir -p "$OUTPUT_DIR"

echo "Filename,WordCount,SectionCount,HasFrontmatter,TagCount,HasCode" > "$CSV_FILE"

find docs -name "*.md" | sort | while read -r file; do
    echo "Analyzing $file..."
    
    # Count words (excluding frontmatter)
    word_count=$(cat "$file" | sed '/^---/,/^---$/d' | wc -w | tr -d ' ')
    
    # Count sections (headers starting with #)
    section_count=$(grep -c "^#" "$file")
    
    # Check if file has frontmatter
    has_frontmatter=0
    grep -q "^---" "$file" && has_frontmatter=1
    
    # Count tags in frontmatter
    tag_count=0
    if [ "$has_frontmatter" -eq 1 ]; then
        tag_count=$(sed -n '/^---/,/^---$/p' "$file" | grep -c "  -")
    fi
    
    # Check if file has code blocks
    has_code=0
    grep -q "^\`\`\`" "$file" && has_code=1
    
    # Output to CSV
    echo "\"$file\",$word_count,$section_count,$has_frontmatter,$tag_count,$has_code" >> "$CSV_FILE"
done

echo "Analysis complete. Results saved to $CSV_FILE"
echo "Summary:"
echo "--------"
echo "Total files: $(wc -l "$CSV_FILE" | awk '{print $1-1}')"
echo "Files with frontmatter: $(grep -c ",1," "$CSV_FILE")"
echo "Files with code blocks: $(grep -c ",1$" "$CSV_FILE")"
echo "Average word count: $(awk -F',' 'NR>1 {sum+=$2; count++} END {print sum/count}' "$CSV_FILE")" 