#!/bin/bash
# Batch analysis script for design_patterns_agent
# Runs invariant analysis on all programs in a given directory

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
BATCH_DIR=""
OUTPUT_DIR=""
BATCH_NAME=""

# Parse arguments
if [ $# -lt 1 ]; then
    echo "Usage: $0 <batch_directory> [output_name]"
    echo "Example: $0 Public-Tests/B02_organic"
    exit 1
fi

BATCH_DIR="$1"
BATCH_NAME="${2:-$(basename "$BATCH_DIR")}"

# Resolve paths
if [[ ! "$BATCH_DIR" = /* ]]; then
    BATCH_DIR="$PROJECT_ROOT/$BATCH_DIR"
fi

OUTPUT_DIR="$PROJECT_ROOT/reports/$BATCH_NAME"

# Verify batch directory exists
if [ ! -d "$BATCH_DIR" ]; then
    echo -e "${RED}Error: Batch directory does not exist: $BATCH_DIR${NC}"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "============================================"
echo "Design Patterns Agent - Batch Analysis"
echo "============================================"
echo "Batch directory: $BATCH_DIR"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Build the tool first
echo -e "${YELLOW}Building the tool...${NC}"
cd "$PROJECT_ROOT"
cargo build --release 2>&1 | tail -5

# Find all translated_rust directories
PROGRAMS=()
for dir in "$BATCH_DIR"/*/translated_rust; do
    if [ -d "$dir" ]; then
        PROGRAM_NAME=$(basename "$(dirname "$dir")")
        PROGRAMS+=("$PROGRAM_NAME")
    fi
done

TOTAL=${#PROGRAMS[@]}
echo ""
echo -e "${GREEN}Found $TOTAL programs to analyze${NC}"
echo ""

# Track results
SUCCESS_COUNT=0
FAIL_COUNT=0
FAILED_PROGRAMS=()

# Process each program
for i in "${!PROGRAMS[@]}"; do
    PROGRAM="${PROGRAMS[$i]}"
    PROGRAM_PATH="$BATCH_DIR/$PROGRAM/translated_rust"
    OUTPUT_FILE="$OUTPUT_DIR/${PROGRAM}.json"

    CURRENT=$((i + 1))
    echo -e "${YELLOW}[$CURRENT/$TOTAL]${NC} Analyzing $PROGRAM..."

    # Run the analysis
    if "$PROJECT_ROOT/target/release/design_patterns_agent" "$PROGRAM_PATH" --format json --output "$OUTPUT_FILE" 2>/dev/null; then
        echo -e "  ${GREEN}Success${NC} - Output: ${PROGRAM}.json"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        echo -e "  ${RED}Failed${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAILED_PROGRAMS+=("$PROGRAM")
        # Create empty report for failed programs
        echo '{"summary":{"total_invariants":0,"state_machine_count":0,"linear_type_count":0,"ownership_count":0,"modules_analyzed":0},"invariants":[]}' > "$OUTPUT_FILE"
    fi
done

echo ""
echo "============================================"
echo "Batch Analysis Complete"
echo "============================================"
echo -e "Successful: ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "Failed: ${RED}$FAIL_COUNT${NC}"

if [ ${#FAILED_PROGRAMS[@]} -gt 0 ]; then
    echo ""
    echo "Failed programs:"
    for prog in "${FAILED_PROGRAMS[@]}"; do
        echo "  - $prog"
    done
fi

echo ""
echo "Individual reports saved to: $OUTPUT_DIR/"
echo ""

# Run consolidation
echo -e "${YELLOW}Consolidating reports...${NC}"
python3 "$SCRIPT_DIR/consolidate_reports.py" "$OUTPUT_DIR" "$PROJECT_ROOT/reports/${BATCH_NAME}_analysis_report.md"

echo ""
echo -e "${GREEN}Done!${NC}"
echo "Consolidated report: reports/${BATCH_NAME}_analysis_report.md"
