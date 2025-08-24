#!/usr/bin/env bash

# Script to run the CIM LLM Adapter Service
# This handles environment setup and service startup

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 CIM LLM Adapter Service Launcher${NC}"
echo "======================================"

# Check for required environment variables
check_env() {
    local var_name=$1
    local var_value=${!var_name}
    
    if [ -z "$var_value" ]; then
        echo -e "${RED}❌ Error: $var_name is not set${NC}"
        return 1
    else
        echo -e "${GREEN}✅ $var_name is configured${NC}"
        return 0
    fi
}

# Load environment from .env file if it exists
if [ -f .env ]; then
    echo -e "${YELLOW}📄 Loading .env file...${NC}"
    export $(cat .env | grep -v '^#' | xargs)
fi

# Check for API key in secrets directory
if [ -z "$ANTHROPIC_API_KEY" ] && [ -f "cim-claude-adapter/secrets/claude.api.key" ]; then
    echo -e "${YELLOW}🔑 Loading API key from secrets directory...${NC}"
    export ANTHROPIC_API_KEY=$(cat cim-claude-adapter/secrets/claude.api.key | tr -d '\n')
fi

# Check required environment variables
echo -e "\n${YELLOW}🔍 Checking environment...${NC}"
ENV_VALID=true

if ! check_env "ANTHROPIC_API_KEY"; then
    echo -e "${YELLOW}💡 Tip: Set your Anthropic API key:${NC}"
    echo "  export ANTHROPIC_API_KEY='your-api-key-here'"
    ENV_VALID=false
fi

# Set default NATS URL if not provided
if [ -z "$NATS_URL" ]; then
    export NATS_URL="nats://localhost:4222"
    echo -e "${YELLOW}ℹ️  Using default NATS_URL: $NATS_URL${NC}"
else
    echo -e "${GREEN}✅ NATS_URL is configured: $NATS_URL${NC}"
fi

# Set default log level if not provided
if [ -z "$RUST_LOG" ]; then
    export RUST_LOG="cim_llm_adapter=debug,info"
    echo -e "${YELLOW}ℹ️  Using default RUST_LOG: $RUST_LOG${NC}"
fi

if [ "$ENV_VALID" = false ]; then
    echo -e "\n${RED}❌ Please configure required environment variables${NC}"
    exit 1
fi

# Check if NATS is running
echo -e "\n${YELLOW}🔍 Checking NATS server...${NC}"
if ! nc -z localhost 4222 2>/dev/null; then
    echo -e "${RED}❌ NATS server not detected on localhost:4222${NC}"
    echo -e "${YELLOW}Please ensure your NATS server is running${NC}"
    exit 1
else
    echo -e "${GREEN}✅ NATS server is running${NC}"
fi

# Build the service if needed
echo -e "\n${YELLOW}🔨 Building LLM Adapter Service...${NC}"
cargo build -p cim-llm-adapter --bin llm-adapter-service

# Create required NATS streams and KV buckets
echo -e "\n${YELLOW}📦 Setting up NATS resources...${NC}"
if command -v nats &> /dev/null; then
    # Create streams
    nats stream add CIM_LLM_PROVIDERS \
        --subjects="cim.llm.provider.>" \
        --storage=file \
        --retention=interest \
        --max-age=365d \
        --no-deny-delete \
        --no-deny-purge \
        2>/dev/null || echo "Stream CIM_LLM_PROVIDERS already exists"
    
    nats stream add CIM_LLM_DIALOGS \
        --subjects="cim.llm.dialog.>" \
        --storage=file \
        --retention=work \
        --max-age=90d \
        --no-deny-delete \
        --no-deny-purge \
        2>/dev/null || echo "Stream CIM_LLM_DIALOGS already exists"
    
    # Create KV buckets
    nats kv add cim-llm-providers \
        --storage=file \
        --history=10 \
        2>/dev/null || echo "KV bucket cim-llm-providers already exists"
    
    nats kv add cim-llm-contexts \
        --storage=file \
        --history=5 \
        --ttl=24h \
        2>/dev/null || echo "KV bucket cim-llm-contexts already exists"
    
    echo -e "${GREEN}✅ NATS resources configured${NC}"
else
    echo -e "${YELLOW}⚠️  NATS CLI not found, skipping resource setup${NC}"
fi

# Run the service
echo -e "\n${GREEN}🚀 Starting CIM LLM Adapter Service...${NC}"
echo "======================================"
echo -e "NATS URL: ${YELLOW}$NATS_URL${NC}"
echo -e "Log Level: ${YELLOW}$RUST_LOG${NC}"
echo -e "\n${GREEN}Service is starting...${NC}\n"

# Run the service
exec cargo run -p cim-llm-adapter --bin llm-adapter-service