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

#### Option 1: Nix (Recommended)

1. **Install Nix** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
   ```

2. **Clone and build with Nix**:
   ```bash
   git clone https://github.com/cowboy-ai/cim-claude-adapter.git
   cd cim-claude-adapter
   nix build
   ```

3. **Run with Nix**:
   ```bash
   export CLAUDE_API_KEY="your-claude-api-key"
   nix run
   ```

#### Option 2: Cargo

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

3. **Run the adapter**:
   ```bash
   cargo run --release
   ```

#### Prerequisites

- **NATS Server** with JetStream using Nix:
  ```nix
  # In your NixOS configuration
  services.nats = {
    enable = true;
    jetstream = true;
    settings = {
      port = 4222;
      http_port = 8222;
    };
  };
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
| `CLAUDE_API_KEY` | ✅ | - | Claude API key (format: `sk-ant-api03-...`) |
| `NATS_URL` | ❌ | `nats://localhost:4222` | NATS server URL |
| `LOG_LEVEL` | ❌ | `info` | Logging level |
| `HEALTH_CHECK_PORT` | ❌ | `8080` | Health check port |

### 🔑 Authentication Setup

The adapter uses Anthropic's standard authentication with `x-api-key` header:

```bash
export CLAUDE_API_KEY=sk-ant-api03-your-actual-key-here
```

Headers sent to Claude API:
- `x-api-key: [your-api-key]`
- `anthropic-version: 2023-06-01` (hard-locked via Nix flake)
- `content-type: application/json`

### 🔒 Version Management

The Anthropic API version is **hard-locked in the Nix flake** for consistency:

```nix
# In flake.nix - single source of truth
anthropicApiVersion = "2023-06-01";
```

**Benefits**:
- ✅ **Reproducible builds** - same API version across all environments
- ✅ **No version drift** - prevents accidental updates breaking compatibility  
- ✅ **Centralized control** - single place to manage API version upgrades
- ✅ **Build-time injection** - version compiled into binary for runtime consistency

**To update the API version**: Edit `anthropicApiVersion` in `flake.nix` and rebuild.

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

#### With Nix (Recommended)
```bash
# Enter development shell
nix develop

# Build the project
nix build

# Build NixOS container
nix build .#container

# Run all checks (format, lint, test, etc.)
nix flake check

# Run specific checks
nix build .#checks.x86_64-linux.clippy
nix build .#checks.x86_64-linux.test
```

#### With Cargo
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

### NixOS Service

Add to your NixOS configuration:

```nix
{
  # Import the flake input
  inputs.cim-claude-adapter.url = "github:TheCowboyAI/cim-agent-claude?dir=cim-claude-adapter";
  
  # In your configuration.nix
  imports = [ inputs.cim-claude-adapter.nixosModules.default ];
  
  services.cim-claude-adapter = {
    enable = true;
    claude.apiKey = config.age.secrets.claude-api-key.path; # Using agenix
    nats.url = "nats://localhost:4222";
    monitoring.healthPort = 8080;
    monitoring.metricsPort = 9090;
    openFirewall = true;
  };
  
  # NATS server
  services.nats = {
    enable = true;
    jetstream = true;
    settings = {
      port = 4222;
      http_port = 8222;
    };
  };
}
```

### NixOS Container

Deploy as a NixOS container:

```nix
{
  containers.cim-claude-adapter = {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "192.168.100.10";
    localAddress = "192.168.100.11";
    
    config = { config, pkgs, ... }: {
      imports = [ inputs.cim-claude-adapter.nixosModules.default ];
      
      services.cim-claude-adapter = {
        enable = true;
        claude.apiKey = "/run/secrets/claude-api-key";
        nats.url = "nats://192.168.100.10:4222"; # Host NATS
        openFirewall = true;
      };
      
      # Networking
      networking.firewall.allowedTCPPorts = [ 8080 9090 ];
      system.stateVersion = "24.05";
    };
  };
}
```

### Multi-Instance Deployment

Scale across multiple containers:

```nix
{
  imports = [ inputs.cim-claude-adapter.nixosModules.default ];
  
  # Helper function for creating adapter instances
  containers = lib.genAttrs [ "primary" "secondary" "tertiary" ] (name: {
    autoStart = true;
    privateNetwork = true;
    hostAddress = "192.168.100.10";
    localAddress = "192.168.100.${toString (11 + (lib.strings.charToInt (lib.stringAsChars (c: c) name) [0]))}";
    
    config = {
      imports = [ inputs.cim-claude-adapter.nixosModules.default ];
      services.cim-claude-adapter = {
        enable = true;
        claude.apiKey = "/run/secrets/claude-api-key";
        nats.url = "nats://192.168.100.10:4222";
        monitoring.healthPort = 8080;
        monitoring.metricsPort = 9090;
        stateDir = "/var/lib/cim-claude-adapter-${name}";
      };
      system.stateVersion = "24.05";
    };
  });
}
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