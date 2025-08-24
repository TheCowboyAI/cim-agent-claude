#!/usr/bin/env bash

echo "Testing Multi-Provider Support"
echo "=============================="
echo ""

# Test Claude
echo "1. Testing Claude provider..."
REQUEST_JSON=$(cat <<EOF
{
  "request_id": "test-claude-$(date +%s)",
  "provider": "claude",
  "messages": [{"role": "user", "content": "What is 2+2?"}],
  "context": {
    "session_id": "test-session",
    "user_id": null,
    "conversation_history": [],
    "metadata": {},
    "created_at": "2025-08-24T16:00:00Z",
    "updated_at": "2025-08-24T16:00:00Z"
  },
  "options": {"max_tokens": 50}
}
EOF
)

echo "$REQUEST_JSON" | nats pub cim.llm.commands.request -
sleep 2

echo ""
echo "2. Testing OpenAI provider..."
REQUEST_JSON=$(cat <<EOF
{
  "request_id": "test-openai-$(date +%s)",
  "provider": "openai",
  "messages": [{"role": "user", "content": "What is 3+3?"}],
  "context": {
    "session_id": "test-session",
    "user_id": null,
    "conversation_history": [],
    "metadata": {},
    "created_at": "2025-08-24T16:00:00Z",
    "updated_at": "2025-08-24T16:00:00Z"
  },
  "options": {"max_tokens": 50, "model": "gpt-4-turbo-preview"}
}
EOF
)

echo "$REQUEST_JSON" | nats pub cim.llm.commands.request -
sleep 2

echo ""
echo "Check the LLM adapter logs to see the results!"