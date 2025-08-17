# Testing the Claude API to NATS Adapter

This guide will help you test the **REAL** Claude API integration on your local NATS server.

## Prerequisites

1. **NATS Server with JetStream** running on localhost:4222
2. **Valid Claude API Key** from Anthropic
3. **NATS CLI** for sending test messages

## Step 1: Start NATS Server

```bash
# Option 1: Direct NATS server
nats-server -js

# Option 2: Docker
docker run -d --name nats -p 4222:4222 -p 8222:8222 nats:2.10-alpine -js -m 8222
```

Verify it's running:
```bash
nats server check
# Should return: OK NATS server is running
```

## Step 2: Set Your Claude API Key

```bash
export CLAUDE_API_KEY="sk-your-actual-api-key-from-anthropic"
```

⚠️ **Important**: Use your REAL API key from https://console.anthropic.com/

## Step 3: Start the Adapter

```bash
cd simple-claude-adapter
RUST_LOG=info cargo run
```

You should see:
```
Starting Simple Claude NATS Adapter
Using Claude API key: sk-ant-api03...
Connecting to NATS at: localhost:4222
Setting up NATS streams
NATS streams ready
Started consumer, listening for commands on claude.cmd.*
```

## Step 4: Test Real Claude Integration

### Quick Test
```bash
./test-real-claude.sh
```

This will:
1. ✅ Verify your API key format
2. ✅ Send a test message to Claude
3. ✅ Wait for the response
4. ✅ Show you the result

### Manual Test
```bash
# Send a command
nats pub claude.cmd.test.prompt '{
  "command_id": "test-001",
  "correlation_id": "corr-001", 
  "prompt": "Hello Claude! Please say hello back and mention NATS.",
  "timestamp": "'$(date -Iseconds)'"
}'

# Listen for responses (in another terminal)
nats sub "claude.event.*" --translate-jq '.data.content'
```

### Expected Response

You should see something like:
```
Hello! Nice to meet you. I see you're testing a NATS messaging system - that's a great choice for building distributed applications with reliable message passing!
```

## Step 5: Monitor and Debug

### Check Stream Status
```bash
# View stream information
nats stream info CLAUDE_COMMANDS
nats stream info CLAUDE_EVENTS

# View recent messages
nats stream view CLAUDE_COMMANDS --count 5
nats stream view CLAUDE_EVENTS --count 5
```

### Check Consumer Status
```bash
nats consumer info CLAUDE_COMMANDS simple-claude-processor
```

### View All Events
```bash
# Raw events
nats sub "claude.event.*"

# Just the content
nats sub "claude.event.*" --translate-jq '.data.content'

# Specific conversation
nats sub "claude.event.YOUR_CONVERSATION_ID.*"
```

## Common Issues and Solutions

### ❌ "Authentication failed - check your CLAUDE_API_KEY"
- Verify your API key starts with `sk-`
- Check it's valid at https://console.anthropic.com/
- Make sure you have Claude API credits

### ❌ "Rate limit exceeded"
- Wait 1 minute between requests
- Claude has rate limits for API usage

### ❌ "NATS server not running"
```bash
# Start NATS with JetStream
nats-server -js

# Or check if it's running on different port
nats server check --server=nats://localhost:4222
```

### ❌ "No response received"
- Check adapter logs for errors
- Verify NATS streams exist: `nats stream ls`
- Check consumer is active: `nats consumer ls CLAUDE_COMMANDS`

## Advanced Testing

### Multiple Conversations
```bash
# Conversation 1
nats pub claude.cmd.session1.prompt '{
  "command_id": "cmd-1",
  "correlation_id": "corr-1",
  "prompt": "What is Rust programming language?",
  "conversation_id": "conv-rust",
  "timestamp": "'$(date -Iseconds)'"
}'

# Conversation 2  
nats pub claude.cmd.session2.prompt '{
  "command_id": "cmd-2", 
  "correlation_id": "corr-2",
  "prompt": "Explain NATS messaging in one sentence.",
  "conversation_id": "conv-nats",
  "timestamp": "'$(date -Iseconds)'"
}'
```

### Load Testing
```bash
# Send multiple messages quickly
for i in {1..5}; do
  nats pub claude.cmd.load.prompt '{
    "command_id": "load-'$i'",
    "correlation_id": "load-corr-'$i'",
    "prompt": "Count to '$i' for me.",
    "timestamp": "'$(date -Iseconds)'"
  }'
  sleep 2
done
```

### Error Testing
```bash
# Test with invalid prompt (empty)
nats pub claude.cmd.error.prompt '{
  "command_id": "error-1",
  "correlation_id": "error-corr-1", 
  "prompt": "",
  "timestamp": "'$(date -Iseconds)'"
}'
```

## Success Indicators

✅ **Working correctly if you see:**
- Adapter logs show "Sending request to Claude API"
- Adapter logs show "Claude API response status: 200"
- Adapter logs show "Extracted content: [Claude's response]"
- Events published to `claude.event.*` subjects
- Response content appears when you subscribe

## Message Flow Verification

```
You → claude.cmd.* → Adapter → Claude API → Adapter → claude.event.* → You
```

Each step should show in the logs:
1. "Received message on subject: claude.cmd.test.prompt"
2. "Processing command: test-001 (correlation: corr-001)"
3. "Sending request to Claude API for correlation: corr-001"
4. "Claude API response status: 200"
5. "Published response for correlation: corr-001"

**Congratulations! If you see Claude's responses, your adapter is successfully translating between NATS and the Claude API! 🎉**