#!/bin/bash

# CI Documentation Validation Script
# This is the primary validation script used in CI pipelines for documentation checks.
# It now includes both basic validation and comprehensive document analysis.

set -e

SCRIPT_DIR="$(dirname "$0")"
DOCS_DIR="docs"
REPORT_FILE="doc_validation_report.txt"
REPORTS_DIR="target/reports/docs_validation"
CI_MODE=${CI_MODE:-false}

# If CI_PIPELINE_ID is set, we're running in CI
if [ -n "$CI_PIPELINE_ID" ]; then
    CI_MODE=true
fi

# Create reports directory
mkdir -p "$REPORTS_DIR"

echo "Navius Documentation Validation" > $REPORT_FILE
echo "===============================" >> $REPORT_FILE
echo "Date: $(date)" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Ensure required tools are installed
echo "Installing documentation validation tools..."

if [ "$CI_MODE" = true ]; then
    # CI environment installations
    npm install -g markdownlint-cli
    npm install -g markdown-link-check
    pip3 install pyyaml
else
    # Local environment - only install if missing
    if ! command -v markdownlint &> /dev/null; then
        npm install -g markdownlint-cli
    fi
    
    if ! command -v markdown-link-check &> /dev/null; then
        npm install -g markdown-link-check
    fi
    
    # Check if PyYAML is installed
    if ! python3 -c "import yaml" &> /dev/null; then
        pip3 install pyyaml
    fi
fi

# Run markdown linting
echo "Running markdown linting..."
markdownlint docs/**/*.md --config .devtools/config/markdownlint.json | tee -a $REPORT_FILE
LINTING_STATUS=$?

# Check for broken links using markdown-link-check
echo "Checking for broken links..."
find docs -name "*.md" -exec markdown-link-check {} \; | tee -a $REPORT_FILE
LINKS_STATUS=$?

# Check for missing frontmatter
echo "Checking for missing frontmatter..." | tee -a $REPORT_FILE
FRONTMATTER_MISSING=0
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^---" "$file"; then
    echo "❌ Missing frontmatter: $file" | tee -a $REPORT_FILE
    FRONTMATTER_MISSING=1
  fi
done

# Run Python frontmatter validation
echo "Validating frontmatter..."
python3 .devtools/scripts/validate_frontmatter.py | tee -a $REPORT_FILE
FRONTMATTER_STATUS=$?

# Check for broken internal links
echo "Checking for broken internal links..." | tee -a $REPORT_FILE
BROKEN_INTERNAL=0
find $DOCS_DIR -name "*.md" | while read file; do
  links=$(grep -o "\[.*\](\.\./.*\.md)" "$file" | grep -o "(.*)" | tr -d "()")
  for link in $links; do
    target_file="$(dirname "$file")/$link"
    target_file=$(echo "$target_file" | sed 's/\/\.\.\//\//g')
    if [ ! -f "$target_file" ]; then
      echo "❌ Broken link in $file: $link" | tee -a $REPORT_FILE
      BROKEN_INTERNAL=1
    fi
  done
done

# Check for missing related documents
echo "Checking for missing related documents in frontmatter..." | tee -a $REPORT_FILE
MISSING_RELATED=0
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if grep -q "^related:" "$file"; then
    related_section=$(sed -n "/^related:/,/^[a-z].*:/p" "$file" | grep -v "^[a-z].*:")
    related_files=$(echo "$related_section" | grep "^ *- " | sed 's/ *- //')
    for related in $related_files; do
      target_file="$(dirname "$file")/$related"
      target_file=$(echo "$target_file" | sed 's/\/\.\.\//\//g')
      if [ ! -f "$target_file" ]; then
        echo "❌ Missing related document in $file: $related" | tee -a $REPORT_FILE
        MISSING_RELATED=1
      fi
    done
  fi
done

# Check for inconsistent headings
echo "Checking for inconsistent heading structures..." | tee -a $REPORT_FILE
HEADING_ISSUES=0
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^# " "$file"; then
    echo "❌ Missing top-level heading (# Title) in $file" | tee -a $REPORT_FILE
    HEADING_ISSUES=1
  fi
  
  if ! grep -q "^## Overview" "$file"; then
    echo "⚠️ Missing overview section (## Overview) in $file" | tee -a $REPORT_FILE
    HEADING_ISSUES=1
  fi
done

# Check for standard sections in guides
echo "Checking for standard sections in guides..." | tee -a $REPORT_FILE
MISSING_SECTIONS=0
find $DOCS_DIR/guides -name "*.md" | grep -v "README.md" | while read file; do
  if ! grep -q "^## Prerequisites" "$file"; then
    echo "⚠️ Missing prerequisites section in $file" | tee -a $REPORT_FILE
    MISSING_SECTIONS=1
  fi
  
  if ! grep -q "^## Related Documents" "$file"; then
    echo "⚠️ Missing related documents section in $file" | tee -a $REPORT_FILE
    MISSING_SECTIONS=1
  fi
done

# Check for outdated dates in frontmatter
echo "Checking for outdated dates in frontmatter..." | tee -a $REPORT_FILE
OUTDATED_COUNT=0
find $DOCS_DIR -name "*.md" | grep -v "README.md" | while read file; do
  if grep -q "^last_updated:" "$file"; then
    last_updated=$(grep "^last_updated:" "$file" | sed 's/last_updated: //')
    # This is a simple check for dates older than 2023
    if [[ "$last_updated" < "January 01, 2023" ]]; then
      echo "⚠️ Possibly outdated content in $file (last_updated: $last_updated)" | tee -a $REPORT_FILE
      OUTDATED_COUNT=1
    fi
  fi
done

# Run comprehensive document analysis if not in CI mode
# In CI, we focus on the basics for speed and reliability
if [ "$CI_MODE" = false ]; then
    echo "Running comprehensive document analysis..."
    
    # Run the comprehensive test script if it exists
    if [ -f ".devtools/scripts/doc-overhaul/comprehensive_test.sh" ]; then
        bash .devtools/scripts/doc-overhaul/comprehensive_test.sh
        COMPREHENSIVE_STATUS=$?
        
        # Add the result to our report
        if [ $COMPREHENSIVE_STATUS -eq 0 ]; then
            echo "✅ Comprehensive document analysis completed successfully." | tee -a $REPORT_FILE
        else
            echo "⚠️ Comprehensive document analysis reported issues." | tee -a $REPORT_FILE
        fi
    else
        echo "⚠️ Comprehensive test script not found, skipping advanced analysis." | tee -a $REPORT_FILE
    fi
fi

# Generate summary
echo "" >> $REPORT_FILE
echo "Validation Summary" >> $REPORT_FILE
echo "=================" >> $REPORT_FILE
echo "Total files checked: $(find $DOCS_DIR -name "*.md" | wc -l)" >> $REPORT_FILE
echo "Files with linting issues: $LINTING_STATUS" >> $REPORT_FILE
echo "Files with missing frontmatter: $FRONTMATTER_MISSING" >> $REPORT_FILE
echo "Files with broken internal links: $BROKEN_INTERNAL" >> $REPORT_FILE
echo "Files with missing related documents: $MISSING_RELATED" >> $REPORT_FILE
echo "Files with inconsistent headings: $HEADING_ISSUES" >> $REPORT_FILE
echo "Guides with missing standard sections: $MISSING_SECTIONS" >> $REPORT_FILE
echo "Files with potentially outdated content: $OUTDATED_COUNT" >> $REPORT_FILE

echo ""
echo "Documentation validation complete. Report saved to $REPORT_FILE"
echo "For detailed results, review the report file."

# Determine exit code for CI
# In CI, we fail the build if there are critical issues
if [ "$CI_MODE" = true ]; then
    # Critical issues that should fail the build in CI
    if [ $FRONTMATTER_MISSING -ne 0 ] || [ $BROKEN_INTERNAL -ne 0 ] || [ $LINTING_STATUS -ne 0 ]; then
        echo "❌ Critical documentation issues found. Please fix before merging."
        exit 1
    fi
    
    # Non-critical issues - warn but don't fail the build
    if [ $MISSING_RELATED -ne 0 ] || [ $HEADING_ISSUES -ne 0 ] || [ $MISSING_SECTIONS -ne 0 ] || [ $OUTDATED_COUNT -ne 0 ]; then
        echo "⚠️ Non-critical documentation issues found. Consider fixing these."
    fi
fi

# Success
exit 0 