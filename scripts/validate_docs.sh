#!/bin/bash
# Documentation validation script
# Checks for common issues in documentation

set -e

DOCS_DIR="docs"
REPORT_FILE="doc_validation_report.txt"

echo "Navius Documentation Validation" > $REPORT_FILE
echo "===============================" >> $REPORT_FILE
echo "Date: $(date)" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Check for missing frontmatter
echo "Checking for missing frontmatter..." | tee -a $REPORT_FILE
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^---" "$file"; then
    echo "❌ Missing frontmatter: $file" | tee -a $REPORT_FILE
  fi
done

# Check for broken internal links
echo "" >> $REPORT_FILE
echo "Checking for broken internal links..." | tee -a $REPORT_FILE
find $DOCS_DIR -name "*.md" | while read file; do
  links=$(grep -o "\[.*\](\.\./.*\.md)" "$file" | grep -o "(.*)" | tr -d "()")
  for link in $links; do
    target_file="$(dirname "$file")/$link"
    target_file=$(echo "$target_file" | sed 's/\/\.\.\//\//g')
    if [ ! -f "$target_file" ]; then
      echo "❌ Broken link in $file: $link" | tee -a $REPORT_FILE
    fi
  done
done

# Check for missing related documents
echo "" >> $REPORT_FILE
echo "Checking for missing related documents in frontmatter..." | tee -a $REPORT_FILE
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if grep -q "^related:" "$file"; then
    related_section=$(sed -n "/^related:/,/^[a-z].*:/p" "$file" | grep -v "^[a-z].*:")
    related_files=$(echo "$related_section" | grep "^ *- " | sed 's/ *- //')
    for related in $related_files; do
      target_file="$(dirname "$file")/$related"
      target_file=$(echo "$target_file" | sed 's/\/\.\.\//\//g')
      if [ ! -f "$target_file" ]; then
        echo "❌ Missing related document in $file: $related" | tee -a $REPORT_FILE
      fi
    done
  fi
done

# Check for inconsistent headings
echo "" >> $REPORT_FILE
echo "Checking for inconsistent heading structures..." | tee -a $REPORT_FILE
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^# " "$file"; then
    echo "❌ Missing top-level heading (# Title) in $file" | tee -a $REPORT_FILE
  fi
  
  if ! grep -q "^## Overview" "$file"; then
    echo "⚠️ Missing overview section (## Overview) in $file" | tee -a $REPORT_FILE
  fi
done

# Check for standard sections in guides
echo "" >> $REPORT_FILE
echo "Checking for standard sections in guides..." | tee -a $REPORT_FILE
find $DOCS_DIR/guides -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^## Prerequisites" "$file"; then
    echo "⚠️ Missing prerequisites section in $file" | tee -a $REPORT_FILE
  fi
  
  if ! grep -q "^## Related Documents" "$file"; then
    echo "⚠️ Missing related documents section in $file" | tee -a $REPORT_FILE
  fi
done

# Check for outdated dates in frontmatter
echo "" >> $REPORT_FILE
echo "Checking for outdated dates in frontmatter..." | tee -a $REPORT_FILE
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if grep -q "^last_updated:" "$file"; then
    last_updated=$(grep "^last_updated:" "$file" | sed 's/last_updated: //')
    # This is a simple check - in a real script, you'd compare dates properly
    if [[ "$last_updated" < "2024" ]]; then
      echo "⚠️ Possibly outdated content in $file (last_updated: $last_updated)" | tee -a $REPORT_FILE
    fi
  fi
done

# Generate summary
echo "" >> $REPORT_FILE
echo "Validation Summary" >> $REPORT_FILE
echo "=================" >> $REPORT_FILE
echo "Total files checked: $(find $DOCS_DIR -name "*.md" | wc -l)" >> $REPORT_FILE
echo "Files with missing frontmatter: $(grep -c "Missing frontmatter" $REPORT_FILE)" >> $REPORT_FILE
echo "Broken internal links: $(grep -c "Broken link" $REPORT_FILE)" >> $REPORT_FILE
echo "Missing related documents: $(grep -c "Missing related document" $REPORT_FILE)" >> $REPORT_FILE
echo "Files with inconsistent headings: $(grep -c "Missing top-level heading\|Missing overview section" $REPORT_FILE)" >> $REPORT_FILE
echo "Guides with missing standard sections: $(grep -c "Missing prerequisites section\|Missing related documents section" $REPORT_FILE)" >> $REPORT_FILE
echo "Files with potentially outdated content: $(grep -c "Possibly outdated content" $REPORT_FILE)" >> $REPORT_FILE

echo ""
echo "Documentation validation complete. Report saved to $REPORT_FILE"
echo "For detailed results, review the report file." 