#!/usr/bin/env bash
#
# run_stdlib_test.sh — Build and run the design patterns agent on the Rust standard library.
#
# Usage:
#   ./run_stdlib_test.sh                          # uses ANTHROPIC_API_KEY from env
#   ANTHROPIC_API_KEY=sk-ant-... ./run_stdlib_test.sh
#   OPENAI_API_KEY=sk-... ./run_stdlib_test.sh --provider openai --model gpt-4o
#
# Any extra arguments are forwarded to the container and override the defaults.
# Examples:
#   ./run_stdlib_test.sh --concurrency 10 --token-budget 2000000
#   ./run_stdlib_test.sh --resume /workspace/runs/<prev_run>/progress.jsonl
#
set -euo pipefail

IMAGE_NAME="dpa-stdlib"
RUNS_DIR="$(pwd)/runs"

# ── Pre-flight checks ────────────────────────────────────────────────────────

if ! command -v docker &>/dev/null; then
    echo "Error: docker is not installed or not in PATH." >&2
    exit 1
fi

if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "Error: Set ANTHROPIC_API_KEY or OPENAI_API_KEY before running." >&2
    echo "  export ANTHROPIC_API_KEY=sk-ant-..." >&2
    echo "  export OPENAI_API_KEY=sk-..." >&2
    exit 1
fi

# ── Build ─────────────────────────────────────────────────────────────────────

echo "==> Building Docker image: ${IMAGE_NAME}"
docker build -t "${IMAGE_NAME}" .

# ── Prepare output directory ──────────────────────────────────────────────────

mkdir -p "${RUNS_DIR}"

# ── Run ───────────────────────────────────────────────────────────────────────

echo ""
echo "==> Starting analysis of rust-lang/rust/library/"
echo "    Output directory: ${RUNS_DIR}"
echo ""

ENV_FLAGS=()
[ -n "${ANTHROPIC_API_KEY:-}" ] && ENV_FLAGS+=(-e "ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}")
[ -n "${OPENAI_API_KEY:-}" ]    && ENV_FLAGS+=(-e "OPENAI_API_KEY=${OPENAI_API_KEY}")

docker run --rm \
    "${ENV_FLAGS[@]}" \
    -v "${RUNS_DIR}:/workspace/runs" \
    "${IMAGE_NAME}" \
    "$@"

echo ""
echo "==> Done. Results saved to ${RUNS_DIR}"
ls -lt "${RUNS_DIR}" | head -5
