#!/bin/bash
# THIS FILE TAKES A SNAPSHOT OF ALL .rs AND .toml FILES IN THIS REPO AND PLACES IT IN A FILE CALLED deep.rs AT PROJECT ROOT. RUN `bash deep.sh` AT PROJECT ROOT IF YOU HAVE A USE CASE LIKE AGENTIC CONTEXT DUMPING.

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Output file
OUTPUT_FILE="deep.rs"

# Auto-cleanup
if [ -f "$OUTPUT_FILE" ]; then
    echo -e "${YELLOW}Removing old $OUTPUT_FILE...${NC}"
    rm "$OUTPUT_FILE"
fi

echo -e "${BOLD}${BLUE}Collecting Rust files...${NC}"
echo ""

# Find all .rs and .toml files recursively, excluding common build artifacts
temp_file=$(mktemp)

# Create exclusion patterns for find command
# Exclude: target/, .git/, .vscode/, .idea/, dist/, build/, node_modules/
find . -type f \( -name "*.rs" -o -name "*.toml" \) \
    -not -path "./target/*" \
    -not -path "./.git/*" \
    -not -path "./.vscode/*" \
    -not -path "./.idea/*" \
    -not -path "./dist/*" \
    -not -path "./build/*" \
    -not -path "./node_modules/*" \
    -not -path "./.cargo/*" \
    -print0 | while IFS= read -r -d '' file; do
    # Remove leading ./
    clean_file="${file#./}"
    echo "--- ./$clean_file ---" >> "$temp_file"
    cat "$file" >> "$temp_file"
    echo "" >> "$temp_file"  # Add newline between files
done

# Move temp file to final output
mv "$temp_file" "$OUTPUT_FILE"

if [ -f "$OUTPUT_FILE" ]; then
    included_files=$(grep -c "^--- \./" "$OUTPUT_FILE" 2>/dev/null || echo 0)
    echo -e "${GREEN}Done!${NC}"
    echo -e "${GREEN}Files included:${NC} $included_files"
    echo -e "${CYAN}Output:${NC} $OUTPUT_FILE"
    
    size=$(du -h "$OUTPUT_FILE" 2>/dev/null | cut -f1)
    lines=$(wc -l < "$OUTPUT_FILE" 2>/dev/null || echo 0)
    echo -e "${CYAN}Size:${NC} $size"
    echo -e "${CYAN}Lines:${NC} $lines"
    echo ""
    echo -e "${GREEN}Ready to upload $OUTPUT_FILE!${NC}"
else
    echo -e "${RED}Error: Failed to create output file${NC}"
    exit 1
fi

exit 0
