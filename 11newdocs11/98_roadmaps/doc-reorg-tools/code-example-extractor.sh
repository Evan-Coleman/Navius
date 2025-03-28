#!/usr/bin/env bash

# code-example-extractor.sh
# Script to extract Rust code examples from markdown files
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/code-examples"
SAMPLE_SIZE=30  # Percentage of documents to sample

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Find all documents with Rust code examples
echo "Finding documents with Rust code examples..."
RUST_DOCS=$(find "$DOCS_DIR" -name "*.md" -exec grep -l "\`\`\`rust" {} \;)
TOTAL_DOCS=$(echo "$RUST_DOCS" | wc -l | tr -d ' ')
echo "Found $TOTAL_DOCS documents with Rust code examples"

# Categorize documents by section
echo "Categorizing documents by section..."
GETTING_STARTED=$(echo "$RUST_DOCS" | grep "01_getting_started" || echo "")
EXAMPLES=$(echo "$RUST_DOCS" | grep "02_examples" || echo "")
GUIDES=$(echo "$RUST_DOCS" | grep "04_guides" || echo "")
REFERENCE=$(echo "$RUST_DOCS" | grep "05_reference" || echo "")
OTHERS=$(echo "$RUST_DOCS" | grep -v "01_getting_started\|02_examples\|04_guides\|05_reference" || echo "")

# Calculate sample sizes for each category
GS_COUNT=$(echo "$GETTING_STARTED" | wc -l | tr -d ' ')
EX_COUNT=$(echo "$EXAMPLES" | wc -l | tr -d ' ')
GU_COUNT=$(echo "$GUIDES" | wc -l | tr -d ' ')
REF_COUNT=$(echo "$REFERENCE" | wc -l | tr -d ' ')
OTH_COUNT=$(echo "$OTHERS" | wc -l | tr -d ' ')

GS_SAMPLE=$((GS_COUNT * SAMPLE_SIZE / 100))
EX_SAMPLE=$((EX_COUNT * SAMPLE_SIZE / 100))
GU_SAMPLE=$((GU_COUNT * SAMPLE_SIZE / 100))
REF_SAMPLE=$((REF_COUNT * SAMPLE_SIZE / 100))
OTH_SAMPLE=$((OTH_COUNT * SAMPLE_SIZE / 100))

# Ensure at least one document from each category if available
[[ $GS_SAMPLE -eq 0 && $GS_COUNT -gt 0 ]] && GS_SAMPLE=1
[[ $EX_SAMPLE -eq 0 && $EX_COUNT -gt 0 ]] && EX_SAMPLE=1
[[ $GU_SAMPLE -eq 0 && $GU_COUNT -gt 0 ]] && GU_SAMPLE=1
[[ $REF_SAMPLE -eq 0 && $REF_COUNT -gt 0 ]] && REF_SAMPLE=1
[[ $OTH_SAMPLE -eq 0 && $OTH_COUNT -gt 0 ]] && OTH_SAMPLE=1

echo "Sample sizes by category:"
echo "- Getting Started: $GS_SAMPLE of $GS_COUNT"
echo "- Examples: $EX_SAMPLE of $EX_COUNT"
echo "- Guides: $GU_SAMPLE of $GU_COUNT"
echo "- Reference: $REF_SAMPLE of $REF_COUNT"
echo "- Others: $OTH_SAMPLE of $OTH_COUNT"

# Function to randomly select documents from a category
select_sample() {
    local docs="$1"
    local count="$2"
    local category="$3"
    
    if [[ -z "$docs" || "$count" -eq 0 ]]; then
        return
    fi
    
    echo "Selecting $count documents from $category..."
    echo "$docs" | sort -R | head -n "$count"
}

# Select sample documents from each category
GS_SAMPLE_DOCS=$(select_sample "$GETTING_STARTED" "$GS_SAMPLE" "Getting Started")
EX_SAMPLE_DOCS=$(select_sample "$EXAMPLES" "$EX_SAMPLE" "Examples")
GU_SAMPLE_DOCS=$(select_sample "$GUIDES" "$GU_SAMPLE" "Guides")
REF_SAMPLE_DOCS=$(select_sample "$REFERENCE" "$REF_SAMPLE" "Reference")
OTH_SAMPLE_DOCS=$(select_sample "$OTHERS" "$OTH_SAMPLE" "Others")

# Combine all sample documents
SAMPLE_DOCS=$(echo -e "$GS_SAMPLE_DOCS\n$EX_SAMPLE_DOCS\n$GU_SAMPLE_DOCS\n$REF_SAMPLE_DOCS\n$OTH_SAMPLE_DOCS" | grep -v "^$")
SAMPLE_COUNT=$(echo "$SAMPLE_DOCS" | wc -l | tr -d ' ')

echo "Selected $SAMPLE_COUNT documents for code example extraction"

# Create tracking spreadsheet
TRACKING_FILE="$OUTPUT_DIR/verification_tracking.csv"
echo "Document Path,Category,Examples Count,Extracted File,Verification Status,Issues,Notes" > "$TRACKING_FILE"

# Extract code examples from each document
process_document() {
    local doc="$1"
    local filename=$(basename "$doc" .md)
    local dirname=$(dirname "$doc")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    # Create directory for document
    local out_dir="$OUTPUT_DIR/${category}/${filename}"
    mkdir -p "$out_dir"
    
    # Extract all Rust code examples
    local example_count=0
    local current_example=1
    
    echo "Processing $doc..."
    
    # Use awk to extract code blocks
    awk '/```rust/{flag=1; buf=""; next} /```/{if (flag) {print buf > "'$out_dir'/example_'$current_example'.rs"; current_example++; flag=0}} flag {buf = buf $0 "\n"}' "$doc"
    
    # Count examples
    example_count=$((current_example - 1))
    
    # Add to tracking spreadsheet
    echo "$doc,$category,$example_count,$out_dir,Pending,," >> "$TRACKING_FILE"
    
    echo "Extracted $example_count Rust code examples from $doc"
    
    # Create a consolidated file with all examples
    if [[ $example_count -gt 0 ]]; then
        local all_examples="$out_dir/all_examples.rs"
        echo "// Consolidated examples from $doc" > "$all_examples"
        echo "// Number of examples: $example_count" >> "$all_examples"
        echo "" >> "$all_examples"
        
        for (( i=1; i<=example_count; i++ )); do
            echo "// Example $i" >> "$all_examples"
            echo "#[allow(unused_variables, dead_code)]" >> "$all_examples"
            echo "fn example_${i}() {" >> "$all_examples"
            cat "$out_dir/example_$i.rs" >> "$all_examples"
            echo "}" >> "$all_examples"
            echo "" >> "$all_examples"
        done
        
        echo "fn main() {" >> "$all_examples"
        echo "    // This is a placeholder main function" >> "$all_examples"
        echo "    println!(\"Examples from $filename\");" >> "$all_examples"
        echo "}" >> "$all_examples"
    fi
}

# Process each document in the sample
echo "Extracting code examples..."
echo "$SAMPLE_DOCS" | while read -r doc; do
    [[ -z "$doc" ]] && continue
    process_document "$doc"
done

echo "Code example extraction complete!"
echo "Extracted examples are stored in $OUTPUT_DIR"
echo "Tracking spreadsheet created at $TRACKING_FILE"
echo ""
echo "Next steps:"
echo "1. Review extracted examples"
echo "2. Attempt compilation with: rustc -A unused_variables -A dead_code [example_file]"
echo "3. Update tracking spreadsheet with results"
echo "4. Create templates for common issues" 