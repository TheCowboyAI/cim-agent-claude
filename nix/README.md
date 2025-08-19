# CIM Agent Claude - NixOS Installation Guide

This directory contains the NixOS module and configuration files for CIM Agent Claude, providing a unified installation of the adapter service and web GUI.

## Quick Start

### 1. Add to your NixOS configuration

```nix
# In your configuration.nix or flake.nix
{
  inputs.cim-agent-claude.url = "github:TheCowboyAI/cim-agent-claude";
  
  outputs = { self, nixpkgs, cim-agent-claude, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        cim-agent-claude.nixosModules.cim-agent-claude
        {
          services.cim-agent-claude = {
            enable = true;
            package = cim-agent-claude.packages.x86_64-linux.cim-claude-adapter;
            web.package = cim-agent-claude.packages.x86_64-linux.cim-claude-gui-wasm;
            
            # Configure your Claude API key
            adapter.claude.apiKeyFile = "/path/to/your/claude-api-key";
          };
        }
      ];
    };
  };
}
```

### 2. Deploy

```bash
sudo nixos-rebuild switch
```

### 3. Access the services

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