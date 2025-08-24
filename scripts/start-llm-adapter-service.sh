#!/usr/bin/env bash
# Start LLM Adapter Service
# Part of CIM Agent Claude ecosystem

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🤖 Starting CIM LLM Adapter Service${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if we're in a Nix environment
if [[ -z "${IN_NIX_SHELL:-}" ]]; then
    echo -e "${YELLOW}⚠️  Not in Nix shell. Entering development environment...${NC}"
    exec nix develop --command "$0" "$@"
fi

# Check required environment variables
if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
    echo -e "${RED}❌ ANTHROPIC_API_KEY environment variable is required${NC}"
    echo "Please set your Claude API key:"
    echo "  export ANTHROPIC_API_KEY=your-api-key-here"
    exit 1
fi

# Set default environment variables if not provided
export NATS_URL="${NATS_URL:-nats://localhost:4222}"
export CIM_DOMAIN="${CIM_DOMAIN:-local}"
export RUST_LOG="${RUST_LOG:-info,cim_llm_adapter=debug}"

echo -e "${GREEN}Environment Configuration:${NC}"
echo "  NATS URL: $NATS_URL"
echo "  Domain: $CIM_DOMAIN"
echo "  Log Level: $RUST_LOG"
echo "  Claude API: ✅ Configured"
echo ""

# Check if NATS server is running
echo -e "${BLUE}🔍 Checking NATS server connectivity...${NC}"
if ! nats server ping --timeout=5s >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  NATS server not responding. Attempting to start local NATS server...${NC}"
    
    # Try to start NATS server in background
    if command -v nats-server >/dev/null 2>&1; then
        nats-server --jetstream --store_dir ./nats-data &
        NATS_PID=$!
        echo "Started NATS server with PID: $NATS_PID"
        sleep 2
    else
        echo -e "${RED}❌ NATS server not found. Please install and start NATS server first.${NC}"
        echo "Installation: https://docs.nats.io/running-a-nats-service/introduction/installation"
        exit 1
    fi
else
    echo -e "${GREEN}✅ NATS server is running${NC}"
fi

echo ""
echo -e "${BLUE}🚀 Starting LLM Adapter Service...${NC}"
echo ""

# Build and run the service
if cargo run --bin llm-adapter-service 2>&1 | while IFS= read -r line; do
    # Color code log levels
    if [[ $line == *"ERROR"* ]]; then
        echo -e "${RED}$line${NC}"
    elif [[ $line == *"WARN"* ]]; then
        echo -e "${YELLOW}$line${NC}"
    elif [[ $line == *"INFO"* ]]; then
        echo -e "${GREEN}$line${NC}"
    elif [[ $line == *"DEBUG"* ]]; then
        echo -e "${BLUE}$line${NC}"
    else
        echo "$line"
    fi
done; then
    echo -e "${GREEN}✅ LLM Adapter Service completed successfully${NC}"
else
    echo -e "${RED}❌ LLM Adapter Service failed${NC}"
    exit 1
fi