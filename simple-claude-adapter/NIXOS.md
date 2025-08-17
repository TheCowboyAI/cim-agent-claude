# Claude NATS Adapter - NixOS Module

A proper NixOS module for the Claude API to NATS adapter service.

## Usage

### 1. Add to your NixOS configuration

```nix
# flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    claude-adapter.url = "path:./simple-claude-adapter";  # or your path
  };
  
  outputs = { nixpkgs, claude-adapter, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        claude-adapter.nixosModules.claude-adapter
        ./configuration.nix
      ];
    };
  };
}
```

### 2. Enable in configuration.nix

```nix
# configuration.nix
{
  services.claude-adapter = {
    enable = true;
    claudeApiKey = "sk-ant-api03-your-actual-key-here";
    natsUrl = "nats://localhost:4222";  # optional, default
    logLevel = "info";                  # optional, default
  };
  
  # Also enable NATS if running locally
  services.nats = {
    enable = true;
    jetstream = true;
  };
}
```

### 3. Rebuild and use

```bash
# Rebuild system
sudo nixos-rebuild switch

# Check service status
systemctl status claude-adapter

# View logs
journalctl -u claude-adapter -f

# Test it
nats pub claude.cmd.test.prompt '{
  "command_id": "test-1",
  "correlation_id": "corr-1",
  "prompt": "Hello Claude!",
  "timestamp": "'$(date -Iseconds)'"
}'

# Listen for response
nats sub "claude.event.*" --translate-jq '.data.content'
```

## Module Options

```nix
services.claude-adapter = {
  enable = true;                          # Enable the service
  claudeApiKey = "sk-ant-api03-...";      # Required: Your Claude API key
  natsUrl = "nats://localhost:4222";      # NATS server URL
  logLevel = "info";                      # Log level: error|warn|info|debug|trace
  user = "claude-adapter";                # Service user
  group = "claude-adapter";               # Service group
};
```

## What It Does

- ✅ **Builds the Rust service** automatically
- ✅ **Creates systemd service** with proper hardening
- ✅ **Creates service user** with minimal privileges
- ✅ **Handles dependencies** and environment
- ✅ **Listens on `claude.cmd.*`** for queries
- ✅ **Publishes to `claude.event.*`** with preserved correlation IDs

## Development

```bash
# Enter development shell
nix develop

# Build package
nix build

# Test the module
nixos-rebuild test --flake .#your-host
```

## Security

The module includes systemd hardening:
- Runs as dedicated user with minimal privileges
- System protection enabled
- No new privileges allowed
- Memory write/execute protection
- Namespace restrictions

## Integration Example

Complete NixOS configuration with NATS and Claude adapter:

```nix
{ config, pkgs, claude-adapter, ... }:

{
  imports = [
    claude-adapter.nixosModules.claude-adapter
  ];

  # NATS server
  services.nats = {
    enable = true;
    jetstream = true;
    settings = {
      server_name = "claude-nats";
      listen = "0.0.0.0:4222";
      monitor_port = 8222;
    };
  };

  # Claude adapter
  services.claude-adapter = {
    enable = true;
    claudeApiKey = "sk-your-key-here";
    logLevel = "info";
  };

  # Open firewall for NATS
  networking.firewall.allowedTCPPorts = [ 4222 8222 ];
}
```

**This is the proper NixOS way** - declarative, reproducible, and integrated with the system! 🎉