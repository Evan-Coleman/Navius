#!/bin/bash

# Script to prioritize documents for migration based on the inventory

# Find the most recent CSV file
CSV_FILE=$(ls -t target/reports/doc_migration/doc_inventory_*.csv | head -1)
OUTPUT_DIR="target/reports/doc_migration"
PRIORITY_FILE="${OUTPUT_DIR}/priority_list_$(date +%Y%m%d_%H%M%S).md"

echo "Using inventory file: $CSV_FILE"
echo "Generating priority list..."

# Create the priority file with header
cat > "$PRIORITY_FILE" << EOT
# Documentation Migration Priority List

Generated on $(date '+%B %d, %Y')

This document provides a prioritized list of documentation files for migration, based on automated analysis.

## High-Priority Documents

Files that are frequently used, have good structure, and are critical for user onboarding:

| Filename | Word Count | Section Count | Has Frontmatter | Has Code | Priority Score |
|----------|------------|---------------|----------------|----------|----------------|
EOT

# Process getting-started documents (highest priority)
echo "Processing getting-started documents..."
grep "docs/getting-started" "$CSV_FILE" | sort -t, -k3,3nr -k2,2nr | head -10 | while IFS=, read -r filename wordcount sectioncount hasfrontmatter tagcount hascode; do
    # Calculate a simple priority score
    score=$((sectioncount * 5 + wordcount / 100))
    echo "| ${filename//\"/} | $wordcount | $sectioncount | $hasfrontmatter | $hascode | $score |" >> "$PRIORITY_FILE"
done

# Add medium priority section
cat >> "$PRIORITY_FILE" << EOT

## Medium-Priority Documents

Important reference and guide documents:

| Filename | Word Count | Section Count | Has Frontmatter | Has Code | Priority Score |
|----------|------------|---------------|----------------|----------|----------------|
EOT

# Process guides and reference documents (medium priority)
echo "Processing guides and reference documents..."
grep -E "docs/guides|docs/reference" "$CSV_FILE" | sort -t, -k3,3nr -k2,2nr | head -15 | while IFS=, read -r filename wordcount sectioncount hasfrontmatter tagcount hascode; do
    # Calculate a simple priority score
    score=$((sectioncount * 3 + wordcount / 150))
    echo "| ${filename//\"/} | $wordcount | $sectioncount | $hasfrontmatter | $hascode | $score |" >> "$PRIORITY_FILE"
done

# Add low priority section
cat >> "$PRIORITY_FILE" << EOT

## Low-Priority Documents

Supporting documentation that can be migrated later:

| Filename | Word Count | Section Count | Has Frontmatter | Has Code | Priority Score |
|----------|------------|---------------|----------------|----------|----------------|
EOT

# Process other documents (low priority)
echo "Processing other documents..."
grep -v -E "docs/getting-started|docs/guides|docs/reference" "$CSV_FILE" | grep -v "Filename" | sort -t, -k3,3nr -k2,2nr | head -20 | while IFS=, read -r filename wordcount sectioncount hasfrontmatter tagcount hascode; do
    # Calculate a simple priority score
    score=$((sectioncount * 2 + wordcount / 200))
    echo "| ${filename//\"/} | $wordcount | $sectioncount | $hasfrontmatter | $hascode | $score |" >> "$PRIORITY_FILE"
done

# Add statistics section
cat >> "$PRIORITY_FILE" << EOT

## Document Statistics

* **Total Documents**: $(grep -c -v "Filename" "$CSV_FILE")
* **Documents with Frontmatter**: $(grep -c ",1," "$CSV_FILE")
* **Documents with Code Blocks**: $(grep -c ",1$" "$CSV_FILE")
* **Average Word Count**: $(awk -F',' 'NR>1 {sum+=$2; count++} END {print int(sum/count)}' "$CSV_FILE")
* **Average Section Count**: $(awk -F',' 'NR>1 {sum+=$3; count++} END {print int(sum/count)}' "$CSV_FILE")

## Migration Recommendations

1. Start with highest priority getting-started documents
2. Next, migrate key guides and reference documentation
3. Finally, migrate supporting documentation

## Quality Improvement Opportunities

* **Missing Frontmatter**: $(grep -c ",0," "$CSV_FILE") documents
* **Missing Code Examples**: $(grep -c ",0$" "$CSV_FILE") documents
* **Low Section Count**: $(awk -F',' 'NR>1 && $3<3 {count++} END {print count}' "$CSV_FILE") documents with fewer than 3 sections
EOT

echo "Priority list generated: $PRIORITY_FILE" 