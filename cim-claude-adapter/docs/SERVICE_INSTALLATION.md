# CIM Expert Service - Installation & Usage

## Overview

The CIM Expert Service is a complete NixOS module that provides CIM architectural expertise as a system service with conversation control. It includes:

- **NATS Integration** - Communicates with your CIM ecosystem
- **HTTP API** - RESTful endpoints for conversation management
- **Web Interface** - Interactive consultation interface
- **WebSocket Support** - Real-time conversation capabilities
- **Complete Audit Trails** - All consultations logged to NATS JetStream

## 🚀 Installation

### Option 1: NixOS System Configuration

Add to your `configuration.nix` or flake:

```nix
{
  imports = [
    ./path/to/cim-claude-adapter/nix/cim-expert-service.nix
  ];

  services.cim-expert = {
    enable = true;
    
    # Service configuration
    port = 8080;
    bindAddress = "0.0.0.0";  # Bind to all interfaces
    logLevel = "info";
    maxConcurrentConversations = 20;
    
    # Claude API configuration
    claude = {
      apiKeyFile = "/run/secrets/claude-api-key";
      maxTokens = 800;
      temperature = 0.3;
      timeoutSeconds = 60;
      maxRetries = 3;
    };
    
    # NATS configuration (connects to your CIM)
    nats = {
      servers = [ 
        "nats://localhost:4222"
        # "nats://your-cim-server:4222"  # Add your CIM servers
      ];
      connectionTimeoutSeconds = 10;
      requestTimeoutSeconds = 30;
      maxReconnectAttempts = 10;
    };
    
    # Expert configuration
    expert = {
      enableConversationHistory = true;
      maxConversationLength = 50;
      enableAuditLogging = true;
    };
    
    # Web interface
    webInterface = {
      enable = true;
      enableApiDocs = true;
    };
  };
  
  # Required: Set up the Claude API key secret
  secrets.claude-api-key = {
    source = "/path/to/your/claude-api-key.txt";
    destination = "/run/secrets/claude-api-key";
    mode = "0400";
    user = "cim-expert";
  };
  
  # Optional: Enable firewall rules
  networking.firewall.allowedTCPPorts = [ 8080 ];
}
```

### Option 2: NixOS Flake Integration

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cim-expert.url = "github:thecowboyai/cim-agent-claude/cim-claude-adapter";
  };
  
  outputs = { self, nixpkgs, cim-expert }: {
    nixosConfigurations.your-server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        cim-expert.nixosModules.cim-expert-service
        {
          services.cim-expert.enable = true;
          # ... configuration as above
        }
      ];
    };
  };
}
```

### Option 3: Standalone Installation

```bash
# Clone the repository
git clone https://github.com/thecowboyai/cim-agent-claude.git
cd cim-agent-claude/cim-claude-adapter

# Build with Nix
nix build

# Install as system service
sudo cp result/bin/cim-expert-service /usr/local/bin/
sudo cp nix/cim-expert-service.nix /etc/nixos/modules/

# Create service configuration
sudo mkdir -p /etc/cim-expert
sudo cp docs/examples/cim-expert-config.toml /etc/cim-expert/

# Create systemd service (if not using NixOS)
sudo cp docs/examples/cim-expert.service /etc/systemd/system/
sudo systemctl enable cim-expert
sudo systemctl start cim-expert
```

## 🔧 Configuration

### Service Settings

```toml
[service]
bind_address = "127.0.0.1"    # Address to bind to
port = 8080                   # HTTP port
log_level = "info"            # trace, debug, info, warn, error
max_concurrent_conversations = 10

[claude]
max_tokens = 800              # Max tokens per response
temperature = 0.3             # Response creativity (0.0-1.0)
timeout_seconds = 60          # Request timeout
max_retries = 3               # Retry attempts

[nats]
servers = ["nats://localhost:4222"]
connection_timeout_seconds = 10
request_timeout_seconds = 30
max_reconnect_attempts = 10

[expert]
enable_conversation_history = true
max_conversation_length = 20
enable_audit_logging = true

[web_interface]
enable = true
enable_api_docs = true
```

### Required Environment Variables

```bash
# Claude API key (required)
export CLAUDE_API_KEY="sk-ant-api03-your-actual-key"

# Optional overrides
export CIM_EXPERT_CONFIG="/etc/cim-expert/config.toml"
export RUST_LOG="cim_claude_adapter=info"
```

## 🌐 Usage

### Web Interface

Once the service is running, access the web interface:

```
http://localhost:8080/          # Interactive conversation interface
http://localhost:8080/docs      # API documentation
http://localhost:8080/health    # Health check
http://localhost:8080/metrics   # Service metrics
```

### HTTP API Endpoints

#### Start a New Conversation
```bash
curl -X POST http://localhost:8080/api/v1/conversations \
  -H "Content-Type: application/json" \
  -d '{
    "context": "healthcare-domain",
    "user_id": "developer-123"
  }'
```

#### Send a Message
```bash
curl -X POST http://localhost:8080/api/v1/conversations/{id}/messages \
  -H "Content-Type: application/json" \
  -d '{
    "message": "How do I implement event sourcing for medical records?",
    "topic": "EventSourcing"
  }'
```

#### List Conversations
```bash
curl http://localhost:8080/api/v1/conversations?user_id=developer-123&limit=10
```

#### Get Conversation History
```bash
curl http://localhost:8080/api/v1/conversations/{id}
```

#### Direct Expert Query (Stateless)
```bash
curl -X POST http://localhost:8080/api/v1/expert/ask \
  -H "Content-Type: application/json" \
  -d '{
    "question": "What are the key components of a CIM system?",
    "topic": "Architecture",
    "domain_context": "manufacturing"
  }'
```

### NATS Integration

The service publishes to NATS subjects for integration with your CIM:

```bash
# Subscribe to all expert consultations
nats sub "cim.expert.events.consultation.*"

# Subscribe to specific expert responses
nats sub "cim.expert.responses.architecture.*"

# Send queries via NATS
nats pub cim.expert.query.architecture '{
  "question": "How do I scale my CIM?",
  "topic": "Architecture",
  "user_id": "system-admin"
}'
```

## 📊 Monitoring & Management

### SystemD Commands

```bash
# Check service status
sudo systemctl status cim-expert

# View logs
sudo journalctl -u cim-expert -f

# Restart service
sudo systemctl restart cim-expert

# Reload configuration
sudo systemctl reload cim-expert
```

### Health Checks

```bash
# Basic health check
curl http://localhost:8080/health

# Service metrics
curl http://localhost:8080/metrics

# NATS connectivity check
curl http://localhost:8080/api/v1/status/nats
```

### Log Management

Logs are automatically rotated using logrotate:

```bash
# View current logs
tail -f /var/log/cim-expert/service.log

# View archived logs
ls -la /var/log/cim-expert/
```

## 🔒 Security

### Network Security

- Service runs on localhost by default
- Use nginx reverse proxy for external access
- Enable TLS/SSL for production deployments
- Configure firewall rules appropriately

### API Security

- API key authentication for sensitive endpoints
- Rate limiting on conversation endpoints
- CORS configured for web interface
- Input validation and sanitization

### System Security

- Runs as dedicated `cim-expert` user
- Restricted filesystem access
- Resource limits enforced
- Security capabilities dropped

## 🚀 Integration Examples

### With CIM-Start Project

```nix
# In your CIM project's flake.nix
{
  inputs = {
    cim-start.url = "github:thecowboyai/cim-start";
    cim-expert.url = "github:thecowboyai/cim-agent-claude/cim-claude-adapter";
  };
  
  outputs = { cim-start, cim-expert, ... }: {
    nixosConfigurations.my-cim = nixpkgs.lib.nixosSystem {
      modules = [
        cim-start.nixosModules.cim-system
        cim-expert.nixosModules.cim-expert-service
        {
          services.cim.enable = true;
          services.cim-expert.enable = true;
          
          # Expert connects to CIM's NATS
          services.cim-expert.nats.servers = [ 
            "nats://127.0.0.1:${toString config.services.cim.nats.port}"
          ];
        }
      ];
    };
  };
}
```

### Programmatic Integration

```rust
// In your Rust application
use reqwest::Client;
use serde_json::json;

let client = Client::new();

// Start conversation
let conversation = client
    .post("http://localhost:8080/api/v1/conversations")
    .json(&json!({
        "context": "my-application",
        "user_id": "app-user"
    }))
    .send()
    .await?
    .json::<ConversationResponse>()
    .await?;

// Ask expert
let response = client
    .post(&format!("http://localhost:8080/api/v1/conversations/{}/messages", conversation.conversation_id))
    .json(&json!({
        "message": "How should I structure my domain events?",
        "topic": "EventSourcing"
    }))
    .send()
    .await?
    .json::<ExpertResponse>()
    .await?;

println!("Expert says: {}", response.response.explanation);
```

## 🎯 Production Deployment

### High Availability Setup

```nix
{
  services.cim-expert = {
    enable = true;
    
    # Production settings
    maxConcurrentConversations = 100;
    
    # Multiple NATS servers
    nats.servers = [
      "nats://nats-1.internal:4222"
      "nats://nats-2.internal:4222" 
      "nats://nats-3.internal:4222"
    ];
    
    # Resource limits
    systemd.services.cim-expert.serviceConfig = {
      MemoryMax = "4G";
      CPUQuota = "400%";
    };
  };
  
  # Load balancer configuration
  services.nginx.upstreams.cim-expert = {
    servers = {
      "127.0.0.1:8080" = {};
      "127.0.0.1:8081" = {};  # Multiple instances
    };
  };
}
```

### Monitoring Integration

```nix
{
  # Prometheus metrics
  services.prometheus.exporters.node.enable = true;
  
  # Grafana dashboard
  services.grafana.provision.dashboards.settings.providers = [{
    name = "cim-expert";
    folder = "CIM";
    path = "/etc/grafana/dashboards/cim-expert.json";
  }];
  
  # Alert rules
  services.prometheus.rules = [
    ''
      groups:
        - name: cim-expert
          rules:
            - alert: CimExpertDown
              expr: up{job="cim-expert"} == 0
              for: 1m
              labels:
                severity: critical
              annotations:
                summary: "CIM Expert service is down"
    ''
  ];
}
```

This service module provides a complete, production-ready CIM Expert that can be installed on any NixOS system and integrates seamlessly with your CIM ecosystem!