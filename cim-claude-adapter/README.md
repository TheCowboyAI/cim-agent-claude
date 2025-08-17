# CIM Claude Adapter

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![NATS](https://img.shields.io/badge/NATS-JetStream-blue.svg)](https://nats.io/)
[![Claude](https://img.shields.io/badge/Claude-API-purple.svg)](https://www.anthropic.com/)

A production-ready, event-driven adapter that integrates Claude AI into CIM (Composable Information Machine) ecosystems using NATS messaging. Built following Domain-Driven Design (DDD) principles with hexagonal architecture.

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
- NATS Server 2.9+ with JetStream
- Claude API key from Anthropic

### Installation

1. **Clone and build**:
   ```bash
   git clone https://github.com/cowboy-ai/cim-claude-adapter.git
   cd cim-claude-adapter
   cargo build --release
   ```

2. **Set environment variables**:
   ```bash
   export CLAUDE_API_KEY="your-claude-api-key"
   export NATS_URL="nats://localhost:4222"
   ```

3. **Start NATS with JetStream**:
   ```bash
   docker run -d --name nats -p 4222:4222 nats:latest -js
   ```

4. **Run the adapter**:
   ```bash
   cargo run --release
   ```

### Docker Quick Start

```bash
docker run --name cim-claude-adapter \
  -e CLAUDE_API_KEY="your-api-key" \
  -e NATS_URL="nats://host.docker.internal:4222" \
  --network host \
  cowboy-ai/cim-claude-adapter:latest
```

## 🏗️ Architecture

The CIM Claude Adapter implements **hexagonal architecture** with clean separation of concerns:

```
┌─────────────────┐    NATS     ┌─────────────────┐    HTTPS   ┌─────────────────┐
│                 │ ◄────────► │                 │ ◄───────► │                 │
│  CIM Services   │  Commands  │  Claude         │  REST API │   Claude API    │
│                 │   Events   │  Adapter        │           │                 │
└─────────────────┘            └─────────────────┘           └─────────────────┘
```

### Key Components

- **Domain Layer**: Business logic with DDD aggregates, events, and commands
- **Application Layer**: Service orchestration and workflow management
- **Infrastructure Layer**: NATS and Claude API integrations
- **Ports & Adapters**: Clean interfaces and implementations

## ✨ Features

### 🎯 Event-Driven Architecture
- Complete event sourcing with audit trails
- Command/Query Responsibility Segregation (CQRS)
- Domain events for all state changes
- Correlation ID tracking across service boundaries

### 📡 NATS Integration  
- JetStream for reliable message delivery
- Structured subject patterns for routing
- Durable consumers with retry logic
- Built-in scalability through NATS clustering

### 🤖 Claude AI Integration
- Rate limiting and circuit breaker patterns
- Automatic retry with exponential backoff
- Token usage tracking and monitoring
- Support for multiple Claude models

### 🛡️ Production Ready
- Health checks and monitoring endpoints
- Structured logging with correlation IDs
- Graceful shutdown handling
- Comprehensive error handling and recovery

## 📖 Documentation

| Document | Description |
|----------|-------------|
| [**User Guide**](./docs/USER_GUIDE.md) | Complete usage guide with examples |
| [**API Reference**](./docs/API.md) | Detailed API documentation with schemas |
| [**Design Document**](./docs/DESIGN.md) | Architecture and design decisions |
| [**Contributing**](./CONTRIBUTING.md) | Guidelines for contributors |

## 🚦 Usage Example

### Start a Conversation

```bash
# Send command via NATS
nats pub cim.claude.commands.start '{
  "session_id": "user-123",
  "initial_prompt": "Explain quantum computing in simple terms",
  "context": {
    "max_tokens": 1000,
    "temperature": 0.7
  },
  "correlation_id": "req-001"
}'
```

### Monitor Events

```bash
# Listen for all Claude events
nats sub 'cim.claude.events.>'

# Listen for responses only
nats sub 'cim.claude.events.response_received'
```

### Example Response Event

```json
{
  "event_id": "evt-456",
  "correlation_id": "req-001",
  "event": "ResponseReceived",
  "timestamp": "2025-01-15T10:30:00Z",
  "data": {
    "conversation_id": "conv-789",
    "response": {
      "content": "Quantum computing uses quantum mechanical phenomena...",
      "usage": {
        "input_tokens": 12,
        "output_tokens": 156,
        "total_tokens": 168
      }
    }
  }
}
```

## ⚙️ Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `CLAUDE_API_KEY` | ✅ | - | Claude API authentication key |
| `NATS_URL` | ❌ | `nats://localhost:4222` | NATS server URL |
| `LOG_LEVEL` | ❌ | `info` | Logging level |
| `HEALTH_CHECK_PORT` | ❌ | `8080` | Health check port |

See the [User Guide](./docs/USER_GUIDE.md) for complete configuration options.

## 🔍 Monitoring

The adapter exposes several monitoring endpoints:

- `GET /health` - Health check (liveness probe)
- `GET /ready` - Readiness check
- `GET /metrics` - Prometheus metrics

### Key Metrics

- `conversations_active` - Current active conversations
- `prompts_per_second` - Request rate
- `claude_api_latency_seconds` - API response times
- `token_usage_total` - Token consumption

## 🛠️ Development

### Prerequisites

- Rust 1.70+ with Cargo
- NATS Server with JetStream
- Claude API access for testing

### Building

```bash
# Development build
cargo build

# Production build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Testing

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration

# With coverage
cargo tarpaulin --out Html
```

See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed development guidelines.

## 🚀 Deployment

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/cim-claude-adapter /usr/local/bin/
CMD ["cim-claude-adapter"]
```

### Kubernetes

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

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](./CONTRIBUTING.md) for details.

### Quick Contribution Steps

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run tests: `cargo test`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🏢 About Cowboy AI

Cowboy AI, LLC develops advanced AI infrastructure and tooling for enterprise applications. The CIM Claude Adapter is part of our broader CIM ecosystem for building composable, event-driven AI systems.

For more information:
- Website: [cowboy-ai.com](https://cowboy-ai.com)
- Email: hello@cowboy-ai.com
- Security: security@cowboy-ai.com

## 🆘 Support

- 📖 [Documentation](./docs/)
- 🐛 [Issue Tracker](https://github.com/cowboy-ai/cim-claude-adapter/issues)
- 💬 [GitHub Discussions](https://github.com/cowboy-ai/cim-claude-adapter/discussions)
- ✉️ Email: support@cowboy-ai.com

---

**Built with ❤️ by the Cowboy AI team**