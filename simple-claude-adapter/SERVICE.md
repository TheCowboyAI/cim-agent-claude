# Claude NATS Service

A proper systemd service that forwards NATS queries to Claude API and returns responses as events with preserved correlation IDs.

## What It Does

**Simple**: Listens on NATS → Forwards to Claude → Returns events

```
NATS Query → Service → Claude API → Service → NATS Event
(correlation_id: ABC)                        (correlation_id: ABC)
```

## Installation

```bash
cd simple-claude-adapter

# 1. Install as systemd service
./install-service.sh

# 2. Set your Claude API key
sudo nano /etc/claude-adapter/config
# Change: CLAUDE_API_KEY=sk-your-actual-key-here

# 3. Start the service
sudo systemctl start claude-adapter
sudo systemctl enable claude-adapter  # auto-start on boot
```

## Usage

**Send queries:**
```bash
nats pub claude.cmd.any.prompt '{
  "command_id": "unique-id",
  "correlation_id": "your-correlation-id",
  "prompt": "Your question to Claude",
  "timestamp": "'$(date -Iseconds)'"
}'
```

**Listen for responses:**
```bash
# All responses
nats sub "claude.event.*"

# Just the content
nats sub "claude.event.*" --translate-jq '.data.content'

# Specific correlation ID
nats sub "claude.event.*" --translate-jq 'select(.correlation_id == "your-correlation-id")'
```

## Test It

```bash
# Quick test
./test-service.sh

# Should show:
# ✅ SUCCESS: Received response with matching correlation ID!
```

## Management

```bash
# Status
sudo systemctl status claude-adapter

# Logs
sudo journalctl -u claude-adapter -f

# Restart
sudo systemctl restart claude-adapter

# Stop
sudo systemctl stop claude-adapter
```

## Message Format

**Input (NATS command):**
```json
{
  "command_id": "cmd-123",
  "correlation_id": "corr-456",
  "prompt": "What is NATS?",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

**Output (NATS event):**
```json
{
  "event_id": "event-789",
  "correlation_id": "corr-456",
  "event_type": "response_received",
  "data": {
    "response_id": "resp-abc",
    "correlation_id": "corr-456",
    "content": "NATS is a messaging system...",
    "conversation_id": "conv-def",
    "timestamp": "2024-01-01T12:00:01Z"
  },
  "timestamp": "2024-01-01T12:00:01Z"
}
```

**Key Point**: `correlation_id` is preserved throughout the entire flow.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│ Your App        │    │ Claude Service  │    │ Claude API      │
│                 │    │                 │    │                 │
│ nats pub        ├───►│ Listen NATS     ├───►│ HTTP Request    │
│ claude.cmd.*    │    │ correlation_id  │    │                 │
│                 │    │                 │    │                 │
│ nats sub        │◄───┤ Publish Event   │◄───┤ HTTP Response   │
│ claude.event.*  │    │ correlation_id  │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

**No interactive chat, no shell scripts** - just a service that handles the forwarding automatically.

## Production Ready

✅ **Systemd service** - starts automatically, managed by system
✅ **Proper logging** - logs to journald
✅ **Service user** - runs as non-privileged `claude-adapter` user  
✅ **Configuration** - API key stored securely in `/etc/claude-adapter/config`
✅ **Correlation tracking** - preserves correlation IDs throughout
✅ **Error handling** - robust error handling with retries