#!/bin/bash
# Run test vectors against translated Rust code for one or more programs.
#
# Usage:
#   ./scripts/run_tests.sh Public-Tests/B01_organic/bin2hex_lib
#   ./scripts/run_tests.sh Public-Tests/B01_organic              # all programs in batch
#   ./scripts/run_tests.sh Public-Tests/B01_organic/bin2hex_lib --llm  # test translated_rust_llm
#   ./scripts/run_tests.sh Public-Tests/B01_organic -v           # verbose: show diff on failure

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

USE_LLM=false
VERBOSE=false
TARGET_PATH=""

# Parse arguments
for arg in "$@"; do
    case "$arg" in
        --llm) USE_LLM=true ;;
        -v|--verbose) VERBOSE=true ;;
        -*) echo -e "${RED}Unknown option: $arg${NC}"; exit 1 ;;
        *) TARGET_PATH="$arg" ;;
    esac
done

if [ -z "$TARGET_PATH" ]; then
    echo "Usage: $0 <program_or_batch_dir> [--llm] [-v|--verbose]"
    echo ""
    echo "Examples:"
    echo "  $0 Public-Tests/B01_organic/bin2hex_lib        # single program"
    echo "  $0 Public-Tests/B01_organic                    # entire batch"
    echo "  $0 Public-Tests/B01_organic/bin2hex_lib --llm  # test LLM translation"
    echo "  $0 Public-Tests/B01_organic -v                 # verbose (diff on failure)"
    exit 1
fi

# Resolve to absolute path
if [[ ! "$TARGET_PATH" = /* ]]; then
    TARGET_PATH="$PROJECT_ROOT/$TARGET_PATH"
fi

# Strip trailing slash
TARGET_PATH="${TARGET_PATH%/}"

if [ ! -d "$TARGET_PATH" ]; then
    echo -e "${RED}Error: Directory not found: $TARGET_PATH${NC}"
    exit 1
fi

RUST_DIR="translated_rust"
if $USE_LLM; then
    RUST_DIR="translated_rust_llm"
fi

# Collect programs to test
PROGRAMS=()

if [ -d "$TARGET_PATH/runner" ]; then
    # Single program
    PROGRAMS+=("$TARGET_PATH")
elif [ -d "$TARGET_PATH" ]; then
    # Batch directory — find all programs with a runner
    for dir in "$TARGET_PATH"/*/; do
        if [ -d "${dir}runner" ] && [ -d "${dir}${RUST_DIR}" ]; then
            PROGRAMS+=("${dir%/}")
        fi
    done
else
    echo -e "${RED}Error: No programs found in $TARGET_PATH${NC}"
    exit 1
fi

if [ ${#PROGRAMS[@]} -eq 0 ]; then
    echo -e "${RED}Error: No programs with ${RUST_DIR}/ and runner/ found${NC}"
    exit 1
fi

TOTAL=${#PROGRAMS[@]}
PASS_COUNT=0
FAIL_COUNT=0
BUILD_FAIL_COUNT=0
FAILED_PROGRAMS=()

echo "============================================"
echo -e " Test Runner — ${CYAN}${RUST_DIR}${NC}"
echo "============================================"
echo -e "Programs: ${TOTAL}"
echo ""

for i in "${!PROGRAMS[@]}"; do
    PROGRAM="${PROGRAMS[$i]}"
    NAME="$(basename "$PROGRAM")"
    CURRENT=$((i + 1))
    RUST_PATH="$PROGRAM/$RUST_DIR"
    RUNNER_PATH="$PROGRAM/runner"

    if [ ! -d "$RUST_PATH" ]; then
        echo -e "${YELLOW}[$CURRENT/$TOTAL]${NC} $NAME — ${YELLOW}skipped${NC} (no $RUST_DIR)"
        continue
    fi

    printf "${YELLOW}[%d/%d]${NC} %-40s" "$CURRENT" "$TOTAL" "$NAME"

    # Build the candidate cdylib
    BUILD_OUTPUT=$(cd "$RUST_PATH" && cargo build --release 2>&1) || {
        echo -e "${RED}BUILD FAILED${NC}"
        BUILD_FAIL_COUNT=$((BUILD_FAIL_COUNT + 1))
        FAILED_PROGRAMS+=("$NAME (build)")
        if $VERBOSE; then
            echo "$BUILD_OUTPUT" | tail -20 | sed 's/^/    /'
        fi
        continue
    }

    # Run test vectors
    DIFF_FLAG=""
    if $VERBOSE; then
        DIFF_FLAG="-d"
    fi

    TEST_OUTPUT=$(cd "$RUNNER_PATH" && RUST_ARTIFACTS=1 cargo run -q -- lib $DIFF_FLAG 2>&1) || {
        # Non-zero exit means at least one test failed
        PASS_LINES=$(echo "$TEST_OUTPUT" | grep -c ": true" || true)
        FAIL_LINES=$(echo "$TEST_OUTPUT" | grep -c ": false" || true)
        echo -e "${RED}FAIL${NC} ($PASS_LINES passed, $FAIL_LINES failed)"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAILED_PROGRAMS+=("$NAME")
        if $VERBOSE; then
            echo "$TEST_OUTPUT" | sed 's/^/    /'
        fi
        continue
    }

    PASS_LINES=$(echo "$TEST_OUTPUT" | grep -c ": true" || true)
    echo -e "${GREEN}PASS${NC} ($PASS_LINES/$PASS_LINES)"
    PASS_COUNT=$((PASS_COUNT + 1))
done

echo ""
echo "============================================"
echo " Results"
echo "============================================"
echo -e " Passed:       ${GREEN}${PASS_COUNT}${NC}"
echo -e " Failed:       ${RED}${FAIL_COUNT}${NC}"
echo -e " Build errors: ${RED}${BUILD_FAIL_COUNT}${NC}"
echo -e " Total:        ${TOTAL}"

if [ ${#FAILED_PROGRAMS[@]} -gt 0 ]; then
    echo ""
    echo "Failed:"
    for prog in "${FAILED_PROGRAMS[@]}"; do
        echo -e "  ${RED}-${NC} $prog"
    done
fi

echo ""

# Exit with failure if anything failed
if [ $((FAIL_COUNT + BUILD_FAIL_COUNT)) -gt 0 ]; then
    exit 1
fi
