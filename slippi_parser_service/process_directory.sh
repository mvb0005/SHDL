#!/bin/bash

# Slippi Directory Processor
# Recursively processes all .slp files in a directory

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
    echo "  -o, --output <dir>     Output directory for processed files (default: input_dir/processed)"
    echo "  -f, --format <format>  Output format: json or text (default: json)"
    echo "  -v, --verbose          Verbose output"
    echo "  -d, --dry-run          Show what would be processed without actually processing"
    echo "  -h, --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 /path/to/slippi/replays"
    echo "  $0 -o /path/to/output -f text /path/to/slippi/replays"
    echo "  $0 -v -d /path/to/slippi/replays"
}

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Parse command line arguments
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
        -*)
            print_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            if [[ -z "$INPUT_DIR" ]]; then
                INPUT_DIR="$1"
            else
                print_error "Multiple input directories specified"
                usage
                exit 1
            fi
            shift
            ;;
    esac
done

# Check if input directory is provided
if [[ -z "$INPUT_DIR" ]]; then
    print_error "Input directory is required"
    usage
    exit 1
fi

# Check if input directory exists
if [[ ! -d "$INPUT_DIR" ]]; then
    print_error "Input directory does not exist: $INPUT_DIR"
    exit 1
fi

# Set default output directory if not specified
if [[ -z "$OUTPUT_DIR" ]]; then
    OUTPUT_DIR="$INPUT_DIR/processed"
fi

# Validate format
if [[ "$FORMAT" != "json" && "$FORMAT" != "text" ]]; then
    print_error "Invalid format: $FORMAT. Use 'json' or 'text'"
    exit 1
fi

# Check if slippi_parser_service exists
PARSER_PATH="./target/release/slippi_parser_service"
if [[ ! -f "$PARSER_PATH" ]]; then
    print_error "Slippi parser service not found at $PARSER_PATH"
    print_info "Please build the project first with: cargo build --release"
    exit 1
fi

# Function to process a single file
process_file() {
    local file="$1"
    local output_file="$2"
    
    if [[ "$VERBOSE" == true ]]; then
        print_info "Processing: $file"
    fi
    
    if [[ "$DRY_RUN" == true ]]; then
        print_info "Would process: $file -> $output_file"
        return 0
    fi
    
    # Create output directory if it doesn't exist
    mkdir -p "$(dirname "$output_file")"
    
    # Process the file
    if "$PARSER_PATH" -f "$file" --format "$FORMAT" > "$output_file" 2>/dev/null; then
        if [[ "$VERBOSE" == true ]]; then
            print_success "Processed: $file"
        fi
        return 0
    else
        print_warning "Failed to process: $file"
        return 1
    fi
}

# Main processing logic
main() {
    print_info "Starting Slippi directory processing"
    print_info "Input directory: $INPUT_DIR"
    print_info "Output directory: $OUTPUT_DIR"
    print_info "Format: $FORMAT"
    print_info "Verbose: $VERBOSE"
    print_info "Dry run: $DRY_RUN"
    echo ""
    
    # Find all .slp files
    local slp_files=()
    while IFS= read -r -d '' file; do
        slp_files+=("$file")
    done < <(find "$INPUT_DIR" -name "*.slp" -type f -print0)
    
    local total_files=${#slp_files[@]}
    
    if [[ $total_files -eq 0 ]]; then
        print_warning "No .slp files found in $INPUT_DIR"
        exit 0
    fi
    
    print_info "Found $total_files .slp files to process"
    
    if [[ "$DRY_RUN" == true ]]; then
        print_info "DRY RUN - No files will be processed"
        echo ""
    fi
    
    # Process each file
    local processed=0
    local failed=0
    
    for file in "${slp_files[@]}"; do
        # Create output filename
        local relative_path="${file#$INPUT_DIR/}"
        local base_name="${relative_path%.slp}"
        local output_file="$OUTPUT_DIR/${base_name}.$FORMAT"
        
        if process_file "$file" "$output_file"; then
            ((processed++))
        else
            ((failed++))
        fi
    done
    
    echo ""
    print_info "Processing complete!"
    print_success "Successfully processed: $processed files"
    if [[ $failed -gt 0 ]]; then
        print_warning "Failed to process: $failed files"
    fi
    print_info "Output files saved to: $OUTPUT_DIR"
}

# Run main function
main 