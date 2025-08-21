# CIM Agent Claude NixOS Module

A comprehensive NixOS module for the CIM (Composable Information Machine) Agent Claude ecosystem, providing multiple services with flexible enable/disable options.

## Quick Start

### 1. Add to your flake inputs:

```nix
{
  inputs = {
    cim-agent-claude.url = "github:TheCowboyAI/cim-agent-claude";
  };
}
```

### 2. Import the module:

```nix
{
  imports = [
    cim-agent-claude.nixosModules.default
  ];
}
```

### 3. Basic configuration:

```nix
services.cim-agent-claude = {
  enable = true;
  package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;
  
  # Enable SAGE orchestrator (recommended)
  sage = {
    enable = true;
    package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
    claude.apiKeyFile = "/var/lib/sage/claude-api-key";
  };
};
```

## Available Services

### 🎭 SAGE (Systematic Agent Guidance Engine)
**Recommended primary service** - Intelligent orchestrator with 17 expert agents

```nix
services.cim-agent-claude.sage = {
  enable = true;  # Default: follows main enable
  package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
  
  user = "sage";
  group = "sage";
  
  nats = {
    url = "nats://localhost:4222";
    sageSubject = "cim.sage";
  };
  
  claude = {
    apiKeyFile = "/var/lib/sage/claude-api-key";  # Required
  };
  
  server = {
    host = "127.0.0.1";
    port = 8082;
  };
  
  observability = {
    logLevel = "INFO";  # TRACE, DEBUG, INFO, WARN, ERROR
  };
  
  environmentFile = null;  # Optional secrets file
};
```

### 🔧 Claude Adapter Service
Direct Claude API integration service

```nix
services.cim-agent-claude.adapter = {
  enable = false;  # Default: follows main enable, often not needed with SAGE
  
  user = "cim-claude";
  group = "cim-claude";
  
  # ... (see full configuration in example files)
};
```

### 🖥️ Desktop GUI
Native desktop application using Iced

```nix
services.cim-agent-claude.gui = {
  enable = false;  # Default: disabled
  package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui;
  
  autostart = false;  # Auto-launch on user login
};
```

### 🌐 Web Interface
WebAssembly-based web GUI served via nginx

```nix
services.cim-agent-claude.web = {
  enable = false;  # Default: disabled
  package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui-wasm;
  
  virtualHost = "cim-claude.local";
  port = 8081;
  
  # SSL support
  enableSSL = false;
  sslCertificate = null;
  sslCertificateKey = null;
};
```

## Service Combinations

### Option 1: SAGE Only (Recommended)
Minimal setup with intelligent orchestration:

```nix
services.cim-agent-claude = {
  enable = true;
  package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;
  
  sage = {
    enable = true;
    package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
    claude.apiKeyFile = "/var/lib/sage/claude-api-key";
  };
  
  # Explicitly disable other services
  adapter.enable = false;
  gui.enable = false;
  web.enable = false;
};
```

### Option 2: SAGE + Web Interface
Perfect for server deployments:

```nix
services.cim-agent-claude = {
  enable = true;
  package = cim-agent-claude.packages.${pkgs.system}.cim-agent-claude;
  
  sage = {
    enable = true;
    package = cim-agent-claude.packages.${pkgs.system}.cim-sage-service;
    claude.apiKeyFile = "/var/lib/sage/claude-api-key";
  };
  
  web = {
    enable = true;
    package = cim-agent-claude.packages.${pkgs.system}.cim-claude-gui-wasm;
    virtualHost = "cim.example.com";
  };
  
  adapter.enable = false;
  gui.enable = false;
};
```

## Expert Agents Available in SAGE

SAGE provides 17 specialized expert agents:

1. **🎭 sage** - Master orchestrator
2. **🏗️ cim-expert** - CIM architecture & foundations
3. **🌐 cim-domain-expert** - Domain-specific architecture
4. **📐 ddd-expert** - Domain-driven design
5. **🔍 event-storming-expert** - Collaborative discovery
6. **📊 domain-expert** - Domain creation & validation
7. **📋 bdd-expert** - Behavior-driven development
8. **🧪 tdd-expert** - Test-driven development
9. **✅ qa-expert** - Quality assurance
10. **📨 nats-expert** - NATS messaging
11. **🌐 network-expert** - Network topology
12. **⚙️ nix-expert** - Nix configuration
13. **🔧 git-expert** - Git & GitHub operations
14. **📐 subject-expert** - CIM subject algebra
15. **🎨 iced-ui-expert** - Desktop GUI development
16. **🔄 elm-architecture-expert** - Functional UI patterns
17. **⚡ cim-tea-ecs-expert** - TEA+ECS integration

## API Key Management

Create secure API key files:

```bash
# For SAGE service
sudo mkdir -p /var/lib/sage
echo "sk-your-claude-api-key" | sudo tee /var/lib/sage/claude-api-key
sudo chown sage:sage /var/lib/sage/claude-api-key
sudo chmod 600 /var/lib/sage/claude-api-key

# For Adapter service (if enabled)
sudo mkdir -p /var/lib/cim-claude
echo "sk-your-claude-api-key" | sudo tee /var/lib/cim-claude/claude-api-key
sudo chown cim-claude:cim-claude /var/lib/cim-claude/claude-api-key
sudo chmod 600 /var/lib/cim-claude/claude-api-key
```

## Deploy

```bash
sudo nixos-rebuild switch
```

## Access Services

- **SAGE Service**: Port 8082
- **Web GUI**: http://localhost:8081  
- **Adapter API**: http://localhost:8080
- **Metrics**: http://localhost:9090/metrics

## Architecture

The CIM Agent Claude system consists of:

1. **CIM Claude Adapter** - Backend service that:
   - Connects to NATS via native TCP (port 4222)
   - Integrates with Claude API for AI conversations
   - Provides REST API for management
   - Publishes metrics for monitoring

2. **CIM Claude GUI** - Dual-mode interface:
   - **Desktop**: Native binary with direct NATS TCP connection
   - **Web**: WebAssembly app connecting via WebSocket proxy
   - Provides conversation management UI
   - Shows real-time system metrics
   - Includes CIM Expert functionality

3. **NATS Server** - Message broker with dual connectivity:
   - **TCP**: Native connection on port 4222 (for adapter & desktop GUI)
   - **WebSocket**: WebSocket endpoint on port 8222 (for web GUI)
   - JetStream for persistent event storage
   - Enables horizontal scaling

4. **Nginx Reverse Proxy** - Web server that:
   - Serves static WASM files on port 8081
   - Proxies `/nats-ws` to NATS WebSocket (port 8222)
   - Proxies `/api/` to adapter REST API (port 8080)
   - Handles SSL termination if enabled

### Connection Flow

```
Desktop GUI ──TCP──> NATS Server (4222)
                         ↑
Adapter ────TCP──────────┘

Web Browser ──HTTP──> Nginx (8081) ──WebSocket──> NATS Server (8222)
                           └──HTTP──> Adapter (8080)
```

## Configuration Options

### Adapter Service

```nix
services.cim-agent-claude.adapter = {
  enable = true;
  user = "cim-claude";
  group = "cim-claude";
  
  nats = {
    url = "nats://localhost:4222";
    subject_prefix = "cim.claude";
  };
  
  claude = {
    apiKeyFile = "/run/secrets/claude-api-key";
    baseUrl = "https://api.anthropic.com";
    model = "claude-3-5-sonnet-20241022";
    maxTokens = 4096;
    temperature = 0.7;
  };
  
  server = {
    host = "127.0.0.1";
    port = 8080;
    cleanupIntervalSeconds = 300;
    healthCheckIntervalSeconds = 30;
  };
  
  observability = {
    logLevel = "INFO";
    metricsEnabled = true;
    metricsPort = 9090;
    tracingEnabled = false;
  };
};
```

### Web Interface

```nix
services.cim-agent-claude.web = {
  enable = true;
  package = cim-packages.cim-claude-gui-wasm;
  virtualHost = "cim-claude.local";
  port = 8081;
  
  # Optional SSL
  enableSSL = false;
  sslCertificate = "/path/to/cert.pem";
  sslCertificateKey = "/path/to/key.pem";
};
```

## Security Considerations

### API Key Management

**Option 1: File-based (simple)**
```nix
environment.etc."cim-claude/api-key" = {
  text = "your-api-key-here";
  mode = "0400";
  user = "cim-claude";
  group = "cim-claude";
};

services.cim-agent-claude.adapter.claude.apiKeyFile = "/etc/cim-claude/api-key";
```

**Option 2: Systemd credentials (recommended)**
```nix
systemd.services.cim-claude-adapter.serviceConfig.LoadCredential = [
  "claude-api-key:/path/to/secure/api-key"
];
# Then reference with $CREDENTIALS_DIRECTORY/claude-api-key in the service
```

**Option 3: External secret management (production)**
Use tools like [agenix](https://github.com/ryantm/agenix) or [sops-nix](https://github.com/Mic92/sops-nix).

### Network Security

The module automatically:
- Creates a dedicated system user with minimal permissions
- Configures strict systemd security settings
- Sets up firewall rules for required ports
- Enables secure headers for the web interface

### Resource Limits

The service includes resource limits:
- Memory: 1GB max
- Tasks: 1000 max
- Network: Local only by default

## Monitoring

### Prometheus Integration

```nix
services.prometheus = {
  enable = true;
  scrapeConfigs = [{
    job_name = "cim-claude-adapter";
    static_configs = [{
      targets = [ "localhost:9090" ];
    }];
  }];
};
```

### Available Metrics

- `cim_conversations_total` - Total conversations started
- `cim_conversations_active` - Currently active conversations  
- `cim_events_published_total` - Events published to NATS
- `cim_events_consumed_total` - Events consumed from NATS
- `cim_api_requests_total` - Claude API requests
- `cim_api_requests_failed_total` - Failed API requests
- `cim_response_time_seconds` - Response time histogram

### Logs

View service logs:
```bash
sudo journalctl -u cim-claude-adapter -f
```

## High Availability

### NATS Clustering

```nix
services.nats = {
  enable = true;
  jetstream = true;
  cluster = {
    enable = true;
    port = 6222;
    routes = [
      "nats://node1.example.com:6222"
      "nats://node2.example.com:6222"
      "nats://node3.example.com:6222"
    ];
  };
};
```

### Multi-Instance Deployment

Deploy multiple adapter instances with different subject prefixes:

```nix
services.cim-agent-claude = {
  enable = true;
  adapter.nats.subject_prefix = "cim.claude.region1";
  # Configure load balancer to distribute requests
};
```

## Development

### Local Development

Use the development shell:
```bash
nix develop github:TheCowboyAI/cim-agent-claude
```

### Building Packages

```bash
# Build adapter
nix build github:TheCowboyAI/cim-agent-claude#cim-claude-adapter

# Build desktop GUI
nix build github:TheCowboyAI/cim-agent-claude#cim-claude-gui

# Build web GUI
nix build github:TheCowboyAI/cim-agent-claude#cim-claude-gui-wasm
```

### Testing

The flake includes comprehensive checks:
```bash
nix flake check github:TheCowboyAI/cim-agent-claude
```

## Troubleshooting

### Common Issues

**Service fails to start**
```bash
# Check service status
systemctl status cim-claude-adapter

# Check logs
journalctl -u cim-claude-adapter -n 50
```

**NATS connection issues**
```bash
# Test NATS connectivity
nats-cli --server=localhost:4222 server info
```

**Claude API issues**
- Verify API key is correct
- Check rate limits and quotas
- Ensure network connectivity to api.anthropic.com

**Web interface not loading**
- Check nginx configuration: `nginx -t`
- Verify WASM files are served correctly
- Check browser console for WebAssembly errors

### Performance Tuning

**For high-throughput environments:**

```nix
services.cim-agent-claude.adapter = {
  server.cleanupIntervalSeconds = 60;  # More frequent cleanup
  observability.logLevel = "WARN";     # Reduce log verbosity
};

# Increase resource limits
systemd.services.cim-claude-adapter.serviceConfig = {
  MemoryMax = "2G";
  TasksMax = 2000;
};
```

## Support

For issues and support:
- GitHub: https://github.com/TheCowboyAI/cim-agent-claude/issues
- Documentation: https://docs.thecowboy.ai/cim-agent-claude
- Email: support@thecowboy.ai