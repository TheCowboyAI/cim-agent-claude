# CIM Claude Adapter NixOS Module

*Copyright 2025 - Cowboy AI, LLC. All rights reserved.*

## Overview

The CIM Claude Adapter NixOS module provides a declarative way to deploy and configure the CIM Claude Adapter service on NixOS systems.

## Basic Configuration

```nix
{
  services.cim-claude-adapter = {
    enable = true;
    claude.apiKey = "sk-ant-api03-your-api-key-here";
    nats.url = "nats://localhost:4222";
  };
}
```

## Complete Configuration Example

```nix
{ config, ... }:

{
  services.cim-claude-adapter = {
    enable = true;
    
    # Claude API configuration
    claude = {
      apiKey = config.age.secrets.claude-api-key.path; # Using agenix
      model = "claude-3-sonnet-20240229";
      maxTokens = 4000;
      temperature = 0.7;
    };
    
    # NATS configuration
    nats = {
      url = "nats://nats.internal:4222";
      maxReconnects = 10;
      reconnectWait = "5s";
      credentialsFile = "/run/secrets/nats-credentials";
    };
    
    # Conversation limits
    conversation = {
      maxPromptLength = 50000;
      maxExchanges = 100;
      sessionTimeout = "30m";
    };
    
    # Monitoring
    monitoring = {
      metricsPort = 9090;
      healthPort = 8080;
      enableTracing = true;
    };
    
    # Logging
    logging = {
      level = "info";
      format = "json";
    };
    
    # Additional environment variables
    environment = {
      RUST_BACKTRACE = "1";
      OTEL_SERVICE_NAME = "cim-claude-adapter";
      OTEL_EXPORTER_OTLP_ENDPOINT = "http://jaeger:4317";
    };
    
    # Open firewall for monitoring endpoints
    openFirewall = true;
  };
  
  # Optional: Configure secrets with agenix
  age.secrets.claude-api-key = {
    file = ../secrets/claude-api-key.age;
    owner = "cim-claude-adapter";
    group = "cim-claude-adapter";
  };
}
```

## Security Considerations

### Secret Management

**DO NOT** put API keys directly in your NixOS configuration. Use one of these approaches:

#### Using agenix

```nix
{
  services.cim-claude-adapter = {
    claude.apiKey = config.age.secrets.claude-api-key.path;
  };
  
  age.secrets.claude-api-key = {
    file = ./secrets/claude-api-key.age;
    owner = "cim-claude-adapter";
  };
}
```

#### Using sops-nix

```nix
{
  sops.secrets.claude-api-key = {
    owner = "cim-claude-adapter";
  };
  
  services.cim-claude-adapter = {
    claude.apiKey = config.sops.secrets.claude-api-key.path;
  };
}
```

#### Using NixOS secrets

```nix
{
  services.cim-claude-adapter = {
    claude.apiKey = "/run/secrets/claude-api-key";
  };
}
```

### Network Security

The service runs with restricted network access by default. Configure firewall rules as needed:

```nix
{
  services.cim-claude-adapter.openFirewall = true; # Opens health and metrics ports
  
  # Or configure manually
  networking.firewall.allowedTCPPorts = [ 8080 9090 ];
}
```

## Integration Examples

### With Prometheus Monitoring

```nix
{
  services = {
    cim-claude-adapter = {
      enable = true;
      monitoring.metricsPort = 9090;
      # ... other config
    };
    
    prometheus = {
      enable = true;
      scrapeConfigs = [{
        job_name = "cim-claude-adapter";
        static_configs = [{
          targets = [ "localhost:9090" ];
        }];
      }];
    };
  };
}
```

### With NATS Server

```nix
{
  services = {
    # NATS server with JetStream
    nats = {
      enable = true;
      jetstream = true;
      settings = {
        port = 4222;
        http_port = 8222;
      };
    };
    
    # CIM Claude Adapter
    cim-claude-adapter = {
      enable = true;
      nats.url = "nats://localhost:4222";
      # ... other config
    };
  };
}
```

### With Reverse Proxy (Nginx)

```nix
{
  services = {
    cim-claude-adapter = {
      enable = true;
      monitoring.healthPort = 8080;
      # Don't open firewall, use nginx proxy
      openFirewall = false;
    };
    
    nginx = {
      enable = true;
      virtualHosts."adapter.example.com" = {
        locations."/health" = {
          proxyPass = "http://localhost:8080/health";
        };
        locations."/metrics" = {
          proxyPass = "http://localhost:9090/metrics";
        };
      };
    };
  };
}
```

### Multi-Instance Deployment

```nix
{ lib, ... }:

let
  mkAdapterInstance = name: port: {
    "cim-claude-adapter-${name}" = {
      enable = true;
      monitoring.healthPort = port;
      monitoring.metricsPort = port + 1000;
      stateDir = "/var/lib/cim-claude-adapter-${name}";
      user = "cim-claude-adapter-${name}";
      group = "cim-claude-adapter";
      # ... other config
    };
  };
in {
  services = lib.mkMerge [
    (mkAdapterInstance "primary" 8080)
    (mkAdapterInstance "secondary" 8081)
    (mkAdapterInstance "tertiary" 8082)
  ];
}
```

## Troubleshooting

### Service Status

```bash
# Check service status
systemctl status cim-claude-adapter

# View logs
journalctl -u cim-claude-adapter -f

# Check configuration
systemctl cat cim-claude-adapter
```

### Health Checks

```bash
# Health endpoint
curl http://localhost:8080/health

# Metrics endpoint
curl http://localhost:9090/metrics
```

### Common Issues

1. **API Key Issues**: Ensure the Claude API key is valid and accessible
2. **NATS Connection**: Verify NATS server is running and accessible
3. **Permission Issues**: Check file permissions for state directory and secrets
4. **Network Issues**: Verify firewall configuration and port availability

## Module Options Reference

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | false | Enable the service |
| `package` | package | `pkgs.cim-claude-adapter` | Package to use |
| `user` | string | "cim-claude-adapter" | Service user |
| `group` | string | "cim-claude-adapter" | Service group |
| `stateDir` | string | "/var/lib/cim-claude-adapter" | State directory |
| `claude.apiKey` | string | - | Claude API key (required) |
| `claude.model` | string | "claude-3-sonnet-20240229" | Claude model |
| `claude.maxTokens` | int | 4000 | Max tokens per request |
| `claude.temperature` | float | 0.7 | Response temperature |
| `nats.url` | string | "nats://localhost:4222" | NATS server URL |
| `nats.credentialsFile` | path? | null | NATS credentials file |
| `monitoring.healthPort` | port | 8080 | Health check port |
| `monitoring.metricsPort` | port | 9090 | Metrics port |
| `logging.level` | enum | "info" | Log level |
| `openFirewall` | bool | false | Open firewall ports |

For complete options, see the module source code.