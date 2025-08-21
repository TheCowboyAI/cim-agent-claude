#!/bin/bash
# Test SAGE with Claude API integration

echo "🎭 Testing SAGE with Claude API integration"
echo "============================================"

# Check if CLAUDE_API_KEY is set
if [ -z "$CLAUDE_API_KEY" ]; then
    echo "ℹ️ CLAUDE_API_KEY not set. This will test in mock mode."
    echo "To test with real Claude API, export your API key:"
    echo "export CLAUDE_API_KEY=your_api_key_here"
    echo ""
fi

echo "Starting SAGE service in background..."
cargo run --bin sage-standalone-test &
SAGE_PID=$!

# Wait for service to start
sleep 3

echo "Testing SAGE service..."
# Test a simple query that should route to cim-expert
echo "📤 Testing CIM expert query..."
cargo run --bin test-client 2>/dev/null | grep -A 20 "🎭 SAGE RESPONSE:" | head -15

# Clean up
echo ""
echo "Stopping SAGE service..."
kill $SAGE_PID 2>/dev/null
wait $SAGE_PID 2>/dev/null

echo "✅ SAGE Claude API integration test complete!"