#!/bin/bash
# Start Universal Agent System
# This script starts all components of the Universal Agent System

set -e

echo "🚀 Starting Universal Agent System"
echo "=================================="

# Check for API key
if [ ! -f "cim-claude-adapter/secrets/claude.api.key" ]; then
    echo "❌ Error: Claude API key not found at cim-claude-adapter/secrets/claude.api.key"
    echo "Please create the file with your API key"
    exit 1
fi

# Export API key
export ANTHROPIC_API_KEY=$(cat cim-claude-adapter/secrets/claude.api.key | tr -d '\n')
echo "✅ API key loaded"

# Check if NATS is running
if ! pgrep -f "nats-server" > /dev/null; then
    echo "⚠️  NATS server not running. Please start it with:"
    echo "   nats-server -js"
    exit 1
fi
echo "✅ NATS server detected"

# Kill any existing services
echo "🔄 Stopping any existing services..."
pkill -f "llm-adapter-service" 2>/dev/null || true
pkill -f "sage_service_v2" 2>/dev/null || true
sleep 2

# Start LLM Adapter Service
echo "1️⃣ Starting LLM Adapter Service..."
nix develop --command cargo run -p cim-llm-adapter --bin llm-adapter-service 2>&1 | sed 's/^/[LLM] /' &
LLM_PID=$!
echo "   PID: $LLM_PID"
sleep 3

# Check if LLM adapter started
if ! pgrep -f "llm-adapter-service" > /dev/null; then
    echo "❌ Failed to start LLM Adapter Service"
    exit 1
fi
echo "✅ LLM Adapter Service started"

# Start SAGE V2 Service
echo "2️⃣ Starting SAGE V2 Service..."
nix develop --command cargo run --bin sage_service_v2 2>&1 | sed 's/^/[SAGE] /' &
SAGE_PID=$!
echo "   PID: $SAGE_PID"
sleep 5

# Check if SAGE started
if ! pgrep -f "sage_service_v2" > /dev/null; then
    echo "❌ Failed to start SAGE V2 Service"
    kill $LLM_PID 2>/dev/null
    exit 1
fi
echo "✅ SAGE V2 Service started"

echo ""
echo "🎉 Universal Agent System is running!"
echo "====================================="
echo ""
echo "Services:"
echo "  LLM Adapter:  PID $LLM_PID"
echo "  SAGE V2:      PID $SAGE_PID"
echo ""
echo "NATS Subjects:"
echo "  Requests:  $(hostname).commands.sage.request"
echo "  Responses: $(hostname).events.sage.response.*"
echo "  LLM:       cim.llm.commands.request"
echo ""
echo "Available Agents: 19"
echo "  Use 'nats pub' to send requests"
echo "  Use 'nats sub' to monitor responses"
echo ""
echo "To stop all services:"
echo "  pkill -f llm-adapter-service"
echo "  pkill -f sage_service_v2"
echo ""
echo "Press Ctrl+C to stop all services..."

# Wait for Ctrl+C
trap "echo ''; echo '🛑 Stopping services...'; kill $LLM_PID $SAGE_PID 2>/dev/null; exit 0" INT TERM

# Keep script running
while true; do
    sleep 1
    # Check if services are still running
    if ! pgrep -f "llm-adapter-service" > /dev/null; then
        echo "⚠️  LLM Adapter Service stopped unexpectedly"
        kill $SAGE_PID 2>/dev/null
        exit 1
    fi
    if ! pgrep -f "sage_service_v2" > /dev/null; then
        echo "⚠️  SAGE V2 Service stopped unexpectedly"
        kill $LLM_PID 2>/dev/null
        exit 1
    fi
done