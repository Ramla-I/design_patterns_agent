#!/usr/bin/env bash
#
# run_crat_test.sh — Run the design patterns agent on each test case in a crat_source directory.
#
# Usage:
#   ./run_crat_test.sh /path/to/crat_source
#   ./run_crat_test.sh                                   # defaults to llm_translation run
#   ./run_crat_test.sh /path/to/crat_source --provider anthropic --model claude-sonnet-4-20250514
#
# Extra arguments after the crat_source path are forwarded to the agent.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
DEFAULT_CRAT_SOURCE="/Users/ramla/Projects/llm_translation/runs/gpt-5.2_20260309_124237/crat_source"

# ── Parse arguments ──────────────────────────────────────────────────────────

CRAT_SOURCE="${1:-$DEFAULT_CRAT_SOURCE}"
if [[ -d "$CRAT_SOURCE" ]]; then
    shift || true
else
    CRAT_SOURCE="$DEFAULT_CRAT_SOURCE"
fi

EXTRA_ARGS=("$@")

# ── Pre-flight checks ───────────────────────────────────────────────────────

if [[ ! -d "$CRAT_SOURCE" ]]; then
    echo "Error: crat_source directory not found: $CRAT_SOURCE" >&2
    exit 1
fi

if [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -z "${OPENAI_API_KEY:-}" ]]; then
    echo "Error: Set ANTHROPIC_API_KEY or OPENAI_API_KEY before running." >&2
    exit 1
fi

# ── Build the agent ──────────────────────────────────────────────────────────

echo "==> Building design_patterns_agent..."
cargo build --release --no-default-features --manifest-path "${SCRIPT_DIR}/Cargo.toml"
AGENT="${SCRIPT_DIR}/target/release/design_patterns_agent"

# ── Collect test cases ───────────────────────────────────────────────────────

CASES=()
for dir in "${CRAT_SOURCE}"/*/; do
    rust_dir="${dir}translated_rust_llm"
    if [[ -d "$rust_dir" ]]; then
        CASES+=("$rust_dir")
    fi
done

TOTAL=${#CASES[@]}
echo "==> Found ${TOTAL} test cases in ${CRAT_SOURCE}"

if [[ "$TOTAL" -eq 0 ]]; then
    echo "No test cases found." >&2
    exit 1
fi

# ── Create batch output directory ────────────────────────────────────────────

RUNS_DIR="${SCRIPT_DIR}/runs"
BATCH_TS="$(date +%Y%m%d_%H%M%S)"
BATCH_DIR="${RUNS_DIR}/crat_batch_${BATCH_TS}"
mkdir -p "$BATCH_DIR"

echo "==> Batch output: ${BATCH_DIR}"

PASSED=0
FAILED=0
ERRORS=()

# ── Run analysis on each test case ───────────────────────────────────────────

for i in "${!CASES[@]}"; do
    rust_dir="${CASES[$i]}"
    case_name="$(basename "$(dirname "$rust_dir")")"
    n=$((i + 1))

    echo ""
    echo "─── [${n}/${TOTAL}] ${case_name} ───"

    # Snapshot runs/ before this invocation
    before="$(ls "${RUNS_DIR}" 2>/dev/null)"

    if "$AGENT" analyze "$rust_dir" \
        --concurrency 1 \
        --token-budget 50000 \
        "${EXTRA_ARGS[@]}" 2>&1; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
        ERRORS+=("$case_name")
        echo "  !! FAILED: ${case_name}"
    fi

    # Move newly created run directory into the batch folder, renamed to the case name
    after="$(ls "${RUNS_DIR}" 2>/dev/null)"
    new_dir="$(comm -13 <(echo "$before") <(echo "$after") | grep -v "^crat_batch_" | head -1)"
    if [[ -n "$new_dir" ]] && [[ -d "${RUNS_DIR}/${new_dir}" ]]; then
        mv "${RUNS_DIR}/${new_dir}" "${BATCH_DIR}/${case_name}"
    fi
done

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "  Results: ${PASSED} passed, ${FAILED} failed out of ${TOTAL}"
echo "══════════════════════════════════════════════════════════════"

if [[ ${#ERRORS[@]} -gt 0 ]]; then
    echo ""
    echo "Failed cases:"
    for e in "${ERRORS[@]}"; do
        echo "  - $e"
    done
fi

echo ""
echo "==> All results saved to ${BATCH_DIR}"
ls "${BATCH_DIR}" | head -20
echo "  (${TOTAL} total cases)"
