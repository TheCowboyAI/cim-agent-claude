# Example NixOS configuration for CIM Agent Claude
# This shows how to use the CIM Agent Claude NixOS module

{ config, pkgs, lib, ... }:

let
  # Import the CIM Agent Claude flake
  cim-agent-claude-flake = builtins.getFlake "github:TheCowboyAI/cim-agent-claude";
  cim-packages = cim-agent-claude-flake.packages.${pkgs.system};

in {
  imports = [
    # Import the CIM Agent Claude module
    cim-agent-claude-flake.nixosModules.cim-agent-claude
  ];

  # Enable CIM Agent Claude service
  services.cim-agent-claude = {
    enable = true;
    
    # Set the package to use
    package = cim-packages.cim-claude-adapter;
    
    # Adapter configuration
    adapter = {
      enable = true;
      user = "cim-claude";
      group = "cim-claude";
      
      # NATS configuration
      nats = {
        url = "nats://localhost:4222";  # WebSocket GUI will connect via ws://localhost:8081/nats-ws
        subject_prefix = "cim.claude.example";
      };
      
      # Claude API configuration
      claude = {
        # Store your API key in a file readable by the service
        apiKeyFile = "/run/secrets/claude-api-key";
        baseUrl = "https://api.anthropic.com";
        model = "claude-3-5-sonnet-20241022";
        maxTokens = 4096;
        temperature = 0.7;
      };
      
      # Server configuration
      server = {
        host = "127.0.0.1";
        port = 8080;
        cleanupIntervalSeconds = 300;
        healthCheckIntervalSeconds = 30;
      };
      
      # Observability
      observability = {
        logLevel = "INFO";
        metricsEnabled = true;
        metricsPort = 9090;
        tracingEnabled = false;
      };
      
      # Optional: Environment file for additional secrets
      # environmentFile = "/etc/cim-claude/environment";
    };
    
    # Web interface configuration
    web = {
      enable = true;
      package = cim-packages.cim-claude-gui-wasm;
      virtualHost = "cim-claude.example.com";
      port = 8081;
      
      # Optional: Enable SSL
      enableSSL = false;
      # sslCertificate = "/path/to/cert.pem";
      # sslCertificateKey = "/path/to/key.pem";
    };
  };

  # NATS server (automatically enabled if using localhost)
  services.nats = {
    enable = true;
    jetstream = true;
    port = 4222;
    
    # Optional: Configure NATS clustering for high availability
    # cluster = {
    #   enable = true;
    #   port = 6222;
    #   routes = [
    #     "nats://node1.example.com:6222"
    #     "nats://node2.example.com:6222"
    #   ];
    # };
  };

  # Optional: Create secret file for Claude API key
  # You can use agenix, sops-nix, or simple file-based secrets
  environment.etc."cim-claude/claude-api-key" = {
    text = "your-claude-api-key-here";
    mode = "0400";
    user = config.services.cim-agent-claude.adapter.user;
    group = config.services.cim-agent-claude.adapter.group;
  };

  # Alternative using systemd credentials (recommended for production)
  systemd.services.cim-claude-adapter.serviceConfig = {
    LoadCredential = [
      "claude-api-key:/etc/cim-claude/claude-api-key"
    ];
  };

  # Alternative environment file approach
  # environment.etc."cim-claude/environment" = {
  #   text = ''
  #     CLAUDE_API_KEY=your-api-key-here
  #     # Add other sensitive environment variables here
  #   '';
  #   mode = "0400";
  #   user = config.services.cim-agent-claude.adapter.user;
  #   group = config.services.cim-agent-claude.adapter.group;
  # };

  # Ensure required ports are open in firewall (handled automatically by module)
  # networking.firewall.allowedTCPPorts = [ 8080 8081 9090 ];

  # Optional: Enable Prometheus monitoring
  services.prometheus = {
    enable = true;
    scrapeConfigs = [
      {
        job_name = "cim-claude-adapter";
        static_configs = [
          {
            targets = [ "localhost:${toString config.services.cim-agent-claude.adapter.observability.metricsPort}" ];
          }
        ];
      }
    ];
  };

  # Optional: Enable Grafana dashboards
  services.grafana = {
    enable = true;
    settings = {
      server = {
        http_port = 3000;
        domain = "grafana.example.com";
      };
    };
  };
}