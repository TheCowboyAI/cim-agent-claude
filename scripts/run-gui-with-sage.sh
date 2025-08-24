#!/usr/bin/env bash
# Run the GUI connected to real SAGE service

echo "🎭 Starting CIM Claude GUI with real SAGE backend..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if NATS is running
if ! nc -z localhost 4222 2>/dev/null; then
    echo "❌ NATS server not running on localhost:4222"
    echo "   Please start NATS with: nats-server -js"
    exit 1
fi

# Check if SAGE service is running
if ! nc -z localhost 8080 2>/dev/null; then
    echo "⚠️  SAGE service may not be running on localhost:8080"
    echo "   Make sure sage-service is running"
fi

echo "✅ NATS server detected on localhost:4222"
echo "🚀 Starting GUI..."
echo ""

# Run GUI in Nix environment with display settings
nix develop --command bash -c "VK_INSTANCE_LAYERS='' cargo run -p cim-claude-gui --bin cim-claude-gui"