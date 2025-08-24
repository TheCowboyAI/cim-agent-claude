#!/usr/bin/env bash
# Load Claude API key from secrets file

API_KEY_FILE="./cim-claude-adapter/secrets/claude.api.key"

if [ -f "$API_KEY_FILE" ]; then
    export ANTHROPIC_API_KEY=$(cat "$API_KEY_FILE")
    echo "✅ Claude API key loaded from $API_KEY_FILE"
else
    echo "❌ API key file not found at $API_KEY_FILE"
    exit 1
fi