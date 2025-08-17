# CIM Claude Adapter - Nix Infrastructure

Copyright 2025 - Cowboy AI, LLC. All rights reserved.

This directory contains the Nix-based infrastructure configuration for the CIM Claude Adapter, including NixOS modules, container definitions, and integration tests.

## Files Overview

- **`module.nix`** - Main NixOS module for the CIM Claude Adapter service
- **`nats-infrastructure.nix`** - Complete NATS JetStream infrastructure with streams, object stores, and KV stores
- **`test-integration.nix`** - Integration test configuration for validating the complete system
- **`README.md`** - This documentation file

## Quick Start

### 1. Basic Usage (Adapter Only)

```nix
# In your NixOS configuration
{
  imports = [ ./path/to/cim-claude-adapter/nix/module.nix ];
  
  services.cim-claude-adapter = {
    enable = true;
    claude.apiKey = "/run/secrets/claude-api-key";
    nats.url = "nats://your-nats-server:4222";
  };
}
```

### 2. Full Infrastructure (Adapter + NATS)

```nix
# In your NixOS configuration  
{
  imports = [ 
    ./path/to/cim-claude-adapter/nix/module.nix 
  ];
  
  services.cim-claude-adapter = {
    enable = true;
    claude.apiKey = "/run/secrets/claude-api-key";
    
    # Enable integrated NATS infrastructure
    nats = {
      enable = true;
      infrastructure = {
        enable = true;
        environment = "production";  # or "development", "staging"
        replication.replicas = 3;
        openFirewall = true;
      };
    };
    
    openFirewall = true;
  };
}
```

### 3. Using the Container

```bash
# Build the container
nix build .#container

# Run with nixos-container
sudo nixos-container create cim-claude --flake .#container
sudo nixos-container start cim-claude
```

### 4. Running Integration Tests

```bash
# Build and test the integration system
nix build .#test-system

# Run integration checks  
nix flake check
```

## Configuration Options

### CIM Claude Adapter (`services.cim-claude-adapter`)

#### Core Options
- `enable` - Enable the service
- `package` - Package to use (defaults to the one built by this flake)
- `user/group` - Service user and group
- `stateDir` - State directory for the service

#### Claude Configuration (`claude`)
- `apiKey` - Anthropic Claude API key (use secrets management!)
- `model` - Claude model to use
- `maxTokens` - Maximum tokens per request
- `temperature` - Response temperature (0.0-1.0)

#### NATS Configuration (`nats`)
- `enable` - Enable NATS integration
- `url` - NATS server URL
- `maxReconnects` - Maximum reconnection attempts
- `reconnectWait` - Wait time between reconnects
- `credentialsFile` - Path to NATS credentials file

#### NATS Infrastructure (`nats.infrastructure`)
- `enable` - Enable integrated NATS server with JetStream
- `environment` - Environment scaling: "development", "staging", "production"
- `replication.replicas` - Number of replicas for JetStream resources (1-5)
- `openFirewall` - Open firewall ports for NATS

#### Monitoring (`monitoring`)
- `metricsPort` - Prometheus metrics port (default: 9090)
- `healthPort` - Health check port (default: 8080)
- `enableTracing` - Enable distributed tracing

#### Logging (`logging`)
- `level` - Log level: "error", "warn", "info", "debug", "trace"
- `format` - Log format: "json", "pretty"

### NATS Infrastructure (`services.cim-claude-nats`)

When `nats.infrastructure.enable = true`, the adapter automatically configures:

#### JetStream Streams
- **CIM_CLAUDE_CONV_CMD** - Conversation commands
- **CIM_CLAUDE_CONV_EVT** - Conversation events (audit trail)
- **CIM_CLAUDE_ATTACH_CMD** - Attachment commands  
- **CIM_CLAUDE_ATTACH_EVT** - Attachment events
- **CIM_CLAUDE_CONV_QRY** - Query requests
- **CIM_SYS_HEALTH_EVT** - System health and metrics

#### Object Stores
- **CIM_CLAUDE_ATTACH_OBJ_IMG** - Images and screenshots
- **CIM_CLAUDE_ATTACH_OBJ_DOC** - Documents (PDF, text)
- **CIM_CLAUDE_ATTACH_OBJ_CODE** - Code files
- **CIM_CLAUDE_ATTACH_OBJ_AUDIO** - Audio files
- **CIM_CLAUDE_ATTACH_OBJ_VIDEO** - Video files
- **CIM_CLAUDE_ATTACH_OBJ_BIN** - Binary files

#### KV Stores
- **CIM_CLAUDE_CONV_KV** - Conversation metadata
- **CIM_CLAUDE_ATTACH_KV** - Attachment metadata
- **CIM_CLAUDE_SESSION_KV** - User sessions
- **CIM_CLAUDE_CONFIG_KV** - Configuration settings
- **CIM_CLAUDE_METRICS_KV** - Aggregated metrics

## Environment Scaling

The infrastructure automatically scales resources based on the environment:

### Development
- Minimal resource allocation
- Single replica for all resources
- Short TTLs for faster iteration
- Reduced storage limits

### Staging  
- Medium resource allocation
- Dual replicas
- Moderate TTLs and storage limits

### Production
- Full resource allocation
- Triple replicas for high availability
- Long TTLs for data retention
- Maximum storage limits

## Security Considerations

### Secrets Management
Use NixOS secrets management for sensitive values:

```nix
# Example with agenix
age.secrets.claude-api-key.file = ./secrets/claude-api-key.age;
services.cim-claude-adapter.claude.apiKey = config.age.secrets.claude-api-key.path;

# Example with sops-nix
sops.secrets.claude-api-key = {};
services.cim-claude-adapter.claude.apiKey = config.sops.secrets.claude-api-key.path;
```

### Firewall Configuration
- Set `openFirewall = false` in production behind a load balancer
- Only expose necessary ports through proper network security
- Use TLS for NATS connections in production

### Service Security
The services run with extensive security hardening:
- No new privileges
- Protected file system
- Restricted address families
- Memory write/execute protection
- Private temporary directories

## Monitoring and Observability

### Metrics
- Prometheus metrics available at `:9090/metrics`
- Health check endpoint at `:8080/health`
- NATS monitoring at `:8222` (when NATS infrastructure is enabled)

### Logging
- Structured JSON logging in production
- Pretty formatting for development
- Configurable log levels
- Integration with systemd journal

## Development and Testing

### Development Shell
```bash
nix develop
# Provides: rust toolchain, nats-cli, development tools
```

### Building
```bash
# Build the adapter
nix build

# Build container
nix build .#container

# Build test system
nix build .#test-system
```

### Testing
```bash
# Run all checks
nix flake check

# Run specific test
nix build .#checks.x86_64-linux.integration-test
```

This infrastructure provides a complete, production-ready deployment solution for the CIM Claude Adapter with integrated NATS JetStream infrastructure, following CIM architectural principles and Nix best practices.