#!/bin/bash
# Script to securely load Claude API key for SAGE service
# This script reads the API key from the secure location and sets the environment variable

API_KEY_FILE="/home/steele/.config/claude/api-key"

if [[ ! -f "$API_KEY_FILE" ]]; then
    echo "❌ API key file not found: $API_KEY_FILE"
    echo "Please create the file and add your Claude API key."
    exit 1
fi

if [[ ! -s "$API_KEY_FILE" ]]; then
    echo "❌ API key file is empty: $API_KEY_FILE"
    echo "Please add your Claude API key to the file."
    echo "Example: echo 'sk-ant-api03-...' > $API_KEY_FILE"
    exit 1
fi

# Read the API key (remove any whitespace)
ANTHROPIC_API_KEY=$(tr -d '[:space:]' < "$API_KEY_FILE")

if [[ -z "$ANTHROPIC_API_KEY" ]]; then
    echo "❌ Failed to read API key from file"
    exit 1
fi

# Validate API key format (basic check)
if [[ ! "$ANTHROPIC_API_KEY" =~ ^sk-ant-api03- ]]; then
    echo "⚠️  Warning: API key doesn't match expected format (sk-ant-api03-...)"
    echo "Proceeding anyway..."
fi

# Export the environment variable
export ANTHROPIC_API_KEY

echo "✅ Claude API key loaded successfully"
echo "🔒 Key format: ${ANTHROPIC_API_KEY:0:15}...${ANTHROPIC_API_KEY: -4}"