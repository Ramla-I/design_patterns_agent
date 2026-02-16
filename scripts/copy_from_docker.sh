#!/bin/bash
# Copy Public-Tests and tools from the Docker container to the local project.
#
# Usage:
#   ./scripts/copy_from_docker.sh                  # uses container "great_galileo"
#   ./scripts/copy_from_docker.sh my_container      # uses a specific container

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CONTAINER="${1:-great_galileo}"
REMOTE_ROOT="/home/ubuntu/Test-Corpus"

# Verify container exists and is running
if ! docker inspect "$CONTAINER" &>/dev/null; then
    echo -e "${RED}Error: Container '$CONTAINER' not found${NC}"
    echo "Available containers:"
    docker ps -a --format "  {{.Names}} ({{.Status}})"
    exit 1
fi

echo "============================================"
echo " Copy from Docker: $CONTAINER"
echo "============================================"
echo ""

# --- Public-Tests ---
echo -e "${YELLOW}Copying Public-Tests/...${NC}"
if [ -d "$PROJECT_ROOT/Public-Tests" ]; then
    echo "  Removing existing Public-Tests/"
    rm -rf "$PROJECT_ROOT/Public-Tests"
fi
docker cp "$CONTAINER:$REMOTE_ROOT/Public-Tests" "$PROJECT_ROOT/Public-Tests"
PROGRAM_COUNT=$(find "$PROJECT_ROOT/Public-Tests" -maxdepth 3 -name "translated_rust" -type d | wc -l | tr -d ' ')
echo -e "  ${GREEN}Done${NC} — $PROGRAM_COUNT programs with translated_rust/"

# --- tools (cando2 etc.) ---
echo -e "${YELLOW}Copying tools/...${NC}"
if [ -d "$PROJECT_ROOT/tools" ]; then
    echo "  Removing existing tools/"
    rm -rf "$PROJECT_ROOT/tools"
fi
docker cp "$CONTAINER:$REMOTE_ROOT/tools" "$PROJECT_ROOT/tools"
echo -e "  ${GREEN}Done${NC}"

echo ""
echo "============================================"
echo -e " ${GREEN}All done.${NC}"
echo "============================================"
echo ""
echo "You can now run:"
echo "  ./scripts/run_tests.sh Public-Tests/B01_organic          # test a batch"
echo "  cargo run -- translate Public-Tests/B01_organic           # LLM translation"
echo "  ./scripts/run_batch_analysis.sh Public-Tests/B01_organic  # invariant analysis"
