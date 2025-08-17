#!/bin/bash

# Test the Claude NATS service
# This just sends messages and checks for responses with same correlation ID

echo "🧪 Testing Claude NATS Service"
echo "=============================="

# Generate a unique correlation ID for this test
CORRELATION_ID="test-$(date +%s)-$(printf "%04d" $RANDOM)"

echo "📊 Test correlation ID: $CORRELATION_ID"

# Send test command
echo "📤 Sending test command..."
nats pub claude.cmd.test.prompt "{
  \"command_id\": \"test-cmd-$(date +%s)\",
  \"correlation_id\": \"$CORRELATION_ID\",
  \"prompt\": \"Please respond with: TEST_SUCCESS_$CORRELATION_ID\",
  \"timestamp\": \"$(date -Iseconds)\"
}"

echo "⏳ Waiting for response with correlation ID: $CORRELATION_ID"

# Wait for response with matching correlation ID
echo "👂 Listening for matching response..."
timeout 15s nats sub "claude.event.*" --count=1 | grep -q "$CORRELATION_ID"

if [ $? -eq 0 ]; then
    echo "✅ SUCCESS: Received response with matching correlation ID!"
    echo "📄 Full response:"
    nats sub "claude.event.*" --count=1 --translate-jq "select(.correlation_id == \"$CORRELATION_ID\") | .data.content" 2>/dev/null | head -1
else
    echo "❌ TIMEOUT: No response received within 15 seconds"
    echo "💡 Check:"
    echo "   - Is the service running? sudo systemctl status claude-adapter"
    echo "   - Are there errors? sudo journalctl -u claude-adapter -n 20"
    echo "   - Is Claude API key valid?"
fi

echo ""
echo "🔍 Service status:"
sudo systemctl is-active claude-adapter || echo "Service not running"