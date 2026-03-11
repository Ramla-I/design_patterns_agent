#!/usr/bin/env bash
#
# run_stdlib_sync_test.sh — Small, targeted end-to-end test that analyzes ONLY
# the std::sync module tree from the local Rust toolchain.
#
# Creates a temporary crate containing just sync/ (symlinked from the real
# stdlib), so the tool never touches io, fs, net, collections, etc.
#
# This exercises the full pipeline (parse → module graph → BFS traversal →
# clustering → LLM inference → dedup → report) on a real, deeply-nested
# module hierarchy (sync → mpmc/poison → array/waker/mutex/condvar/…).
#
# Usage:
#   ./run_stdlib_sync_test.sh                              # defaults to openai / gpt-5.2
#   ./run_stdlib_sync_test.sh --provider anthropic --model claude-sonnet-4-20250514
#   OPENAI_API_KEY=sk-... ./run_stdlib_sync_test.sh
#
set -euo pipefail

# ── Locate the Rust stdlib source via rustc --print sysroot ─────────────────

SYSROOT="$(rustc --print sysroot 2>/dev/null || true)"
STD_SRC="${SYSROOT}/lib/rustlib/src/rust/library/std/src"

if [ ! -d "${STD_SRC}/sync" ]; then
    echo "Error: cannot find std::sync source at ${STD_SRC}/sync" >&2
    echo "  Install it with: rustup component add rust-src" >&2
    exit 1
fi

# ── Pre-flight: API key ─────────────────────────────────────────────────────

if [ -z "${ANTHROPIC_API_KEY:-}" ] && [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "Error: Set ANTHROPIC_API_KEY or OPENAI_API_KEY before running." >&2
    exit 1
fi

# ── Build (no translation feature needed) ───────────────────────────────────

echo "==> Building design_patterns_agent (--no-default-features)"
cargo build --release --no-default-features

BINARY="./target/release/design_patterns_agent"

# ── Create a temp crate that contains only sync ─────────────────────────────
#
# Structure:
#   tmp/src/lib.rs          → "pub mod sync;"
#   tmp/src/sync/           → symlink to real stdlib std/src/sync/
#   tmp/src/sync/poison/    → (included via symlink, has condvar/mutex/once/rwlock)
#   tmp/src/sync/mpmc/      → (included via symlink, has array/context/counter/…)

TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

mkdir -p "${TMPDIR}/src"
echo "pub mod sync;" > "${TMPDIR}/src/lib.rs"
cp -R "${STD_SRC}/sync" "${TMPDIR}/src/sync"

echo "==> Created temp crate at ${TMPDIR}"
echo "    src/lib.rs → pub mod sync;"
echo "    src/sync/  → copied from ${STD_SRC}/sync"

# ── Default arguments ───────────────────────────────────────────────────────

DEFAULT_ARGS=(
    "--concurrency" "3"
    "--priority-modules" "sync"
    "--token-budget" "200000"
)

# Merge: skip any default flag the user explicitly overrides.
MERGED_ARGS=()
user_flags=" $* "
i=0
while [ $i -lt ${#DEFAULT_ARGS[@]} ]; do
    flag="${DEFAULT_ARGS[$i]}"
    if [[ "$flag" == --* ]] && [[ "$user_flags" == *" $flag "* ]]; then
        i=$((i + 1))
        if [ $i -lt ${#DEFAULT_ARGS[@]} ] && [[ "${DEFAULT_ARGS[$i]}" != --* ]]; then
            i=$((i + 1))
        fi
        continue
    fi
    MERGED_ARGS+=("${DEFAULT_ARGS[$i]}")
    i=$((i + 1))
done
MERGED_ARGS+=("$@")

# ── Run ─────────────────────────────────────────────────────────────────────

echo ""
echo "==> Analyzing std::sync module tree"
echo "    Args: ${MERGED_ARGS[*]}"
echo ""

"${BINARY}" analyze "${TMPDIR}" "${MERGED_ARGS[@]}"

echo ""
echo "==> Done.  Results in runs/"
ls -lt runs/ | head -5
