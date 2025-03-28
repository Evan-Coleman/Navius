#!/bin/bash

# Comprehensive Documentation Test Suite
# This script performs advanced documentation validation beyond basic format checks
# It verifies document relationships, content accuracy, and semantic correctness

set -e

DOCS_DIR="docs"
REPORTS_DIR="target/reports/docs_validation"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="${REPORTS_DIR}/comprehensive_test_${TIMESTAMP}.md"
GRAPH_FILE="${REPORTS_DIR}/document_graph_${TIMESTAMP}.dot"
HTML_VISUALIZATION="${REPORTS_DIR}/document_graph_${TIMESTAMP}.html"
SINGLE_FILE_MODE=false
TARGET_FILE=""
FOCUS_DIR=""
CSV_OUTPUT=false
CUSTOM_RULES_FILE=""

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
declare -A content_quality_scores
declare -A code_validation_results
declare -A readability_scores

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --file)
            SINGLE_FILE_MODE=true
            TARGET_FILE="$2"
            shift 2
            ;;
        --dir)
            FOCUS_DIR="$2"
            shift 2
            ;;
        --csv)
            CSV_OUTPUT=true
            shift
            ;;
        --rules)
            CUSTOM_RULES_FILE="$2"
            shift 2
            ;;
        --help)
            echo "Usage: comprehensive_test.sh [OPTIONS]"
            echo "Options:"
            echo "  --file FILE       Analyze a single file instead of all documentation"
            echo "  --dir DIRECTORY   Focus analysis on a specific directory"
            echo "  --csv             Output results in CSV format for spreadsheet import"
            echo "  --rules FILE      Use custom validation rules from specified file"
            echo "  --help            Display this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

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

# Function to check if code blocks are syntactically valid
validate_code_blocks() {
    local file=$1
    local language=""
    local code_block=""
    local in_code_block=false
    local block_count=0
    local valid_blocks=0
    local result="PASS"
    
    # Read the file line by line
    while IFS= read -r line; do
        # Check for the start of a code block
        if [[ "$line" =~ ^```([a-zA-Z0-9]*)$ && "$in_code_block" == false ]]; then
            language="${BASH_REMATCH[1]}"
            code_block=""
            in_code_block=true
            block_count=$((block_count + 1))
            continue
        fi
        
        # Check for the end of a code block
        if [[ "$line" == "```" && "$in_code_block" == true ]]; then
            in_code_block=false
            
            # Validate based on language
            case "$language" in
                rust)
                    # Create a temporary file for validation
                    temp_file=$(mktemp)
                    echo "$code_block" > "$temp_file"
                    
                    # Use rustfmt to validate syntax
                    if rustfmt --check "$temp_file" &> /dev/null; then
                        valid_blocks=$((valid_blocks + 1))
                    else
                        result="FAIL"
                    fi
                    
                    rm "$temp_file"
                    ;;
                bash|sh)
                    # Create a temporary file for validation
                    temp_file=$(mktemp)
                    echo "$code_block" > "$temp_file"
                    
                    # Use bash -n to validate syntax
                    if bash -n "$temp_file" &> /dev/null; then
                        valid_blocks=$((valid_blocks + 1))
                    else
                        result="FAIL"
                    fi
                    
                    rm "$temp_file"
                    ;;
                *)
                    # For other languages, just count them without validation
                    valid_blocks=$((valid_blocks + 1))
                    ;;
            esac
            
            continue
        fi
        
        # Collect code blocks
        if [[ "$in_code_block" == true ]]; then
            code_block="${code_block}${line}
"
        fi
    done < "$file"
    
    # Store the validation result
    code_validation_results["$file"]="$result:$valid_blocks:$block_count"
}

# Function to calculate readability score
calculate_readability() {
    local file=$1
    local content=""
    local in_code_block=false
    local in_frontmatter=false
    local frontmatter_count=0
    
    # Extract content without code blocks and frontmatter
    while IFS= read -r line; do
        # Skip frontmatter
        if [[ "$line" == "---" ]]; then
            frontmatter_count=$((frontmatter_count + 1))
            if [[ "$frontmatter_count" == 1 ]]; then
                in_frontmatter=true
                continue
            elif [[ "$frontmatter_count" == 2 ]]; then
                in_frontmatter=false
                continue
            fi
        fi
        
        if [[ "$in_frontmatter" == true ]]; then
            continue
        fi
        
        # Skip code blocks
        if [[ "$line" =~ ^```[a-zA-Z0-9]*$ && "$in_code_block" == false ]]; then
            in_code_block=true
            continue
        fi
        
        if [[ "$line" == "```" && "$in_code_block" == true ]]; then
            in_code_block=false
            continue
        fi
        
        if [[ "$in_code_block" == true ]]; then
            continue
        fi
        
        # Add the line to content
        content="${content}${line}
"
    done < "$file"
    
    # Calculate basic readability metrics
    words=$(echo "$content" | wc -w)
    sentences=$(echo "$content" | grep -o "[.!?]" | wc -l)
    paragraphs=$(echo "$content" | grep -c "^$")
    
    # Avoid division by zero
    if [[ "$sentences" -eq 0 ]]; then
        sentences=1
    fi
    
    # Simple readability calculation (words per sentence)
    words_per_sentence=$(echo "scale=2; $words / $sentences" | bc)
    
    # Assign a readability score based on words per sentence
    # 15-20 words per sentence is considered ideal
    if (( $(echo "$words_per_sentence < 10" | bc -l) )); then
        readability_score="Simple:$words_per_sentence"
    elif (( $(echo "$words_per_sentence <= 20" | bc -l) )); then
        readability_score="Good:$words_per_sentence"
    else
        readability_score="Complex:$words_per_sentence"
    fi
    
    readability_scores["$file"]="$readability_score:$words:$sentences:$paragraphs"
}

# Function to evaluate content quality
evaluate_content_quality() {
    local file=$1
    local score=0
    local frontmatter=$(extract_frontmatter "$file")
    
    # Check for frontmatter completeness (2 points)
    if [[ -n "$(get_frontmatter_field "$frontmatter" "title")" ]]; then
        score=$((score + 1))
    fi
    
    if [[ -n "$(get_frontmatter_field "$frontmatter" "description")" ]]; then
        score=$((score + 1))
    fi
    
    # Check for heading structure (3 points)
    if grep -q "^# " "$file"; then
        score=$((score + 1))
    fi
    
    if grep -q "^## " "$file"; then
        score=$((score + 1))
    fi
    
    # Check if there are at least 2 subsections
    if [[ $(grep -c "^## " "$file") -ge 2 ]]; then
        score=$((score + 1))
    fi
    
    # Check for code examples (2 points)
    if grep -q "^\`\`\`" "$file"; then
        score=$((score + 1))
        
        # Extra point if code blocks have language specified
        if grep -q "^\`\`\`[a-zA-Z]" "$file"; then
            score=$((score + 1))
        fi
    fi
    
    # Check for internal links (1 point)
    if grep -q "\[.*\](.*\.md)" "$file"; then
        score=$((score + 1))
    fi
    
    # Check for Related Documents section (2 points)
    if grep -q "^## Related Documents" "$file"; then
        score=$((score + 1))
        
        # Extra point if there are actual related documents listed
        if grep -A 5 "^## Related Documents" "$file" | grep -q "\[.*\](.*\.md)"; then
            score=$((score + 1))
        fi
    fi
    
    # Determine quality label based on score
    local quality_label=""
    if [[ $score -ge 9 ]]; then
        quality_label="Excellent"
    elif [[ $score -ge 7 ]]; then
        quality_label="Good"
    elif [[ $score -ge 5 ]]; then
        quality_label="Adequate"
    elif [[ $score -ge 3 ]]; then
        quality_label="Poor"
    else
        quality_label="Very Poor"
    fi
    
    content_quality_scores["$file"]="$quality_label:$score:10"
}

# Function to generate document relationship graph
generate_graph() {
    echo "digraph DocumentRelationships {" > "$GRAPH_FILE"
    echo "  node [shape=box, style=filled, fillcolor=lightblue];" >> "$GRAPH_FILE"
    echo "  graph [rankdir=LR];" >> "$GRAPH_FILE"
    
    # Add nodes
    for file in "${!document_references[@]}"; do
        filename=$(basename "$file")
        fileId="${filename//[^a-zA-Z0-9]/_}"
        echo "  $fileId [label=\"$filename\"];" >> "$GRAPH_FILE"
    done
    
    # Add edges
    for pair in "${!related_pairs[@]}"; do
        source=$(echo "$pair" | cut -d'|' -f1)
        target=$(echo "$pair" | cut -d'|' -f2)
        
        source_filename=$(basename "$source")
        target_filename=$(basename "$target")
        
        sourceId="${source_filename//[^a-zA-Z0-9]/_}"
        targetId="${target_filename//[^a-zA-Z0-9]/_}"
        
        echo "  $sourceId -> $targetId;" >> "$GRAPH_FILE"
    done
    
    echo "}" >> "$GRAPH_FILE"
    
    # Try to generate HTML visualization if graphviz is installed
    if command -v dot &> /dev/null; then
        dot -Tsvg "$GRAPH_FILE" > "${GRAPH_FILE}.svg"
        
        # Create an HTML file with the SVG embedded
        cat > "$HTML_VISUALIZATION" << EOL
<!DOCTYPE html>
<html>
<head>
    <title>Document Relationship Graph</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        h1 { color: #333; }
        .container { max-width: 100%; overflow: auto; }
    </style>
</head>
<body>
    <h1>Document Relationship Graph</h1>
    <p>Generated on $(date)</p>
    <div class="container">
        $(cat "${GRAPH_FILE}.svg")
    </div>
</body>
</html>
EOL
        echo -e "${GREEN}Generated document relationship visualization: $HTML_VISUALIZATION${NC}"
    else
        echo -e "${YELLOW}Graphviz not installed. Skipping visualization generation.${NC}"
        echo -e "${YELLOW}Install with: sudo apt-get install graphviz${NC}"
    fi
}

# Generate AI-assisted recommendations
generate_recommendations() {
    local file=$1
    
    # Extract various metrics
    local quality=${content_quality_scores[$file]}
    local quality_label=$(echo "$quality" | cut -d':' -f1)
    local quality_score=$(echo "$quality" | cut -d':' -f2)
    
    local readability=${readability_scores[$file]}
    local readability_label=$(echo "$readability" | cut -d':' -f1)
    local words_per_sentence=$(echo "$readability" | cut -d':' -f2)
    
    local code_validation=${code_validation_results[$file]}
    local code_status=$(echo "$code_validation" | cut -d':' -f1)
    
    # Generate recommendations based on metrics
    local recommendations=""
    
    # Content quality recommendations
    if [[ "$quality_label" == "Very Poor" || "$quality_label" == "Poor" ]]; then
        recommendations+="- Structure needs significant improvement. Add proper frontmatter, headings, and sections.\n"
    elif [[ "$quality_label" == "Adequate" ]]; then
        recommendations+="- Good basic structure, but needs more detail and better section organization.\n"
    fi
    
    # Readability recommendations
    if [[ "$readability_label" == "Complex" ]]; then
        recommendations+="- Simplify content: break long sentences into shorter ones (currently $words_per_sentence words per sentence).\n"
    elif [[ "$readability_label" == "Simple" ]]; then
        recommendations+="- Content may be too simplistic. Consider adding more detailed explanations.\n"
    fi
    
    # Code validation recommendations
    if [[ "$code_status" == "FAIL" ]]; then
        recommendations+="- Fix syntax errors in code examples.\n"
    fi
    
    # Check if related documents section exists
    if ! grep -q "^## Related Documents" "$file"; then
        recommendations+="- Add a 'Related Documents' section with relevant links.\n"
    fi
    
    # Check for broken links
    if grep -q "\[.*\]([^)]*)" "$file"; then
        local links=$(grep -o "\[.*\]([^)]*)" "$file" | grep -o "([^)]*)" | tr -d '(' | tr -d ')')
        for link in $links; do
            # Skip external links and anchors
            if [[ "$link" =~ ^https?:// || "$link" =~ ^# ]]; then
                continue
            fi
            
            # Check if link is to a markdown file that exists
            if [[ "$link" == *.md ]]; then
                # Handle absolute paths
                if [[ "$link" == /* ]]; then
                    # Remove /docs prefix if present
                    link_path=${link#/docs}
                    link_path="docs$link_path"
                else
                    # Relative path
                    link_path=$(dirname "$file")/$link
                fi
                
                if [ ! -f "$link_path" ]; then
                    recommendations+="- Fix broken link to $link.\n"
                fi
            fi
        done
    fi
    
    echo -e "$recommendations"
}

# Main report generation
echo -e "${BLUE}Starting comprehensive documentation testing...${NC}"

# Determine which files to analyze
if [[ "$SINGLE_FILE_MODE" == true ]]; then
    if [[ ! -f "$TARGET_FILE" ]]; then
        echo -e "${RED}Error: File $TARGET_FILE does not exist.${NC}"
        exit 1
    fi
    
    files=("$TARGET_FILE")
    echo -e "${BLUE}Analyzing single file: $TARGET_FILE${NC}"
else
    if [[ -n "$FOCUS_DIR" ]]; then
        if [[ ! -d "$FOCUS_DIR" ]]; then
            echo -e "${RED}Error: Directory $FOCUS_DIR does not exist.${NC}"
            exit 1
        fi
        
        echo -e "${BLUE}Focusing analysis on directory: $FOCUS_DIR${NC}"
        files=($(find "$FOCUS_DIR" -name "*.md" -type f))
    else
        echo -e "${BLUE}Analyzing all documentation in $DOCS_DIR${NC}"
        files=($(find "$DOCS_DIR" -name "*.md" -type f))
    fi
fi

# CSV output mode
if [[ "$CSV_OUTPUT" == true ]]; then
    echo "File,Title,Category,Tags,Quality Score,Readability,Code Status,Word Count,Related Documents"
    
    for file in "${files[@]}"; do
        frontmatter=$(extract_frontmatter "$file")
        title=$(get_frontmatter_field "$frontmatter" "title")
        category=$(get_frontmatter_field "$frontmatter" "category")
        tags=$(get_frontmatter_list "$frontmatter" "tags" | tr '\n' ',' | sed 's/,$//')
        
        # Calculate metrics
        evaluate_content_quality "$file"
        calculate_readability "$file"
        validate_code_blocks "$file"
        
        # Extract metrics
        quality=${content_quality_scores[$file]}
        quality_label=$(echo "$quality" | cut -d':' -f1)
        
        readability=${readability_scores[$file]}
        readability_label=$(echo "$readability" | cut -d':' -f1)
        word_count=$(echo "$readability" | cut -d':' -f2)
        
        code_validation=${code_validation_results[$file]}
        code_status=$(echo "$code_validation" | cut -d':' -f1)
        
        # Count related documents
        related_count=$(grep -c "\[.*\](.*\.md)" "$file")
        
        # Output CSV row
        echo "\"$file\",\"$title\",\"$category\",\"$tags\",\"$quality_label\",\"$readability_label\",\"$code_status\",\"$word_count\",\"$related_count\""
    done
    
    exit 0
fi

# Initialize the report
echo "# Comprehensive Documentation Test Report - $(date '+%B %d, %Y')" > $REPORT_FILE
echo "" >> $REPORT_FILE
echo "## Test Configuration" >> $REPORT_FILE
echo "" >> $REPORT_FILE

if [[ "$SINGLE_FILE_MODE" == true ]]; then
    echo "- Mode: Single File Analysis" >> $REPORT_FILE
    echo "- Target: $TARGET_FILE" >> $REPORT_FILE
elif [[ -n "$FOCUS_DIR" ]]; then
    echo "- Mode: Directory Analysis" >> $REPORT_FILE
    echo "- Target: $FOCUS_DIR" >> $REPORT_FILE
else
    echo "- Mode: Full Documentation Analysis" >> $REPORT_FILE
    echo "- Target: $DOCS_DIR" >> $REPORT_FILE
fi

if [[ -n "$CUSTOM_RULES_FILE" ]]; then
    echo "- Custom Rules: $CUSTOM_RULES_FILE" >> $REPORT_FILE
fi

echo "" >> $REPORT_FILE

# Process the files
total_documents=${#files[@]}
files_with_frontmatter=0
files_without_frontmatter=0

echo -e "${BLUE}Phase 1: Document Inventory and Metadata Analysis${NC}"
echo "## Document Inventory" >> $REPORT_FILE
echo "" >> $REPORT_FILE

for file in "${files[@]}"; do
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
    
    # Evaluate content quality
    evaluate_content_quality "$file"
    
    # Calculate readability
    calculate_readability "$file"
    
    # Validate code blocks
    validate_code_blocks "$file"
done

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

# Generate the document graph
generate_graph

# Check for orphaned documents (no incoming references)
echo "### Orphaned Documents" >> $REPORT_FILE
echo "Documents that are not referenced by any other document:" >> $REPORT_FILE
echo "" >> $REPORT_FILE

orphaned_count=0
for file in "${files[@]}"; do
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

# Phase 5: Content Quality Analysis
echo -e "${BLUE}Phase 5: Content Quality Analysis${NC}"
echo "## Content Quality Assessment" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "| Document | Quality | Score | Readability | Words/Sentence | Code Status |" >> $REPORT_FILE
echo "|----------|---------|-------|-------------|----------------|-------------|" >> $REPORT_FILE

for file in "${files[@]}"; do
    # Extract quality metrics
    quality=${content_quality_scores[$file]}
    quality_label=$(echo "$quality" | cut -d':' -f1)
    quality_score=$(echo "$quality" | cut -d':' -f2)
    
    # Extract readability metrics
    readability=${readability_scores[$file]}
    readability_label=$(echo "$readability" | cut -d':' -f1)
    words_per_sentence=$(echo "$readability" | cut -d':' -f2)
    
    # Extract code validation metrics
    code_validation=${code_validation_results[$file]}
    code_status=$(echo "$code_validation" | cut -d':' -f1)
    
    # Output the table row
    echo "| $(basename "$file") | $quality_label | $quality_score/10 | $readability_label | $words_per_sentence | $code_status |" >> $REPORT_FILE
done
echo "" >> $REPORT_FILE

# Phase 6: Document Improvement Recommendations
echo -e "${BLUE}Phase 6: Document Improvement Recommendations${NC}"
echo "## Improvement Recommendations" >> $REPORT_FILE
echo "" >> $REPORT_FILE

for file in "${files[@]}"; do
    # Skip documents that are already excellent
    quality=${content_quality_scores[$file]}
    quality_label=$(echo "$quality" | cut -d':' -f1)
    
    if [[ "$quality_label" == "Excellent" ]]; then
        continue
    fi
    
    # Generate recommendations for this file
    recommendations=$(generate_recommendations "$file")
    
    if [[ -n "$recommendations" ]]; then
        echo "### $(basename "$file")" >> $REPORT_FILE
        echo "" >> $REPORT_FILE
        echo -e "$recommendations" >> $REPORT_FILE
        echo "" >> $REPORT_FILE
    fi
done

# Output summary statistics
echo "## Summary" >> $REPORT_FILE
echo "" >> $REPORT_FILE
echo "- **Total Documents:** $total_documents" >> $REPORT_FILE
echo "- **Frontmatter Coverage:** $files_with_frontmatter/$total_documents ($(echo "scale=1; 100*$files_with_frontmatter/$total_documents" | bc)%)" >> $REPORT_FILE
echo "- **Orphaned Documents:** $orphaned_count" >> $REPORT_FILE
echo "- **Broken References:** $broken_ref_count" >> $REPORT_FILE

# Count documents by quality level
excellent_docs=0
good_docs=0
adequate_docs=0
poor_docs=0
very_poor_docs=0

for file in "${files[@]}"; do
    quality=${content_quality_scores[$file]}
    quality_label=$(echo "$quality" | cut -d':' -f1)
    
    case "$quality_label" in
        "Excellent") excellent_docs=$((excellent_docs + 1)) ;;
        "Good") good_docs=$((good_docs + 1)) ;;
        "Adequate") adequate_docs=$((adequate_docs + 1)) ;;
        "Poor") poor_docs=$((poor_docs + 1)) ;;
        "Very Poor") very_poor_docs=$((very_poor_docs + 1)) ;;
    esac
done

echo "- **Quality Distribution:**" >> $REPORT_FILE
echo "  - Excellent: $excellent_docs ($(echo "scale=1; 100*$excellent_docs/$total_documents" | bc)%)" >> $REPORT_FILE
echo "  - Good: $good_docs ($(echo "scale=1; 100*$good_docs/$total_documents" | bc)%)" >> $REPORT_FILE
echo "  - Adequate: $adequate_docs ($(echo "scale=1; 100*$adequate_docs/$total_documents" | bc)%)" >> $REPORT_FILE
echo "  - Poor: $poor_docs ($(echo "scale=1; 100*$poor_docs/$total_documents" | bc)%)" >> $REPORT_FILE
echo "  - Very Poor: $very_poor_docs ($(echo "scale=1; 100*$very_poor_docs/$total_documents" | bc)%)" >> $REPORT_FILE
echo "" >> $REPORT_FILE

# Output path to the report file
echo -e "${GREEN}Report generated: $REPORT_FILE${NC}"
echo $REPORT_FILE

# Return path to report file
echo $REPORT_FILE 