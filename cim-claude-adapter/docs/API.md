# CIM Claude Adapter - API Reference

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

## Overview

The CIM Claude Adapter provides a NATS-based API for integrating Claude AI into CIM ecosystems. All interactions are message-driven using the command/event pattern with full correlation tracking.

## Message Format Standards

### Common Headers

All messages include standard CIM headers:

```json
{
  "correlation_id": "uuid-v4",
  "timestamp": "ISO-8601-datetime",
  "version": "1.0"
}
```

### Error Format

```json
{
  "error_type": "ErrorCategory",
  "error_code": "SPECIFIC_ERROR_CODE",
  "message": "Human readable error message",
  "details": {},
  "correlation_id": "uuid-v4",
  "timestamp": "ISO-8601-datetime",
  "retry_after": "optional-duration-seconds"
}
```

## Commands

Commands express intent and are sent to specific NATS subjects. All commands return acknowledgment and may trigger multiple events.

### StartConversation

**Subject**: `cim.claude.commands.start`

**Purpose**: Initiate a new conversation with Claude AI.

**Request Schema**:
```json
{
  "session_id": "string (required)",
  "initial_prompt": "string (required, max 50000 chars)",
  "context": {
    "max_tokens": "integer (optional, default: 4000, max: 8000)",
    "temperature": "float (optional, default: 0.7, range: 0.0-1.0)",
    "system_prompt": "string (optional)",
    "metadata": "object (optional)"
  },
  "correlation_id": "string (required, uuid-v4)"
}
```

**Response**: Synchronous acknowledgment + asynchronous `ConversationStarted` event

**Example**:
```bash
nats pub cim.claude.commands.start '{
  "session_id": "user-session-123",
  "initial_prompt": "Explain the principles of Domain-Driven Design",
  "context": {
    "max_tokens": 2000,
    "temperature": 0.8,
    "system_prompt": "You are an expert software architect"
  },
  "correlation_id": "start-conv-001"
}'
```

**Validation Rules**:
- `session_id`: Must be non-empty string
- `initial_prompt`: 1-50,000 characters
- `max_tokens`: 1-8,000 (subject to Claude limits)
- `temperature`: 0.0-1.0
- `correlation_id`: Valid UUID v4

---

### SendPrompt

**Subject**: `cim.claude.commands.send`

**Purpose**: Send additional prompt to existing conversation.

**Request Schema**:
```json
{
  "conversation_id": "string (required, uuid-v4)",
  "prompt": "string (required, max 50000 chars)",
  "correlation_id": "string (required, uuid-v4)"
}
```

**Prerequisites**: Conversation must be in `Active` state

**Example**:
```bash
nats pub cim.claude.commands.send '{
  "conversation_id": "550e8400-e29b-41d4-a716-446655440000",
  "prompt": "Can you provide a code example?",
  "correlation_id": "send-prompt-002"
}'
```

**Validation Rules**:
- `conversation_id`: Must reference existing active conversation
- `prompt`: 1-50,000 characters
- Rate limiting applied per conversation

---

### EndConversation

**Subject**: `cim.claude.commands.end`

**Purpose**: Gracefully terminate a conversation.

**Request Schema**:
```json
{
  "conversation_id": "string (required, uuid-v4)",
  "reason": "enum (required)",
  "correlation_id": "string (required, uuid-v4)"
}
```

**Reason Values**:
- `UserRequested`: User-initiated termination
- `Timeout`: Conversation timeout
- `RateLimitExceeded`: Rate limit violation
- `ApiError`: Claude API error
- `MaxExchangesReached`: Exchange limit hit
- `InvalidState`: Invalid state transition

**Example**:
```bash
nats pub cim.claude.commands.end '{
  "conversation_id": "550e8400-e29b-41d4-a716-446655440000",
  "reason": "UserRequested",
  "correlation_id": "end-conv-003"
}'
```

## Events

Events represent facts that have occurred. They are published to subject hierarchies and can be subscribed to for reactive processing.

### ConversationStarted

**Subject**: `cim.claude.events.conversation_started`

**Purpose**: Confirms successful conversation initiation.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "ConversationStarted",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "session_id": "string",
    "initial_prompt": "string",
    "context": {
      "max_tokens": "integer",
      "temperature": "float",
      "system_prompt": "string",
      "metadata": "object"
    }
  }
}
```

---

### PromptSent

**Subject**: `cim.claude.events.prompt_sent`

**Purpose**: Confirms prompt was sent to Claude API.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "PromptSent",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "prompt": "string",
    "sequence_number": "integer",
    "claude_request_metadata": {
      "model": "string",
      "max_tokens": "integer",
      "temperature": "float",
      "request_timestamp": "string (ISO-8601)"
    }
  }
}
```

---

### ResponseReceived

**Subject**: `cim.claude.events.response_received`

**Purpose**: Delivers Claude's response to the conversation.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "ResponseReceived",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "response": {
      "content": "string",
      "usage": {
        "input_tokens": "integer",
        "output_tokens": "integer",
        "total_tokens": "integer"
      },
      "finish_reason": "string",
      "model": "string"
    },
    "sequence_number": "integer",
    "processing_duration_ms": "integer"
  }
}
```

**Finish Reasons**:
- `stop`: Natural completion
- `length`: Max token limit reached
- `content_filter`: Content policy violation

---

### ConversationEnded

**Subject**: `cim.claude.events.conversation_ended`

**Purpose**: Confirms conversation termination.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "ConversationEnded",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "reason": "enum",
    "total_exchanges": "integer",
    "total_tokens_used": {
      "input_tokens": "integer",
      "output_tokens": "integer",
      "total_tokens": "integer"
    }
  }
}
```

---

### RateLimitExceeded

**Subject**: `cim.claude.events.rate_limit_exceeded`

**Purpose**: Notifies of rate limit violations.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "RateLimitExceeded",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "limit_type": "enum",
    "retry_after_seconds": "integer"
  }
}
```

**Limit Types**:
- `PromptsPerMinute`: Prompt rate limit
- `TokensPerHour`: Token consumption limit
- `ConcurrentRequests`: Concurrent request limit

---

### ClaudeApiErrorOccurred

**Subject**: `cim.claude.events.api_error_occurred`

**Purpose**: Reports Claude API errors.

**Event Schema**:
```json
{
  "event_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)",
  "causation_id": "string (uuid-v4)",
  "event": "ClaudeApiErrorOccurred",
  "timestamp": "string (ISO-8601)",
  "version": 1,
  "data": {
    "conversation_id": "string (uuid-v4)",
    "error_type": "enum",
    "error_message": "string",
    "retry_count": "integer"
  }
}
```

**Error Types**:
- `Authentication`: API key issues
- `RateLimit`: Rate limiting
- `Timeout`: Request timeout
- `ServerError`: Claude server error
- `ValidationError`: Request validation failed
- `NetworkError`: Network connectivity

## Queries

Read-only operations for retrieving conversation state and metrics.

### GetConversationStatus

**Subject**: `cim.claude.queries.conversation_status`

**Request**:
```json
{
  "conversation_id": "string (uuid-v4)",
  "correlation_id": "string (uuid-v4)"
}
```

**Response**:
```json
{
  "conversation_id": "string (uuid-v4)",
  "state": "enum",
  "session_id": "string",
  "created_at": "string (ISO-8601)",
  "last_activity": "string (ISO-8601)",
  "exchange_count": "integer",
  "token_usage": {
    "input_tokens": "integer",
    "output_tokens": "integer",
    "total_tokens": "integer"
  }
}
```

**States**:
- `Active`: Ready for prompts
- `Waiting`: Awaiting Claude response
- `RateLimited`: Temporarily limited
- `Failed`: Error state
- `Ended`: Terminated

---

### GetUsageMetrics

**Subject**: `cim.claude.queries.usage_metrics`

**Request**:
```json
{
  "time_window": "enum (hour|day|week|month)",
  "session_id": "string (optional)",
  "correlation_id": "string (uuid-v4)"
}
```

**Response**:
```json
{
  "time_window": "string",
  "metrics": {
    "conversations_started": "integer",
    "conversations_completed": "integer",
    "total_prompts": "integer",
    "total_tokens": {
      "input_tokens": "integer",
      "output_tokens": "integer",
      "total_tokens": "integer"
    },
    "average_response_time_ms": "float",
    "error_rate": "float",
    "rate_limit_hits": "integer"
  },
  "timestamp": "string (ISO-8601)"
}
```

## Subject Patterns

### Subscription Patterns

```bash
# All Claude events
cim.claude.events.>

# Specific conversation events
cim.claude.events.*.{conversation_id}

# Error events only
cim.claude.events.*error*

# All conversation lifecycle events
cim.claude.events.conversation_*

# Rate limit events
cim.claude.events.rate_limit_exceeded
```

### Filtering Examples

```bash
# Monitor all conversations for a session
nats sub 'cim.claude.events.>' --filter="session_id:user-123"

# Track token usage
nats sub 'cim.claude.events.response_received' --filter="usage.total_tokens:>1000"

# Error monitoring
nats sub 'cim.claude.events.api_error_occurred'
```

## Rate Limits

### Internal Rate Limits

- **Commands per session**: 60/minute
- **Concurrent conversations per session**: 5
- **Prompt size**: 50,000 characters max
- **Conversation lifetime**: 30 minutes idle timeout

### Claude API Limits

The adapter automatically handles Claude API rate limits:

- **Requests per minute**: Varies by plan
- **Tokens per hour**: Varies by plan
- **Concurrent requests**: Usually 5-10

Rate limit information is included in `RateLimitExceeded` events with `retry_after_seconds`.

## Error Handling

### Error Codes

| Code | Description | Recovery |
|------|-------------|----------|
| `CONVERSATION_NOT_FOUND` | Invalid conversation ID | Check ID, start new conversation |
| `INVALID_STATE_TRANSITION` | Command not valid for current state | Wait for state change or end conversation |
| `PROMPT_TOO_LONG` | Prompt exceeds length limit | Reduce prompt size |
| `RATE_LIMIT_EXCEEDED` | Too many requests | Wait for retry_after period |
| `CLAUDE_API_ERROR` | Claude API failure | Check API status, retry later |
| `VALIDATION_FAILED` | Invalid request format | Fix request format |
| `SESSION_EXPIRED` | Session no longer valid | Re-authenticate session |

### Retry Strategies

1. **Exponential Backoff**: For transient errors
   - Initial delay: 1 second
   - Max delay: 60 seconds
   - Max retries: 5

2. **Rate Limit Compliance**: Honor `retry_after_seconds`

3. **Circuit Breaker**: Fail fast during outages
   - Failure threshold: 10 consecutive failures
   - Recovery timeout: 30 seconds

## Health Monitoring

### Health Check Endpoints

- `GET /health`: Liveness probe
- `GET /ready`: Readiness probe
- `GET /metrics`: Prometheus metrics

### Key Metrics

```prometheus
# Conversation metrics
conversations_active{session_id}
conversations_total{session_id,status}
conversation_duration_seconds{session_id}

# API metrics
claude_api_requests_total{method,status}
claude_api_duration_seconds{method}
claude_api_rate_limit_hits_total

# Token metrics
tokens_consumed_total{session_id,type}
tokens_per_request{session_id}

# Error metrics
errors_total{type,code}
```

## Security

### Authentication

- **NATS**: NKey-based authentication
- **Claude API**: Bearer token authentication
- **TLS**: All connections encrypted

### Authorization

- **Session-based**: Commands scoped to session
- **Resource-based**: Conversations owned by sessions
- **Rate limiting**: Per-session quotas

### Data Privacy

- **No persistent storage**: Conversations not stored long-term
- **Audit logging**: All commands and events logged
- **PII handling**: Follow data protection regulations

## SDK Examples

### JavaScript/Node.js

```javascript
import { connect, StringCodec } from 'nats';

const nc = await connect({ servers: 'nats://localhost:4222' });
const sc = StringCodec();

// Start conversation
const startCommand = {
  session_id: 'user-123',
  initial_prompt: 'Hello Claude',
  context: { max_tokens: 1000 },
  correlation_id: crypto.randomUUID()
};

await nc.publish('cim.claude.commands.start', sc.encode(JSON.stringify(startCommand)));

// Listen for events
const sub = nc.subscribe('cim.claude.events.>');
for await (const m of sub) {
  const event = JSON.parse(sc.decode(m.data));
  console.log('Event:', event.event, event.data);
}
```

### Python

```python
import asyncio
import json
import uuid
from nats.aio.client import Client as NATS

async def main():
    nc = NATS()
    await nc.connect('nats://localhost:4222')
    
    # Start conversation
    start_command = {
        'session_id': 'user-123',
        'initial_prompt': 'Hello Claude',
        'context': {'max_tokens': 1000},
        'correlation_id': str(uuid.uuid4())
    }
    
    await nc.publish('cim.claude.commands.start', 
                    json.dumps(start_command).encode())
    
    # Subscribe to events
    async def event_handler(msg):
        event = json.loads(msg.data.decode())
        print(f"Event: {event['event']}", event.get('data', {}))
    
    await nc.subscribe('cim.claude.events.>', cb=event_handler)
    
    # Keep listening
    await asyncio.sleep(60)
    await nc.close()

if __name__ == '__main__':
    asyncio.run(main())
```

### Rust

```rust
use async_nats;
use serde_json;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("nats://localhost:4222").await?;
    
    // Start conversation
    let start_command = serde_json::json!({
        "session_id": "user-123",
        "initial_prompt": "Hello Claude",
        "context": {"max_tokens": 1000},
        "correlation_id": Uuid::new_v4()
    });
    
    client.publish("cim.claude.commands.start", 
                  serde_json::to_vec(&start_command)?).await?;
    
    // Subscribe to events
    let mut subscriber = client.subscribe("cim.claude.events.>").await?;
    while let Some(message) = subscriber.next().await {
        let event: serde_json::Value = serde_json::from_slice(&message.data)?;
        println!("Event: {}", event);
    }
    
    Ok(())
}
```

This API reference provides complete coverage of all available commands, events, queries, and integration patterns for the CIM Claude Adapter.