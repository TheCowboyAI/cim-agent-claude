#!/bin/bash
# SAGE Service Startup Script with Claude API Key Loading
# This script properly loads the API key and starts the SAGE service

set -euo pipefail

echo "🎭 Starting SAGE Service with Claude API Integration"
echo "=================================================="

# Load the API key
if ! source "$(dirname "$0")/load-claude-api-key.sh"; then
    echo "❌ Failed to load Claude API key"
    exit 1
fi

# Set NATS URL if not already set
export NATS_URL="${NATS_URL:-nats://localhost:4222}"

echo "🌐 NATS URL: $NATS_URL"
echo "🔑 Claude API: Configured"
echo "🎭 Starting SAGE Service..."
echo

# Check if we're in the correct directory
if [[ ! -f "Cargo.toml" ]]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Start the SAGE service using Nix development environment
exec nix develop --command cargo run --bin sage-service