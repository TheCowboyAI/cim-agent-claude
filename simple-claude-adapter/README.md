# Simple Claude NATS Adapter

A minimal implementation of the Claude API to NATS adapter for testing on your local NATS server.

## Quick Start

### 1. Set up your NATS server

Make sure you have NATS server with JetStream running on localhost:4222:

```bash
# If you have nats-server installed:
nats-server -js

# Or using Docker:
docker run -d --name nats -p 4222:4222 -p 8222:8222 nats:2.10-alpine -js -m 8222
```

### 2. Set your Claude API key

```bash
export CLAUDE_API_KEY="your-claude-api-key-here"
```

### 3. Run the adapter

```bash
cd simple-claude-adapter
cargo run
```

You should see:
```
Starting Simple Claude NATS Adapter
Connecting to NATS at: localhost:4222
Setting up NATS streams
NATS streams ready
Started consumer, listening for commands on claude.cmd.*
```

### 4. Send a test command

In another terminal, use the NATS CLI to send a test command:

```bash
# Install NATS CLI if you don't have it
# go install github.com/nats-io/natscli/nats@latest

# Send a command
nats pub claude.cmd.test.prompt '{
  "command_id": "cmd-123",
  "correlation_id": "corr-456", 
  "prompt": "Hello Claude! Tell me a joke about programming.",
  "timestamp": "2024-01-01T12:00:00Z"
}'
```

### 5. Listen for responses

```bash
# Subscribe to all Claude events
nats sub "claude.event.*" 

# Or subscribe to specific response events
nats sub "claude.event.*.response"
```

## What It Does

This simple adapter:

1. **Connects to NATS** at localhost:4222
2. **Creates JetStream streams**:
   - `CLAUDE_COMMANDS` for incoming commands
   - `CLAUDE_EVENTS` for outgoing events
3. **Listens for commands** on `claude.cmd.*` subjects
4. **Sends prompts to Claude API** using the official Anthropic API
5. **Publishes responses** as events on `claude.event.{conversation_id}.response`

## Message Format

### Commands (sent to `claude.cmd.*`)
```json
{
  "command_id": "unique-command-id",
  "correlation_id": "correlation-id-for-tracking",
  "prompt": "Your prompt text here",
  "conversation_id": "optional-conversation-id", 
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Response Events (published to `claude.event.{conversation_id}.response`)
```json
{
  "event_id": "unique-event-id",
  "correlation_id": "same-correlation-id-from-command",
  "event_type": "response_received",
  "data": {
    "response_id": "unique-response-id",
    "correlation_id": "correlation-id",
    "content": "Claude's response text",
    "conversation_id": "conversation-id",
    "timestamp": "2024-01-01T12:00:01Z"
  },
  "timestamp": "2024-01-01T12:00:01Z"
}
```

## Configuration

Environment variables:
- `CLAUDE_API_KEY` (required): Your Claude API key
- `NATS_URL` (optional): NATS server URL, defaults to "localhost:4222"

## Testing with Multiple Commands

```bash
# Send multiple prompts
nats pub claude.cmd.session1.prompt '{"command_id":"cmd-1","correlation_id":"corr-1","prompt":"What is Rust?","timestamp":"2024-01-01T12:00:00Z"}'

nats pub claude.cmd.session1.prompt '{"command_id":"cmd-2","correlation_id":"corr-2","prompt":"Explain async/await","timestamp":"2024-01-01T12:01:00Z"}'

nats pub claude.cmd.session2.prompt '{"command_id":"cmd-3","correlation_id":"corr-3","prompt":"Tell me about NATS","timestamp":"2024-01-01T12:02:00Z"}'
```

## Monitoring

You can monitor the NATS streams:

```bash
# Check stream status
nats stream info CLAUDE_COMMANDS
nats stream info CLAUDE_EVENTS

# Check consumer status  
nats consumer info CLAUDE_COMMANDS simple-claude-processor

# View recent messages
nats stream view CLAUDE_COMMANDS
nats stream view CLAUDE_EVENTS
```

## Architecture

This is a simplified version of the full hexagonal architecture. It demonstrates:

- ✅ **NATS Integration**: JetStream streams and consumers
- ✅ **Claude API**: Direct HTTP integration with Anthropic's API
- ✅ **Event-Driven**: Commands in, events out
- ✅ **Correlation Tracking**: Full request/response correlation
- ✅ **Async Processing**: Non-blocking message processing

Missing from this simple version (but present in the full implementation):
- Domain aggregates and complex business rules
- State management and conversation tracking
- Circuit breakers and advanced error handling
- Comprehensive observability and metrics
- Production-grade security and rate limiting

This simple adapter is perfect for testing the basic NATS ↔ Claude API integration on your local development environment!