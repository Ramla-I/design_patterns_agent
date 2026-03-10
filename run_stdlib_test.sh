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
#   ./run_stdlib_test.sh --validate   # second-pass review via gpt-4o-mini (low cost, reduces hallucinations)
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

# Validation uses gpt-4o-mini by default, so OPENAI_API_KEY is needed alongside ANTHROPIC_API_KEY
if [ -n "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "Warning: OPENAI_API_KEY not set. Validation pass (--validate) uses gpt-4o-mini by default." >&2
    echo "  Set OPENAI_API_KEY or pass --validation-model to use an Anthropic model instead." >&2
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

# When extra arguments are provided, they replace Docker's CMD entirely.
# Merge the defaults with any user overrides so --multi-crate etc. are always present.
DEFAULT_ARGS=(
    "--multi-crate"
    "--concurrency" "5"
    "--priority-modules" "sync,io,fs,net,cell,collections,thread,process"
    "--provider" "anthropic"
    "--model" "claude-sonnet-4-20250514"
    "--token-budget" "1000000"
    "--validate"
    "--validation-model" "gpt-4o-mini"
)

# User args override defaults: build a set of flags the user explicitly provided,
# then only include defaults whose flag was NOT overridden.
MERGED_ARGS=()
user_flags=" $* "
i=0
while [ $i -lt ${#DEFAULT_ARGS[@]} ]; do
    flag="${DEFAULT_ARGS[$i]}"
    if [[ "$flag" == --* ]]; then
        if [[ "$user_flags" == *" $flag "* ]]; then
            # User overrides this flag — skip the default (and its value if any)
            i=$((i + 1))
            # Skip the value too if the next element is not a flag
            if [ $i -lt ${#DEFAULT_ARGS[@]} ] && [[ "${DEFAULT_ARGS[$i]}" != --* ]]; then
                i=$((i + 1))
            fi
            continue
        fi
    fi
    MERGED_ARGS+=("${DEFAULT_ARGS[$i]}")
    i=$((i + 1))
done

# Append user args after defaults (user args take precedence for flags like --provider)
MERGED_ARGS+=("$@")

docker run --rm \
    "${ENV_FLAGS[@]}" \
    -v "${RUNS_DIR}:/workspace/runs" \
    "${IMAGE_NAME}" \
    "${MERGED_ARGS[@]}"

echo ""
echo "==> Done. Results saved to ${RUNS_DIR}"
ls -lt "${RUNS_DIR}" | head -5
