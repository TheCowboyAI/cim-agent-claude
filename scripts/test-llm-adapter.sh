#!/usr/bin/env bash

# Script to test the CIM LLM Adapter Service
# Sends test requests via NATS and verifies responses

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧪 CIM LLM Adapter Test Suite${NC}"
echo "================================"

# Check if NATS CLI is available
if ! command -v nats &> /dev/null; then
    echo -e "${RED}❌ NATS CLI is required but not found${NC}"
    echo "Install with: go install github.com/nats-io/natscli/nats@latest"
    exit 1
fi

# Check if service is running
echo -e "\n${YELLOW}🔍 Checking LLM Adapter Service...${NC}"
if ! nats req "cim.llm.health" "" --timeout=2s 2>/dev/null; then
    echo -e "${RED}❌ LLM Adapter Service is not responding${NC}"
    echo "Start it with: ./scripts/run-llm-adapter.sh"
    exit 1
fi
echo -e "${GREEN}✅ Service is responding${NC}"

# Test 1: Provider Registration
echo -e "\n${BLUE}Test 1: Provider Registration${NC}"
echo "--------------------------------"
REQUEST_ID=$(uuidgen || echo "test-$(date +%s)")

cat <<EOF > /tmp/register-provider.json
{
  "request_id": "$REQUEST_ID",
  "provider_type": "claude",
  "config": {
    "api_key": "$ANTHROPIC_API_KEY",
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 1000
  }
}
EOF

echo "Registering Claude provider..."
RESPONSE=$(nats req "cim.llm.provider.register" "$(cat /tmp/register-provider.json)" --timeout=5s 2>/dev/null || echo "FAILED")

if [[ "$RESPONSE" == "FAILED" ]]; then
    echo -e "${RED}❌ Provider registration failed${NC}"
else
    echo -e "${GREEN}✅ Provider registered successfully${NC}"
    PROVIDER_ID=$(echo "$RESPONSE" | jq -r '.provider_id' 2>/dev/null || echo "unknown")
    echo "Provider ID: $PROVIDER_ID"
fi

# Test 2: Start Conversation
echo -e "\n${BLUE}Test 2: Start Conversation${NC}"
echo "----------------------------"
CONVERSATION_ID=$(uuidgen || echo "conv-$(date +%s)")

cat <<EOF > /tmp/start-conversation.json
{
  "request_id": "$(uuidgen || echo "req-$(date +%s)")",
  "conversation_id": "$CONVERSATION_ID",
  "provider_id": "$PROVIDER_ID",
  "initial_context": {
    "system_prompt": "You are a helpful AI assistant testing the CIM LLM Adapter."
  }
}
EOF

echo "Starting conversation..."
RESPONSE=$(nats req "cim.llm.dialog.start" "$(cat /tmp/start-conversation.json)" --timeout=5s 2>/dev/null || echo "FAILED")

if [[ "$RESPONSE" == "FAILED" ]]; then
    echo -e "${RED}❌ Conversation start failed${NC}"
else
    echo -e "${GREEN}✅ Conversation started${NC}"
    echo "Conversation ID: $CONVERSATION_ID"
fi

# Test 3: Send Message
echo -e "\n${BLUE}Test 3: Send Message${NC}"
echo "----------------------"

cat <<EOF > /tmp/send-message.json
{
  "request_id": "$(uuidgen || echo "msg-$(date +%s)")",
  "conversation_id": "$CONVERSATION_ID",
  "message": {
    "role": "user",
    "content": "Hello! Can you confirm you're working through the CIM LLM Adapter?"
  }
}
EOF

echo "Sending test message..."
RESPONSE=$(nats req "cim.llm.dialog.turn.request" "$(cat /tmp/send-message.json)" --timeout=10s 2>/dev/null || echo "FAILED")

if [[ "$RESPONSE" == "FAILED" ]]; then
    echo -e "${RED}❌ Message send failed${NC}"
else
    echo -e "${GREEN}✅ Message sent and response received${NC}"
    ASSISTANT_RESPONSE=$(echo "$RESPONSE" | jq -r '.message.content' 2>/dev/null || echo "Could not parse response")
    echo -e "${BLUE}Assistant:${NC} $ASSISTANT_RESPONSE"
fi

# Test 4: Check Context Preservation
echo -e "\n${BLUE}Test 4: Context Preservation${NC}"
echo "------------------------------"

cat <<EOF > /tmp/followup-message.json
{
  "request_id": "$(uuidgen || echo "followup-$(date +%s)")",
  "conversation_id": "$CONVERSATION_ID",
  "message": {
    "role": "user",
    "content": "What did I just ask you about?"
  }
}
EOF

echo "Sending follow-up to test context..."
RESPONSE=$(nats req "cim.llm.dialog.turn.request" "$(cat /tmp/followup-message.json)" --timeout=10s 2>/dev/null || echo "FAILED")

if [[ "$RESPONSE" == "FAILED" ]]; then
    echo -e "${RED}❌ Follow-up message failed${NC}"
else
    echo -e "${GREEN}✅ Follow-up sent and response received${NC}"
    ASSISTANT_RESPONSE=$(echo "$RESPONSE" | jq -r '.message.content' 2>/dev/null || echo "Could not parse response")
    echo -e "${BLUE}Assistant:${NC} $ASSISTANT_RESPONSE"
    
    # Check if response references the previous question
    if [[ "$ASSISTANT_RESPONSE" == *"CIM LLM Adapter"* ]] || [[ "$ASSISTANT_RESPONSE" == *"adapter"* ]]; then
        echo -e "${GREEN}✅ Context was preserved! Assistant remembered the previous question.${NC}"
    else
        echo -e "${YELLOW}⚠️  Context may not be fully preserved${NC}"
    fi
fi

# Test 5: List Active Conversations
echo -e "\n${BLUE}Test 5: List Active Conversations${NC}"
echo "-----------------------------------"

echo "Checking KV store for active conversations..."
CONVERSATIONS=$(nats kv get cim-llm-contexts "$CONVERSATION_ID" 2>/dev/null || echo "NONE")

if [[ "$CONVERSATIONS" != "NONE" ]]; then
    echo -e "${GREEN}✅ Conversation context stored in KV${NC}"
else
    echo -e "${YELLOW}⚠️  Could not retrieve conversation from KV store${NC}"
fi

# Summary
echo -e "\n${BLUE}📊 Test Summary${NC}"
echo "================"
echo -e "${GREEN}✅ Tests completed${NC}"
echo ""
echo "Next steps:"
echo "1. Check the service logs for any errors"
echo "2. Try the universal GUI with: cargo run --bin universal-gui"
echo "3. Test agent switching and orchestration"

# Cleanup
rm -f /tmp/register-provider.json /tmp/start-conversation.json /tmp/send-message.json /tmp/followup-message.json