# CIM Claude Adapter - User Guide

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

## Table of Contents

1. [Introduction](#introduction)
2. [Quick Start](#quick-start)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Usage Examples](#usage-examples)
6. [API Reference](#api-reference)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)
9. [Troubleshooting](#troubleshooting)
10. [Integration Patterns](#integration-patterns)

## Introduction

The CIM Claude Adapter enables seamless integration of Claude AI into your CIM (Composable Information Machine) ecosystem. It provides a NATS-native, event-driven interface for Claude conversations with full observability, error handling, and scalability.

### Key Features

- **Event-Driven Architecture**: All interactions captured as domain events
- **NATS Integration**: Native NATS messaging with JetStream support
- **Conversation Management**: Full conversation lifecycle tracking
- **Rate Limiting**: Automatic Claude API rate limit compliance
- **Error Recovery**: Comprehensive error handling and retry logic
- **Monitoring**: Built-in metrics and tracing
- **Scalability**: Horizontal scaling through NATS clustering

### Prerequisites

- NATS Server with JetStream enabled
- Claude API access and API key
- Rust 1.70+ (for building from source)
- Docker (for containerized deployment)

## Quick Start

### 1. Start NATS Server

```bash
# Using Docker
docker run -p 4222:4222 -p 8222:8222 --name nats-server \
  nats:latest -js

# Or using nats-server binary
nats-server -js
```

### 2. Set Environment Variables

```bash
export CLAUDE_API_KEY="your-claude-api-key"
export NATS_URL="nats://localhost:4222"
```

### 3. Run the Adapter

```bash
# Using Docker
docker run -e CLAUDE_API_KEY=$CLAUDE_API_KEY \
           -e NATS_URL=$NATS_URL \
           --network host \
           cowboy-ai/cim-claude-adapter:latest

# Or from source
cargo run --release
```

### 4. Test with NATS CLI

```bash
# Start a conversation
nats pub cim.claude.commands.start '{
  "session_id": "user-123",
  "initial_prompt": "Hello Claude, explain quantum computing",
  "context": {
    "max_tokens": 1000,
    "temperature": 0.7
  },
  "correlation_id": "req-001"
}'

# Monitor events
nats sub 'cim.claude.events.>'
```

## Installation

### Docker Installation (Recommended)

```bash
# Pull the latest image
docker pull cowboy-ai/cim-claude-adapter:latest

# Run with environment configuration
docker run -d \
  --name cim-claude-adapter \
  --network host \
  -e CLAUDE_API_KEY="$CLAUDE_API_KEY" \
  -e NATS_URL="nats://localhost:4222" \
  -e LOG_LEVEL="info" \
  cowboy-ai/cim-claude-adapter:latest
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cim-claude-adapter
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cim-claude-adapter
  template:
    metadata:
      labels:
        app: cim-claude-adapter
    spec:
      containers:
      - name: adapter
        image: cowboy-ai/cim-claude-adapter:latest
        env:
        - name: CLAUDE_API_KEY
          valueFrom:
            secretKeyRef:
              name: claude-secret
              key: api-key
        - name: NATS_URL
          value: "nats://nats-server:4222"
        ports:
        - containerPort: 8080
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/cowboy-ai/cim-claude-adapter.git
cd cim-claude-adapter

# Build release binary
cargo build --release

# Run
./target/release/cim-claude-adapter
```

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `CLAUDE_API_KEY` | Yes | - | Claude API authentication key |
| `NATS_URL` | No | `nats://localhost:4222` | NATS server connection URL |
| `MAX_PROMPT_LENGTH` | No | `50000` | Maximum prompt character length |
| `MAX_EXCHANGES_PER_CONVERSATION` | No | `100` | Maximum exchanges per conversation |
| `DEFAULT_TEMPERATURE` | No | `0.7` | Default Claude temperature |
| `DEFAULT_MAX_TOKENS` | No | `4000` | Default Claude max tokens |
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | No | `60` | Internal rate limiting |
| `LOG_LEVEL` | No | `info` | Logging level (error, warn, info, debug, trace) |
| `METRICS_PORT` | No | `9090` | Prometheus metrics port |
| `HEALTH_CHECK_PORT` | No | `8080` | Health check endpoint port |

### Configuration File

Create a `config.toml` file for advanced configuration:

```toml
[claude]
api_key = "${CLAUDE_API_KEY}"
model = "claude-3-sonnet-20240229"
max_tokens = 4000
temperature = 0.7

[nats]
url = "nats://localhost:4222"
max_reconnects = 10
reconnect_wait = "5s"

[conversation]
max_prompt_length = 50000
max_exchanges = 100
session_timeout = "30m"

[monitoring]
metrics_port = 9090
health_port = 8080
enable_tracing = true

[logging]
level = "info"
format = "json"
```

## Usage Examples

### Basic Conversation

```rust
use serde_json::json;

// Start conversation
let start_command = json!({
    "session_id": "user-456",
    "initial_prompt": "What are the benefits of Rust programming?",
    "context": {
        "max_tokens": 1500,
        "temperature": 0.8,
        "system_prompt": "You are a helpful programming assistant."
    },
    "correlation_id": "conv-789"
});

// Send via NATS
nats_client.publish("cim.claude.commands.start", start_command).await?;
```

### Multi-Turn Conversation

```rust
// After receiving ConversationStarted event with conversation_id
let follow_up = json!({
    "conversation_id": "conv-uuid-here",
    "prompt": "Can you show me a simple example?",
    "correlation_id": "follow-up-001"
});

nats_client.publish("cim.claude.commands.send", follow_up).await?;
```

### Conversation Management

```rust
// End conversation gracefully
let end_command = json!({
    "conversation_id": "conv-uuid-here",
    "reason": "UserRequested",
    "correlation_id": "end-001"
});

nats_client.publish("cim.claude.commands.end", end_command).await?;
```

### Event Handling

```rust
// Subscribe to all Claude events
let mut subscription = nats_client.subscribe("cim.claude.events.>").await?;

while let Some(message) = subscription.next().await {
    let event: serde_json::Value = serde_json::from_slice(&message.data)?;
    
    match event["event"].as_str() {
        Some("ConversationStarted") => {
            let conv_id = event["conversation_id"].as_str().unwrap();
            println!("New conversation: {}", conv_id);
        },
        Some("ResponseReceived") => {
            let response = &event["response"];
            println!("Claude response: {}", response["content"]);
        },
        Some("ConversationEnded") => {
            println!("Conversation ended");
        },
        _ => {}
    }
}
```

## API Reference

### Commands

#### StartConversation

Initiates a new conversation with Claude.

**Subject**: `cim.claude.commands.start`

**Payload**:
```json
{
  "session_id": "string",
  "initial_prompt": "string",
  "context": {
    "max_tokens": 4000,
    "temperature": 0.7,
    "system_prompt": "string",
    "metadata": {}
  },
  "correlation_id": "string"
}
```

#### SendPrompt

Sends a prompt to an existing conversation.

**Subject**: `cim.claude.commands.send`

**Payload**:
```json
{
  "conversation_id": "string",
  "prompt": "string",
  "correlation_id": "string"
}
```

#### EndConversation

Terminates an existing conversation.

**Subject**: `cim.claude.commands.end`

**Payload**:
```json
{
  "conversation_id": "string",
  "reason": "UserRequested|Timeout|RateLimitExceeded|ApiError|MaxExchangesReached",
  "correlation_id": "string"
}
```

### Events

#### ConversationStarted

Emitted when a new conversation begins.

**Subject**: `cim.claude.events.conversation_started`

**Payload**:
```json
{
  "event_id": "string",
  "correlation_id": "string",
  "causation_id": "string",
  "conversation_id": "string",
  "session_id": "string",
  "initial_prompt": "string",
  "context": {},
  "timestamp": "2025-01-01T00:00:00Z",
  "version": 1
}
```

#### PromptSent

Emitted when a prompt is sent to Claude.

**Subject**: `cim.claude.events.prompt_sent`

#### ResponseReceived

Emitted when Claude responds.

**Subject**: `cim.claude.events.response_received`

**Payload**:
```json
{
  "event_id": "string",
  "conversation_id": "string",
  "response": {
    "content": "string",
    "usage": {
      "input_tokens": 100,
      "output_tokens": 200,
      "total_tokens": 300
    },
    "finish_reason": "stop",
    "model": "claude-3-sonnet-20240229"
  },
  "sequence_number": 1,
  "processing_duration_ms": 1500,
  "timestamp": "2025-01-01T00:00:00Z"
}
```

#### ConversationEnded

Emitted when a conversation terminates.

**Subject**: `cim.claude.events.conversation_ended`

#### RateLimitExceeded

Emitted when rate limits are hit.

**Subject**: `cim.claude.events.rate_limit_exceeded`

#### ClaudeApiErrorOccurred

Emitted when Claude API errors occur.

**Subject**: `cim.claude.events.api_error_occurred`

## Error Handling

### Error Types

1. **Validation Errors**: Invalid input data
2. **State Errors**: Invalid conversation state transitions
3. **Rate Limit Errors**: Claude API rate limiting
4. **Network Errors**: Connectivity issues
5. **API Errors**: Claude API errors

### Error Response Format

```json
{
  "error_type": "ValidationError",
  "error_code": "INVALID_PROMPT_LENGTH",
  "message": "Prompt exceeds maximum length of 50000 characters: 52000",
  "correlation_id": "req-123",
  "timestamp": "2025-01-01T00:00:00Z",
  "retry_after": null
}
```

### Retry Strategies

- **Exponential Backoff**: For transient errors
- **Rate Limit Compliance**: Honor `retry_after` headers
- **Circuit Breaker**: Fail fast during API outages
- **Dead Letter Queue**: For permanently failed messages

## Best Practices

### Conversation Design

1. **Keep Prompts Focused**: Single-purpose prompts perform better
2. **Use System Prompts**: Set context with system prompts
3. **Monitor Token Usage**: Track costs and optimize accordingly
4. **Handle Long Responses**: Consider streaming for long responses

### Error Resilience

1. **Implement Retries**: Handle transient failures gracefully
2. **Monitor Rate Limits**: Track usage to avoid limits
3. **Use Circuit Breakers**: Protect against cascading failures
4. **Log Correlation IDs**: Enable end-to-end tracing

### Performance Optimization

1. **Connection Pooling**: Reuse NATS connections
2. **Batch Operations**: Group related operations
3. **Async Processing**: Use async/await for non-blocking I/O
4. **Resource Monitoring**: Track memory and CPU usage

### Security

1. **Secure API Keys**: Use secret management systems
2. **Network Security**: TLS for all connections
3. **Input Validation**: Validate all inputs thoroughly
4. **Audit Logging**: Log all security-relevant events

## Troubleshooting

### Common Issues

#### Connection Refused

**Problem**: Cannot connect to NATS server
**Solution**: 
- Check NATS server is running
- Verify `NATS_URL` configuration
- Check network connectivity
- Review firewall settings

#### Authentication Failed

**Problem**: Claude API authentication fails
**Solution**:
- Verify `CLAUDE_API_KEY` is correct
- Check API key permissions
- Ensure API key hasn't expired
- Review Claude API status

#### Rate Limits Exceeded

**Problem**: Too many requests to Claude API
**Solution**:
- Implement client-side rate limiting
- Use exponential backoff
- Monitor usage patterns
- Consider upgrading API plan

#### Memory Issues

**Problem**: High memory usage
**Solution**:
- Monitor conversation retention
- Implement conversation cleanup
- Check for memory leaks
- Tune garbage collection

### Debug Mode

Enable debug logging for troubleshooting:

```bash
export LOG_LEVEL=debug
export RUST_LOG=cim_claude_adapter=debug
```

### Health Checks

The adapter exposes health check endpoints:

- `/health`: Liveness probe
- `/ready`: Readiness probe
- `/metrics`: Prometheus metrics

### Monitoring

Key metrics to monitor:

- `conversations_active`: Active conversations
- `prompts_per_second`: Request rate
- `claude_api_latency`: API response time
- `error_rate`: Error percentage
- `token_usage`: Token consumption

## Integration Patterns

### CIM Service Integration

```rust
// CIM service using the adapter
struct MyService {
    nats_client: async_nats::Client,
}

impl MyService {
    async fn ask_claude(&self, question: &str) -> Result<String> {
        let command = StartConversation {
            session_id: self.session_id.clone(),
            initial_prompt: question.to_string(),
            context: ConversationContext::default(),
            correlation_id: CorrelationId::new(),
        };
        
        // Send command
        self.nats_client.publish(
            "cim.claude.commands.start",
            serde_json::to_vec(&command)?
        ).await?;
        
        // Wait for response event
        let mut sub = self.nats_client.subscribe(
            "cim.claude.events.response_received"
        ).await?;
        
        if let Some(msg) = sub.next().await {
            let event: ResponseReceived = serde_json::from_slice(&msg.data)?;
            return Ok(event.response.content);
        }
        
        Err("No response received".into())
    }
}
```

### Event Sourcing Integration

```rust
// Store all Claude events for replay
struct EventStore {
    events: Vec<EventEnvelope>,
}

impl EventStore {
    async fn handle_claude_event(&mut self, event: EventEnvelope) {
        // Store for replay capability
        self.events.push(event.clone());
        
        // Update projections
        match event.event {
            DomainEvent::ConversationStarted { .. } => {
                // Update conversation projection
            },
            DomainEvent::ResponseReceived { .. } => {
                // Update response cache
            },
            _ => {}
        }
    }
}
```

### Microservices Pattern

```rust
// Service mesh integration
#[derive(Clone)]
struct ClaudeService {
    adapter_client: ClaudeAdapterClient,
    circuit_breaker: CircuitBreaker,
}

impl ClaudeService {
    async fn query(&self, prompt: &str) -> Result<String> {
        self.circuit_breaker.call(async {
            self.adapter_client.start_conversation(prompt).await
        }).await
    }
}
```

This user guide provides comprehensive coverage of the CIM Claude Adapter's capabilities, configuration options, and integration patterns. For additional support, refer to the API documentation and design documentation.