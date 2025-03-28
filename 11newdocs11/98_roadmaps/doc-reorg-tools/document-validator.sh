#!/usr/bin/env bash

# document-validator.sh
# Script to validate migrated documents according to the tiered approach
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/doc-validation"
TIER1_VALIDATION=100  # 100% validation
TIER2_VALIDATION=50   # 50% validation
TIER3_VALIDATION=20   # 20% spot validation

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to get or create tier assignments
get_or_create_tier_assignments() {
    local tier_file="$OUTPUT_DIR/tier_assignments.csv"
    
    if [[ ! -f "$tier_file" ]]; then
        echo "Document Path,Category,Tier Assignment,Priority,Status" > "$tier_file"
        echo "Creating new tier assignments..."
        
        # Find all markdown files
        find "$DOCS_DIR" -name "*.md" | while read -r doc; do
            local category=$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
            local tier="3"  # Default to Tier 3
            local priority="Low"
            
            # Assign Tier 1 to core documents (adjust these rules based on project needs)
            if [[ "$doc" == *"01_getting_started"* ]] || 
               [[ "$doc" == *"README.md"* ]] || 
               [[ "$doc" == *"/installation.md"* ]] || 
               [[ "$doc" == *"/hello-world.md"* ]] || 
               [[ "$doc" == *"/quick-start.md"* ]]; then
                tier="1"
                priority="High"
            # Assign Tier 2 to important but not critical documents
            elif [[ "$doc" == *"02_examples"* ]] || 
                 [[ "$doc" == *"/api/"* ]] || 
                 [[ "$doc" == *"/reference/"* ]]; then
                tier="2"
                priority="Medium"
            fi
            
            echo "$doc,$category,$tier,$priority,Pending" >> "$tier_file"
        done
        
        echo "Created tier assignments for $(grep -c "," "$tier_file") documents"
    else
        echo "Using existing tier assignments from $tier_file"
        echo "Found $(grep -c "," "$tier_file") documents with tier assignments"
    fi
    
    return 0
}

# Calculate sample sizes for each tier
calculate_samples() {
    local tier_file="$1"
    
    # Count documents in each tier
    local tier1_count=$(grep ",1," "$tier_file" | wc -l | tr -d ' ')
    local tier2_count=$(grep ",2," "$tier_file" | wc -l | tr -d ' ')
    local tier3_count=$(grep ",3," "$tier_file" | wc -l | tr -d ' ')
    
    # Calculate samples based on validation percentages
    local tier1_sample=$((tier1_count * TIER1_VALIDATION / 100))
    local tier2_sample=$((tier2_count * TIER2_VALIDATION / 100))
    local tier3_sample=$((tier3_count * TIER3_VALIDATION / 100))
    
    # Ensure we validate at least one document per tier if available
    [[ $tier1_sample -eq 0 && $tier1_count -gt 0 ]] && tier1_sample=1
    [[ $tier2_sample -eq 0 && $tier2_count -gt 0 ]] && tier2_sample=1
    [[ $tier3_sample -eq 0 && $tier3_count -gt 0 ]] && tier3_sample=1
    
    echo "Document count by tier:"
    echo "- Tier 1 (100% validation): $tier1_count documents, validating $tier1_sample"
    echo "- Tier 2 (50% validation): $tier2_count documents, validating $tier2_sample"
    echo "- Tier 3 (Spot validation): $tier3_count documents, validating $tier3_sample"
    
    # Return sample sizes
    echo "$tier1_sample $tier2_sample $tier3_sample"
}

# Function to validate frontmatter
validate_frontmatter() {
    local doc="$1"
    local result_file="$2"
    
    # Extract frontmatter from markdown file
    local frontmatter=$(sed -n '/^---$/,/^---$/p' "$doc")
    
    # Check required frontmatter fields
    local issues=""
    
    [[ "$frontmatter" != *"title:"* ]] && issues="${issues}Missing title field; "
    [[ "$frontmatter" != *"description:"* ]] && issues="${issues}Missing description field; "
    [[ "$frontmatter" != *"category:"* ]] && issues="${issues}Missing category field; "
    [[ "$frontmatter" != *"tags:"* ]] && issues="${issues}Missing tags field; "
    [[ "$frontmatter" != *"last_updated:"* ]] && issues="${issues}Missing last_updated field; "
    [[ "$frontmatter" != *"version:"* ]] && issues="${issues}Missing version field; "
    
    if [[ -z "$issues" ]]; then
        echo "✓ Frontmatter valid" >> "$result_file"
        return 0
    else
        echo "✗ Frontmatter issues: $issues" >> "$result_file"
        return 1
    fi
}

# Function to validate document structure
validate_structure() {
    local doc="$1"
    local result_file="$2"
    
    local issues=""
    
    # Check for title heading
    if ! grep -q "^# " "$doc"; then
        issues="${issues}Missing title heading (# Title); "
    fi
    
    # Check for overview section
    if ! grep -q "^## Overview" "$doc"; then
        issues="${issues}Missing Overview section; "
    fi
    
    # Check for content depth (at least some subheadings)
    if [[ $(grep -c "^##" "$doc") -lt 2 ]]; then
        issues="${issues}Insufficient content structure (needs more sections); "
    fi
    
    if [[ -z "$issues" ]]; then
        echo "✓ Document structure valid" >> "$result_file"
        return 0
    else
        echo "✗ Structure issues: $issues" >> "$result_file"
        return 1
    fi
}

# Function to validate code examples
validate_code_examples() {
    local doc="$1"
    local result_file="$2"
    
    # Check if document has code examples
    if ! grep -q "\`\`\`" "$doc"; then
        echo "- No code examples to validate" >> "$result_file"
        return 0
    fi
    
    # Count code blocks
    local code_blocks=$(grep -c "\`\`\`" "$doc")
    local code_blocks_count=$((code_blocks / 2))
    
    # Count Rust code blocks specifically
    local rust_blocks=$(grep -c "\`\`\`rust" "$doc")
    
    echo "- Found $code_blocks_count code blocks ($rust_blocks Rust blocks)" >> "$result_file"
    
    # Check for common issues with Rust code examples
    local issues=""
    
    # Check for use statements in Rust code blocks
    if grep -q "\`\`\`rust" "$doc" && ! grep -A3 "\`\`\`rust" "$doc" | grep -q "use "; then
        issues="${issues}Rust code blocks may be missing imports; "
    fi
    
    # Check for function definitions in Rust code blocks
    if grep -q "\`\`\`rust" "$doc" && ! grep -A10 "\`\`\`rust" "$doc" | grep -q "fn "; then
        issues="${issues}Rust code blocks may be missing function definitions; "
    fi
    
    if [[ -z "$issues" ]]; then
        echo "✓ Code examples appear valid" >> "$result_file"
        return 0
    else
        echo "✗ Code example issues: $issues" >> "$result_file"
        return 1
    fi
}

# Function to validate internal links
validate_links() {
    local doc="$1"
    local result_file="$2"
    
    # Check if document has internal links
    if ! grep -q "\[.*\](.*\.md)" "$doc"; then
        echo "- No internal links to validate" >> "$result_file"
        return 0
    fi
    
    # Count internal links
    local internal_links=$(grep -c "\[.*\](.*\.md)" "$doc")
    
    echo "- Found $internal_links internal links" >> "$result_file"
    
    # Check each link
    local broken_links=0
    local working_links=0
    
    grep -o "\[.*\](.*\.md)" "$doc" | while read -r link_line; do
        # Extract the URL part from the markdown link
        link_url=$(echo "$link_line" | sed -E 's/\[(.*)\]\((.*)\)/\2/g')
        
        # Skip if not an internal link to a markdown file
        if [[ ! "$link_url" == *".md"* ]]; then
            continue
        fi
        
        # Determine target file path
        local target_file=""
        if [[ "$link_url" == /* ]]; then
            # Absolute path (from repo root)
            target_file="$DOCS_DIR$link_url"
        elif [[ "$link_url" == ../* ]]; then
            # Relative path using parent directory
            target_file="$(dirname "$doc")/$link_url"
            target_file=$(realpath --relative-to="$(pwd)" "$target_file")
        else
            # Relative path in same directory
            target_file="$(dirname "$doc")/$link_url"
        fi
        
        # Check if target file exists
        if [[ -f "$target_file" ]]; then
            ((working_links++))
        else
            ((broken_links++))
            echo "  ✗ Broken link: $link_url -> $target_file" >> "$result_file"
        fi
    done
    
    if [[ $broken_links -eq 0 ]]; then
        echo "✓ All internal links are valid" >> "$result_file"
        return 0
    else
        echo "✗ Found $broken_links broken links" >> "$result_file"
        return 1
    fi
}

# Function to validate cross-references
validate_cross_references() {
    local doc="$1"
    local result_file="$2"
    
    # Check if document has a "related" section in frontmatter
    if ! grep -q "related:" "$doc"; then
        echo "✗ Missing related documents section in frontmatter" >> "$result_file"
        return 1
    fi
    
    # Count related documents
    local related_count=$(sed -n '/^related:/,/^[a-z]/p' "$doc" | grep -c "^  - ")
    
    if [[ $related_count -eq 0 ]]; then
        echo "✗ No related documents listed" >> "$result_file"
        return 1
    else
        echo "✓ Found $related_count related documents" >> "$result_file"
        return 0
    fi
}

# Function to validate a document
validate_document() {
    local doc="$1"
    local filename=$(basename "$doc" .md)
    local dirname=$(dirname "$doc")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    # Create directory for validation results
    local out_dir="$OUTPUT_DIR/${category}/${filename}"
    mkdir -p "$out_dir"
    
    local result_file="$out_dir/validation_results.txt"
    
    echo "Validating $doc..."
    echo "# Validation Results for $doc" > "$result_file"
    echo "Validation Date: $(date)" >> "$result_file"
    echo "" >> "$result_file"
    
    echo "## Frontmatter" >> "$result_file"
    validate_frontmatter "$doc" "$result_file"
    local frontmatter_status=$?
    
    echo "" >> "$result_file"
    echo "## Document Structure" >> "$result_file"
    validate_structure "$doc" "$result_file"
    local structure_status=$?
    
    echo "" >> "$result_file"
    echo "## Code Examples" >> "$result_file"
    validate_code_examples "$doc" "$result_file"
    local code_status=$?
    
    echo "" >> "$result_file"
    echo "## Internal Links" >> "$result_file"
    validate_links "$doc" "$result_file"
    local links_status=$?
    
    echo "" >> "$result_file"
    echo "## Cross-References" >> "$result_file"
    validate_cross_references "$doc" "$result_file"
    local refs_status=$?
    
    echo "" >> "$result_file"
    echo "## Summary" >> "$result_file"
    
    local total_issues=$((frontmatter_status + structure_status + code_status + links_status + refs_status))
    
    if [[ $total_issues -eq 0 ]]; then
        echo "✓ Document passed all validation checks" >> "$result_file"
        echo "Document passed all validation checks"
        return 0
    else
        echo "✗ Document has $total_issues areas with issues that need correction" >> "$result_file"
        echo "Document has $total_issues areas with issues that need correction"
        return 1
    fi
}

# Main execution
echo "===== Document Validator Tool ====="
echo "This tool implements the tiered document validation strategy"
echo "from the Phase 2 Completion Plan"
echo ""

# Get or create tier assignments
get_or_create_tier_assignments

# Calculate samples for each tier
tier_samples=$(calculate_samples "$OUTPUT_DIR/tier_assignments.csv")
read -r tier1_sample tier2_sample tier3_sample <<< "$tier_samples"

# Create tracking spreadsheet
TRACKING_FILE="$OUTPUT_DIR/validation_tracking.csv"
echo "Document Path,Category,Tier,Priority,Issues Found,Validation Status,Validation Date" > "$TRACKING_FILE"

# Select sample documents from each tier
echo "Selecting documents to validate..."

# Tier 1 - 100% validation
echo "Validating $tier1_sample Tier 1 documents..."
grep ",1," "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f1 | head -n "$tier1_sample" | while read -r doc; do
    [[ -z "$doc" ]] && continue
    
    validate_document "$doc"
    validation_status=$?
    
    if [[ $validation_status -eq 0 ]]; then
        status="Pass"
        issues=0
    else
        status="Fail"
        # Count issues in validation result
        result_file="$OUTPUT_DIR/$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')/$(basename "$doc" .md)/validation_results.txt"
        issues=$(grep -c "✗" "$result_file")
    fi
    
    category=$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    priority=$(grep "$doc" "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f4)
    
    echo "$doc,$category,1,$priority,$issues,$status,$(date '+%Y-%m-%d')" >> "$TRACKING_FILE"
done

# Tier 2 - 50% validation
echo "Validating $tier2_sample Tier 2 documents..."
grep ",2," "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f1 | sort -R | head -n "$tier2_sample" | while read -r doc; do
    [[ -z "$doc" ]] && continue
    
    validate_document "$doc"
    validation_status=$?
    
    if [[ $validation_status -eq 0 ]]; then
        status="Pass"
        issues=0
    else
        status="Fail"
        # Count issues in validation result
        result_file="$OUTPUT_DIR/$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')/$(basename "$doc" .md)/validation_results.txt"
        issues=$(grep -c "✗" "$result_file")
    fi
    
    category=$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    priority=$(grep "$doc" "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f4)
    
    echo "$doc,$category,2,$priority,$issues,$status,$(date '+%Y-%m-%d')" >> "$TRACKING_FILE"
done

# Tier 3 - Spot validation
echo "Validating $tier3_sample Tier 3 documents..."
grep ",3," "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f1 | sort -R | head -n "$tier3_sample" | while read -r doc; do
    [[ -z "$doc" ]] && continue
    
    validate_document "$doc"
    validation_status=$?
    
    if [[ $validation_status -eq 0 ]]; then
        status="Pass"
        issues=0
    else
        status="Fail"
        # Count issues in validation result
        result_file="$OUTPUT_DIR/$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')/$(basename "$doc" .md)/validation_results.txt"
        issues=$(grep -c "✗" "$result_file")
    fi
    
    category=$(echo "$doc" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    priority=$(grep "$doc" "$OUTPUT_DIR/tier_assignments.csv" | cut -d, -f4)
    
    echo "$doc,$category,3,$priority,$issues,$status,$(date '+%Y-%m-%d')" >> "$TRACKING_FILE"
done

# Generate summary
echo ""
echo "Document validation complete!"
echo "Results are stored in $OUTPUT_DIR"
echo "Validation tracking spreadsheet created at $TRACKING_FILE"

# Count passes and fails
pass_count=$(grep ",Pass," "$TRACKING_FILE" | wc -l | tr -d ' ')
fail_count=$(grep ",Fail," "$TRACKING_FILE" | wc -l | tr -d ' ')
total_count=$((pass_count + fail_count))

echo ""
echo "Summary:"
echo "- Total documents validated: $total_count"
echo "- Documents passing validation: $pass_count ($(( pass_count * 100 / total_count ))%)"
echo "- Documents failing validation: $fail_count ($(( fail_count * 100 / total_count ))%)"
echo ""
echo "Next steps:"
echo "1. Review the validation tracking spreadsheet"
echo "2. Fix issues in failing documents, starting with high-priority ones"
echo "3. Re-run validation to confirm fixes"
echo "4. Update the tier assignments if needed" 