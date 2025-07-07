#!/bin/bash

# Robust Slippi Directory Processor
# Recursively processes all .slp files in a directory with better error handling

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Default values
INPUT_DIR=""
OUTPUT_DIR=""
FORMAT="json"
VERBOSE=false
DRY_RUN=false

# Function to print usage
usage() {
    echo "Usage: $0 [OPTIONS] <input_directory>"
    echo ""
    echo "Options:"
    echo "  -o, --output <dir>     Output directory (default: input_dir/processed)"
    echo "  -f, --format <format>  Output format: json or text (default: json)"
    echo "  -v, --verbose          Verbose output"
    echo "  -d, --dry-run          Show what would be processed"
    echo "  -h, --help             Show this help message"
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -f|--format)
            FORMAT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [[ -z "$INPUT_DIR" ]]; then
                INPUT_DIR="$1"
            else
                echo -e "${RED}Error: Multiple input directories specified${NC}"
                usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Validate input
if [[ -z "$INPUT_DIR" ]]; then
    echo -e "${RED}Error: Input directory is required${NC}"
    usage
    exit 1
fi

if [[ ! -d "$INPUT_DIR" ]]; then
    echo -e "${RED}Error: Input directory does not exist: $INPUT_DIR${NC}"
    exit 1
fi

# Set default output directory
if [[ -z "$OUTPUT_DIR" ]]; then
    OUTPUT_DIR="$INPUT_DIR/processed"
fi

# Validate format
if [[ "$FORMAT" != "json" && "$FORMAT" != "text" ]]; then
    echo -e "${RED}Error: Invalid format: $FORMAT. Use 'json' or 'text'${NC}"
    exit 1
fi

# Check if parser exists
PARSER_PATH="./target/release/slippi_parser_service"
if [[ ! -f "$PARSER_PATH" ]]; then
    echo -e "${RED}Error: Slippi parser not found at $PARSER_PATH${NC}"
    echo "Please build with: cargo build --release"
    exit 1
fi

echo -e "${BLUE}Starting Slippi directory processing${NC}"
echo "Input: $INPUT_DIR"
echo "Output: $OUTPUT_DIR"
echo "Format: $FORMAT"
echo "Verbose: $VERBOSE"
echo "Dry run: $DRY_RUN"
echo ""

# Count files first
file_count=$(find "$INPUT_DIR" -name "*.slp" -type f | wc -l)
echo -e "${BLUE}Found $file_count .slp files${NC}"

if [[ $file_count -eq 0 ]]; then
    echo -e "${YELLOW}No .slp files found in $INPUT_DIR${NC}"
    exit 0
fi

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${BLUE}DRY RUN - Files that would be processed:${NC}"
    find "$INPUT_DIR" -name "*.slp" -type f | while read -r file; do
        relative_path="${file#$INPUT_DIR/}"
        base_name="${relative_path%.slp}"
        output_file="$OUTPUT_DIR/${base_name}.$FORMAT"
        echo "  $file -> $output_file"
    done
    exit 0
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Process files with progress tracking
processed=0
failed=0
current=0

find "$INPUT_DIR" -name "*.slp" -type f | while read -r file; do
    ((current++))
    
    # Create output filename
    relative_path="${file#$INPUT_DIR/}"
    base_name="${relative_path%.slp}"
    output_file="$OUTPUT_DIR/${base_name}.$FORMAT"
    
    # Create subdirectory if needed
    mkdir -p "$(dirname "$output_file")"
    
    # Show progress
    echo -ne "\r${BLUE}Progress: $current/$file_count (${processed} success, ${failed} failed)${NC}"
    
    if [[ "$VERBOSE" == true ]]; then
        echo ""
        echo -e "${BLUE}Processing: $file${NC}"
    fi
    
    # Process the file with error handling
    if "$PARSER_PATH" -f "$file" --format "$FORMAT" > "$output_file" 2>/dev/null; then
        if [[ "$VERBOSE" == true ]]; then
            echo -e "${GREEN}Success: $file${NC}"
        fi
        ((processed++))
    else
        if [[ "$VERBOSE" == true ]]; then
            echo -e "${YELLOW}Failed: $file${NC}"
        fi
        ((failed++))
        # Remove failed output file if it exists
        rm -f "$output_file"
    fi
done

echo ""
echo ""
echo -e "${BLUE}Processing complete!${NC}"
echo -e "${GREEN}Successfully processed: $processed files${NC}"
if [[ $failed -gt 0 ]]; then
    echo -e "${YELLOW}Failed to process: $failed files${NC}"
fi
echo -e "${BLUE}Output files saved to: $OUTPUT_DIR${NC}" 