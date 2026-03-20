#!/usr/bin/env bash
#
# run_crat_test.sh — Run the design patterns agent (via Docker) on each test case
#                     in a crat_source directory.
#
# Usage:
#   ./run_crat_test.sh /path/to/crat_source
#   ./run_crat_test.sh                                   # defaults to bundled crat_source
#   ./run_crat_test.sh /path/to/crat_source --provider anthropic --model claude-sonnet-4-20250514
#
# Extra arguments after the crat_source path are forwarded to the agent.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${0}")" && pwd)"
IMAGE_NAME="dpa-crat"
DEFAULT_CRAT_SOURCE="${SCRIPT_DIR}/runs/gpt-5.2_20260309_124237/crat_source"

# ── Parse arguments ──────────────────────────────────────────────────────────

CRAT_SOURCE="${1:-$DEFAULT_CRAT_SOURCE}"
if [[ -d "$CRAT_SOURCE" ]]; then
    shift || true
else
    CRAT_SOURCE="$DEFAULT_CRAT_SOURCE"
fi

# Make CRAT_SOURCE an absolute path
CRAT_SOURCE="$(cd "$CRAT_SOURCE" && pwd)"

EXTRA_ARGS=("$@")

# ── Pre-flight checks ───────────────────────────────────────────────────────

if ! command -v docker &>/dev/null; then
    echo "Error: docker is not installed or not in PATH." >&2
    exit 1
fi

if [[ ! -d "$CRAT_SOURCE" ]]; then
    echo "Error: crat_source directory not found: $CRAT_SOURCE" >&2
    exit 1
fi

if [[ -z "${ANTHROPIC_API_KEY:-}" ]] && [[ -z "${OPENAI_API_KEY:-}" ]]; then
    echo "Error: Set ANTHROPIC_API_KEY or OPENAI_API_KEY before running." >&2
    exit 1
fi

# ── Build the Docker image ──────────────────────────────────────────────────

echo "==> Building Docker image: ${IMAGE_NAME}"
docker build -t "${IMAGE_NAME}" -f "${SCRIPT_DIR}/Dockerfile.crat" "${SCRIPT_DIR}"

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

# ── Prepare environment flags ────────────────────────────────────────────────

ENV_FLAGS=()
[[ -n "${ANTHROPIC_API_KEY:-}" ]] && ENV_FLAGS+=(-e "ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}")
[[ -n "${OPENAI_API_KEY:-}" ]]    && ENV_FLAGS+=(-e "OPENAI_API_KEY=${OPENAI_API_KEY}")

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

    # Each container writes its output to /workspace/runs inside the container.
    # We mount a case-specific host directory there.
    CASE_OUT="${BATCH_DIR}/${case_name}"
    mkdir -p "$CASE_OUT"

    # The test case source is mounted read-only at /data/test_case inside the container.
    # Override the entrypoint to point at the mounted test case instead of /data/rust/library.
    if docker run --rm \
        "${ENV_FLAGS[@]}" \
        -v "${rust_dir}:/data/test_case" \
        -v "${CASE_OUT}:/workspace/runs" \
        --entrypoint design_patterns_agent \
        "${IMAGE_NAME}" \
        analyze /data/test_case \
        --concurrency 1 \
        --token-budget 50000 \
        "${EXTRA_ARGS[@]}" 2>&1; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
        ERRORS+=("$case_name")
        echo "  !! FAILED: ${case_name}"
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
