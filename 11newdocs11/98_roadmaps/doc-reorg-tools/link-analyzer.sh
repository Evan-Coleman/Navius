#!/usr/bin/env bash

# link-analyzer.sh
# Script to analyze and validate internal links in markdown documents
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/link-analysis"
SAMPLE_SIZE=30  # Percentage of documents to sample for detailed analysis

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Find all documents with internal links
echo "Finding documents with internal links..."
LINK_DOCS=$(find "$DOCS_DIR" -name "*.md" -exec grep -l "\[.*\](.*\.md)" {} \;)
TOTAL_DOCS=$(echo "$LINK_DOCS" | wc -l | tr -d ' ')
echo "Found $TOTAL_DOCS documents with internal markdown links"

# Categorize documents by section
echo "Categorizing documents by section..."
GETTING_STARTED=$(echo "$LINK_DOCS" | grep "01_getting_started" || echo "")
EXAMPLES=$(echo "$LINK_DOCS" | grep "02_examples" || echo "")
GUIDES=$(echo "$LINK_DOCS" | grep "04_guides" || echo "")
REFERENCE=$(echo "$LINK_DOCS" | grep "05_reference" || echo "")
OTHERS=$(echo "$LINK_DOCS" | grep -v "01_getting_started\|02_examples\|04_guides\|05_reference" || echo "")

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

echo "Selected $SAMPLE_COUNT documents for link analysis"

# Create tracking spreadsheet
TRACKING_FILE="$OUTPUT_DIR/link_tracking.csv"
echo "Document Path,Category,Links Count,Broken Links,Valid Links,Link Types,Notes" > "$TRACKING_FILE"

# Function to extract and validate links
extract_and_validate_links() {
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
    
    echo "Processing $doc..."
    
    # Extract all links
    local links_file="$out_dir/all_links.txt"
    grep -o "\[.*\](.*\.md)" "$doc" > "$links_file" || echo "No standard markdown links found" > "$links_file"
    
    # Count links
    local links_count=$(grep -c "\.md" "$links_file" || echo 0)
    
    # Analyze link types
    local same_dir_links=$(grep -c "\[.*\]([^/].*\.md)" "$links_file" || echo 0)
    local parent_dir_links=$(grep -c "\[.*\](\.\./" "$links_file" || echo 0)
    local absolute_links=$(grep -c "\[.*\](/.*\.md)" "$links_file" || echo 0)
    local link_types="$same_dir_links same dir, $parent_dir_links parent dir, $absolute_links absolute"
    
    # Validate links
    local valid_links=0
    local broken_links=0
    local broken_links_list=""
    
    # Process each extracted link
    while IFS= read -r link_line; do
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
            ((valid_links++))
            echo "✓ Valid: $link_url -> $target_file" >> "$out_dir/link_validation.txt"
        else
            ((broken_links++))
            echo "✗ Broken: $link_url -> $target_file" >> "$out_dir/link_validation.txt"
            broken_links_list="$broken_links_list$link_url; "
        fi
    done < "$links_file"
    
    # Add to tracking spreadsheet
    echo "$doc,$category,$links_count,$broken_links,$valid_links,\"$link_types\",\"$broken_links_list\"" >> "$TRACKING_FILE"
    
    echo "Found $links_count links in $doc ($broken_links broken, $valid_links valid)"
}

# Process each document in the sample
echo "Analyzing links..."
echo "$SAMPLE_DOCS" | while read -r doc; do
    [[ -z "$doc" ]] && continue
    extract_and_validate_links "$doc"
done

# Create a critical paths file to test key documentation journeys
CRITICAL_PATHS_FILE="$OUTPUT_DIR/critical_paths.csv"
echo "Starting Document,Target Document,Path,Status,Notes" > "$CRITICAL_PATHS_FILE"

# Define critical documentation paths to verify
cat << EOF >> "$CRITICAL_PATHS_FILE"
11newdocs11/01_getting_started/installation.md,11newdocs11/01_getting_started/hello-world.md,Getting Started flow,Not Validated,
11newdocs11/01_getting_started/hello-world.md,11newdocs11/02_examples/basic/rest-api.md,Hello World to Examples,Not Validated,
11newdocs11/01_getting_started/installation.md,11newdocs11/04_guides/deployment/production-deployment.md,Installation to Deployment,Not Validated,
11newdocs11/02_examples/basic/rest-api.md,11newdocs11/05_reference/api/router-api.md,Example to API Reference,Not Validated,
11newdocs11/04_guides/middleware/error-handling.md,11newdocs11/05_reference/api/error-api.md,Guide to Reference,Not Validated,
EOF

echo "Link analysis complete!"
echo "Results are stored in $OUTPUT_DIR"
echo "Link tracking spreadsheet created at $TRACKING_FILE"
echo "Critical paths file created at $CRITICAL_PATHS_FILE"
echo ""
echo "Next steps:"
echo "1. Review the link tracking spreadsheet"
echo "2. Fix broken links in high-priority documents"
echo "3. Validate critical documentation paths"
echo "4. Update linked documents for consistency" 