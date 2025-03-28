#!/usr/bin/env bash

# code-example-extractor.sh
# Script to extract Rust code examples from markdown files
# Part of the Phase 2 Completion Plan implementation

set -e

# Configuration
DOCS_DIR="11newdocs11"
OUTPUT_DIR="target/code-verification/examples"
SAMPLE_SIZE=30  # Percentage of documents to sample

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Function to process a single document
process_document() {
    local doc="$1"
    
    if [[ ! -f "$doc" ]]; then
        echo "Error: Document $doc not found."
        return 1
    fi
    
    local filename=$(basename "$doc" .md)
    local dirname=$(dirname "$doc")
    local category=$(echo "$dirname" | grep -o "[0-9]*_[^/]*" | head -1 | sed 's/^[0-9]*_//')
    
    if [[ -z "$category" ]]; then
        category="misc"
    fi
    
    # Create directory for document
    local out_dir="$OUTPUT_DIR/${category}/${filename}"
    mkdir -p "$out_dir"
    
    # Create examples tracking file for the document
    local tracking_file="$out_dir/examples.csv"
    echo "Example,StartLine,EndLine,File,Issues,Status" > "$tracking_file"
    
    echo "Processing $doc..."
    
    # Extract all Rust code examples with line numbers
    local line_num=1
    local in_code_block=0
    local start_line=0
    local example_count=0
    local block_content=""
    
    while IFS= read -r line; do
        if [[ "$line" =~ ^"\`\`\`rust"$ ]]; then
            in_code_block=1
            start_line=$line_num
            block_content=""
        elif [[ "$line" =~ ^"\`\`\`"$ && $in_code_block -eq 1 ]]; then
            in_code_block=0
            example_count=$((example_count + 1))
            local example_file="$out_dir/example_${example_count}.rs"
            
            # Write content to file
            echo "// Example ${example_count} from ${filename}.md" > "$example_file"
            echo "// DEFAULT_IMPORTS" >> "$example_file"
            echo -e "$block_content" >> "$example_file"
            
            # Test compilation
            local status="Compiles"
            local issues=""
            
            # Check for common issues
            if ! grep -q "^use " "$example_file"; then
                [[ -z "$issues" ]] && issues="Missing imports" || issues="$issues; Missing imports"
            fi
            
            if ! grep -q "fn " "$example_file"; then
                [[ -z "$issues" ]] && issues="Code fragment without declarations" || issues="$issues; Code fragment without declarations"
            fi
            
            if grep -q "\.unwrap()" "$example_file" || grep -q "\.expect(" "$example_file"; then
                [[ -z "$issues" ]] && issues="Uses unwrap or expect" || issues="$issues; Uses unwrap or expect"
            fi
            
            # Try to compile
            if ! rustc --edition=2021 -o /dev/null "$example_file" 2>/dev/null; then
                status="Fails"
                if [[ -z "$issues" ]]; then
                    issues="Compilation error"
                fi
            fi
            
            # Record in tracking file
            echo "${example_count},${start_line},$((line_num)),\"${example_file}\",\"${issues}\",\"${status}\"" >> "$tracking_file"
        elif [[ $in_code_block -eq 1 ]]; then
            block_content="${block_content}${line}\n"
        fi
        
        line_num=$((line_num + 1))
    done < "$doc"
    
    echo "Extracted $example_count Rust code examples from $doc"
    
    # Create a consolidated file with all examples
    if [[ $example_count -gt 0 ]]; then
        local all_examples="$out_dir/all_examples.rs"
        echo "// Consolidated examples from $doc" > "$all_examples"
        echo "// Number of examples: $example_count" >> "$all_examples"
        echo "" >> "$all_examples"
        
        for (( i=1; i<=example_count; i++ )); do
            if [[ -f "$out_dir/example_$i.rs" ]]; then
                echo "// Example $i" >> "$all_examples"
                echo "#[allow(unused_variables, dead_code)]" >> "$all_examples"
                echo "fn example_${i}() {" >> "$all_examples"
                # Skip the first comment line when including in consolidated file
                tail -n +3 "$out_dir/example_$i.rs" >> "$all_examples"
                echo "}" >> "$all_examples"
                echo "" >> "$all_examples"
            fi
        done
        
        echo "fn main() {" >> "$all_examples"
        echo "    // This is a placeholder main function" >> "$all_examples"
        echo "    println!(\"Examples from $filename\");" >> "$all_examples"
        echo "}" >> "$all_examples"
    fi
    
    return 0
}

# Function to process all documents through sampling
process_all_documents() {
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
    
    # Process each document in the sample
    echo "Extracting code examples..."
    echo "$SAMPLE_DOCS" | while read -r doc; do
        [[ -z "$doc" ]] && continue
        process_document "$doc"
    done
}

# Function to print usage information
print_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --all                Process all documents based on sampling"
    echo "  --document <path>    Process a specific document"
    echo "  --category <name>    Process all documents in a category"
    echo "  --help               Display this help message"
    echo ""
    echo "Examples:"
    echo "  $0 --all                    # Process documents based on sampling"
    echo "  $0 --document 11newdocs11/05_reference/api/router-api.md   # Process specific document"
    echo "  $0 --category 05_reference  # Process documents in the reference category"
}

# Main execution
echo "===== Code Example Extractor Tool ====="
echo "This tool extracts Rust code examples from markdown files"
echo "Part of the Phase 2 Completion Plan implementation"
echo ""

# Check if any arguments were provided
if [[ $# -eq 0 ]]; then
    # No arguments - print help
    print_usage
    exit 0
fi

# Process arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --all)
            process_all_documents
            ;;
        
        --document)
            if [[ -z "$2" || ! -f "$2" ]]; then
                echo "Error: Document '$2' not found."
                echo "Please provide a valid document path."
                exit 1
            fi
            process_document "$2"
            shift
            ;;
        
        --category)
            if [[ -z "$2" ]]; then
                echo "Error: Category name not provided."
                echo "Please provide a valid category name."
                exit 1
            fi
            
            echo "Processing all documents in category $2..."
            find "$DOCS_DIR" -path "*$2*" -name "*.md" -exec grep -l "\`\`\`rust" {} \; | while read -r doc; do
                process_document "$doc"
            done
            shift
            ;;
        
        --help)
            print_usage
            exit 0
            ;;
        
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
    shift
done

echo ""
echo "Code example extraction complete!"
echo "Extracted examples are stored in $OUTPUT_DIR"
echo ""
echo "Next steps:"
echo "1. Review extracted examples"
echo "2. Check compilation results in tracking files"
echo "3. Use code-example-fixer.sh to fix common issues"
echo "4. Use code-example-verifier.sh for detailed verification" 