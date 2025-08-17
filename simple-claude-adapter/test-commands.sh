#!/bin/bash

# Test script for the simple Claude NATS adapter

echo "🧪 Testing Simple Claude NATS Adapter"
echo "======================================"

# Check if NATS CLI is available
if ! command -v nats &> /dev/null; then
    echo "❌ NATS CLI not found. Install it with:"
    echo "   go install github.com/nats-io/natscli/nats@latest"
    exit 1
fi

echo "✅ NATS CLI found"

# Check if NATS server is running
if ! nats server check connection >/dev/null 2>&1; then
    echo "❌ NATS server not running at localhost:4222"
    echo "   Start it with: nats-server -js"
    echo "   Or with Docker: docker run -d --name nats -p 4222:4222 -p 8222:8222 nats:2.10-alpine -js -m 8222"
    exit 1
fi

echo "✅ NATS server is running"

# Check streams (they should be created by the adapter)
echo ""
echo "📊 Checking NATS streams..."
nats stream ls

echo ""
echo "🚀 Sending test commands..."

# Test command 1: Simple greeting
echo "1️⃣  Sending greeting command..."
nats pub claude.cmd.test.prompt '{
  "command_id": "cmd-001",
  "correlation_id": "corr-001", 
  "prompt": "Hello! Please respond with a brief greeting.",
  "timestamp": "'$(date -Iseconds)'"
}' --count=1

sleep 2

# Test command 2: Programming joke
echo "2️⃣  Sending joke request..."
nats pub claude.cmd.test.prompt '{
  "command_id": "cmd-002", 
  "correlation_id": "corr-002",
  "prompt": "Tell me a short programming joke.",
  "timestamp": "'$(date -Iseconds)'"
}' --count=1

sleep 2

# Test command 3: Technical question
echo "3️⃣  Sending technical question..."
nats pub claude.cmd.test.prompt '{
  "command_id": "cmd-003",
  "correlation_id": "corr-003", 
  "prompt": "In one sentence, what is NATS messaging?",
  "timestamp": "'$(date -Iseconds)'"
}' --count=1

echo ""
echo "📨 Commands sent! Check the adapter logs to see processing."
echo ""
echo "🔍 To monitor responses, run in another terminal:"
echo "   nats sub 'claude.event.*' --translate-jq '.data.content'"
echo ""
echo "📈 To check stream status:"
echo "   nats stream info CLAUDE_COMMANDS"
echo "   nats stream info CLAUDE_EVENTS"
echo ""
echo "✨ Test complete!"