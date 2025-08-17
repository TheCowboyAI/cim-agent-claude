#!/bin/bash

# Real test script for Claude API integration
# This will test the actual Claude API connection

echo "🧪 Testing REAL Claude API Integration"
echo "====================================="

# Check if CLAUDE_API_KEY is set
if [[ -z "$CLAUDE_API_KEY" ]]; then
    echo "❌ CLAUDE_API_KEY environment variable is not set!"
    echo "   Set it with: export CLAUDE_API_KEY=\"your-actual-api-key\""
    exit 1
fi

echo "✅ Claude API key is set (${CLAUDE_API_KEY:0:8}...)"

# Check if NATS CLI is available
if ! command -v nats &> /dev/null; then
    echo "❌ NATS CLI not found. Install it with:"
    echo "   go install github.com/nats-io/natscli/nats@latest"
    exit 1
fi

echo "✅ NATS CLI found"

# Check if NATS server is running
if ! nats server check &> /dev/null; then
    echo "❌ NATS server not running at localhost:4222"
    echo "   Start it with: nats-server -js"
    exit 1
fi

echo "✅ NATS server is running"

echo ""
echo "🚀 Sending REAL test to Claude API..."
echo "   (Make sure the adapter is running in another terminal)"
echo ""

# Send a real test command
echo "📝 Sending: 'Hello Claude! Please respond with exactly: NATS_TEST_SUCCESS'"

nats pub claude.cmd.realtest.prompt '{
  "command_id": "real-test-001",
  "correlation_id": "real-corr-001", 
  "prompt": "Hello Claude! Please respond with exactly: NATS_TEST_SUCCESS",
  "timestamp": "'$(date -Iseconds)'"
}' --count=1

echo ""
echo "⏳ Waiting 5 seconds for Claude to respond..."
sleep 5

echo ""
echo "📨 Checking for responses..."

# Subscribe and wait for a response
echo "🔍 Looking for response events..."
timeout 10s nats sub "claude.event.*" --translate-jq '.data.content' --count=1 | head -1

echo ""
echo "💡 If you see 'NATS_TEST_SUCCESS' above, the integration is working!"
echo "   If not, check the adapter logs for errors."

echo ""
echo "🔧 To debug:"
echo "   1. Check adapter logs for HTTP errors"
echo "   2. Verify your Claude API key is valid"
echo "   3. Check NATS stream status:"
echo "      nats stream info CLAUDE_COMMANDS"
echo "      nats stream info CLAUDE_EVENTS"

echo ""
echo "✨ Test complete!"