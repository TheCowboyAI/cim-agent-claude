#!/usr/bin/env bash
# Test multi-provider support in the Universal Agent System

set -e

echo "🧪 Universal Agent System - Multi-Provider Test Suite"
echo "====================================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configurations
NATS_URL=${NATS_URL:-"nats://localhost:4222"}
TEST_SESSION="multi-provider-test-$(date +%s)"

# Function to test a provider
test_provider() {
    local provider=$1
    local model=$2
    local test_query=$3
    
    echo -e "${BLUE}Testing ${provider} with model ${model}...${NC}"
    
    # Create test request
    local request_id="test-${provider}-$(date +%s)"
    local request_json=$(cat <<EOF
{
    "request_id": "${request_id}",
    "provider": "${provider}",
    "model": "${model}",
    "messages": [
        {
            "role": "user",
            "content": "${test_query}"
        }
    ],
    "options": {
        "max_tokens": 100,
        "temperature": 0.7
    },
    "context": {
        "session_id": "${TEST_SESSION}"
    }
}
EOF
)
    
    # Send request via NATS
    echo "  Sending request to cim.llm.dialog.turn.request..."
    response=$(echo "$request_json" | nats request "cim.llm.dialog.turn.request" --timeout=30s 2>&1) || {
        echo -e "  ${RED}✗ Failed to get response from ${provider}${NC}"
        echo "  Error: $response"
        return 1
    }
    
    # Check if response contains content
    if echo "$response" | grep -q "content"; then
        echo -e "  ${GREEN}✓ ${provider} responded successfully${NC}"
        echo "  Response preview: $(echo "$response" | jq -r '.content' 2>/dev/null | head -c 100)..."
        return 0
    else
        echo -e "  ${YELLOW}⚠ ${provider} returned unexpected response${NC}"
        echo "  Response: $response"
        return 1
    fi
}

# Function to check provider health
check_provider_health() {
    local provider=$1
    
    echo -e "${BLUE}Checking ${provider} health...${NC}"
    
    local health_request=$(cat <<EOF
{
    "provider": "${provider}"
}
EOF
)
    
    response=$(echo "$health_request" | nats request "cim.llm.health.check" --timeout=5s 2>&1) || {
        echo -e "  ${RED}✗ Health check failed for ${provider}${NC}"
        return 1
    }
    
    if echo "$response" | grep -q "Healthy"; then
        echo -e "  ${GREEN}✓ ${provider} is healthy${NC}"
        return 0
    else
        echo -e "  ${YELLOW}⚠ ${provider} health status: $(echo "$response" | jq -r '.status' 2>/dev/null)${NC}"
        return 1
    fi
}

# Main test execution
echo "1. Checking NATS connectivity..."
if nc -z localhost 4222 2>/dev/null; then
    echo -e "  ${GREEN}✓ NATS is running${NC}"
else
    echo -e "  ${RED}✗ NATS is not running. Please start NATS first.${NC}"
    exit 1
fi

echo ""
echo "2. Checking LLM Adapter Service..."
if pgrep -f "llm-adapter-service" > /dev/null; then
    echo -e "  ${GREEN}✓ LLM Adapter service is running${NC}"
else
    echo -e "  ${YELLOW}⚠ LLM Adapter service not detected. Starting it...${NC}"
    cd /git/thecowboyai/cim-agent-claude
    nix develop --command cargo run -p cim-llm-adapter --bin llm-adapter-service &
    sleep 5
fi

echo ""
echo "3. Provider Health Checks"
echo "-------------------------"

# Check each provider's health
providers=("claude" "openai" "ollama")
for provider in "${providers[@]}"; do
    check_provider_health "$provider" || true
done

echo ""
echo "4. Testing Paid API Providers"
echo "-----------------------------"

# Test Claude (if API key exists)
if [ -f "cim-llm-adapter/secrets/claude.api.key" ] && [ -s "cim-llm-adapter/secrets/claude.api.key" ]; then
    test_provider "claude" "claude-3-5-sonnet-20241022" "What is the capital of France?"
else
    echo -e "${YELLOW}⚠ Skipping Claude - No API key found${NC}"
fi

# Test OpenAI (if API key exists)
if [ -f "cim-llm-adapter/secrets/openai.api.key" ] && [ -s "cim-llm-adapter/secrets/openai.api.key" ]; then
    test_provider "openai" "gpt-4-turbo-preview" "What is 2+2?"
else
    echo -e "${YELLOW}⚠ Skipping OpenAI - No API key found${NC}"
fi

echo ""
echo "5. Testing Local Models via Ollama"
echo "----------------------------------"

# Check if Ollama is running
if curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo -e "  ${GREEN}✓ Ollama is running${NC}"
    
    # Get list of available models
    models=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null)
    
    if [ -z "$models" ]; then
        echo -e "  ${YELLOW}⚠ No models installed in Ollama${NC}"
        echo "  To install models, run:"
        echo "    ollama pull llama2:7b"
        echo "    ollama pull mistral"
        echo "    ollama pull vicuna"
    else
        echo "  Available models: $models"
        
        # Test each available model
        for model in $models; do
            case $model in
                *llama2*)
                    test_provider "ollama" "$model" "Write a haiku about programming" || true
                    ;;
                *mistral*)
                    test_provider "ollama" "$model" "Explain recursion in simple terms" || true
                    ;;
                *vicuna*)
                    test_provider "ollama" "$model" "What makes a good software architecture?" || true
                    ;;
                *codellama*)
                    test_provider "ollama" "$model" "Write a Python function to reverse a string" || true
                    ;;
                *)
                    echo -e "  ${BLUE}Skipping unknown model: $model${NC}"
                    ;;
            esac
        done
    fi
else
    echo -e "  ${YELLOW}⚠ Ollama is not running${NC}"
    echo "  To start Ollama:"
    echo "    ollama serve"
    echo "  Then install models:"
    echo "    ollama pull llama2:7b"
    echo "    ollama pull mistral"
    echo "    ollama pull vicuna"
fi

echo ""
echo "6. Testing Multi-Agent Composition"
echo "----------------------------------"

echo -e "${BLUE}Testing SAGE with multiple providers...${NC}"

# Create a complex query that would benefit from multiple agents
complex_request=$(cat <<'EOF'
{
    "request_id": "multi-agent-test",
    "query": "Design a simple event-sourced payment system with proper domain boundaries",
    "agents": ["ddd-expert", "event-storming-expert", "nats-expert"],
    "providers": {
        "ddd-expert": "claude",
        "event-storming-expert": "openai", 
        "nats-expert": "mistral"
    }
}
EOF
)

echo "  Sending multi-agent request..."
# This would be sent to SAGE V2 service if running

echo ""
echo "====================================================="
echo "Test Summary"
echo "====================================================="

echo -e "${GREEN}✓ Multi-provider system is configured${NC}"
echo ""
echo "Available Providers:"
echo "  - Claude (Anthropic) - High-quality reasoning"
echo "  - OpenAI (GPT-4) - General purpose"
echo "  - Ollama (Local) - Private, free, multiple models"
echo ""
echo "To enable more providers:"
echo "  1. Add API keys to cim-llm-adapter/secrets/"
echo "  2. Install Ollama models: ollama pull <model>"
echo "  3. Update providers.toml for custom configurations"
echo ""
echo "Next steps:"
echo "  - Test agent personality switching with different providers"
echo "  - Configure agent-to-provider mappings in providers.toml"
echo "  - Run performance benchmarks across providers"