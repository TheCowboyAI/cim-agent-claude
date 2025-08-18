# CIM Claude Adapter - NixOS Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

# Import the NATS infrastructure module
let
  natsInfrastructure = import ./nats-infrastructure.nix { inherit config lib pkgs; };
in

with lib;

let
  cfg = config.services.cim-claude-adapter;
  
  # Default configuration file
  configFile = pkgs.writeText "cim-claude-adapter.toml" ''
    [claude]
    api_key = "${cfg.claude.apiKey}"
    model = "${cfg.claude.model}"
    max_tokens = ${toString cfg.claude.maxTokens}
    temperature = ${toString cfg.claude.temperature}
    
    [nats]
    url = "${cfg.nats.url}"
    max_reconnects = ${toString cfg.nats.maxReconnects}
    reconnect_wait = "${cfg.nats.reconnectWait}"
    ${optionalString (cfg.nats.credentialsFile != null) ''
    credentials_file = "${cfg.nats.credentialsFile}"
    ''}
    
    [conversation]
    max_prompt_length = ${toString cfg.conversation.maxPromptLength}
    max_exchanges = ${toString cfg.conversation.maxExchanges}
    session_timeout = "${cfg.conversation.sessionTimeout}"
    
    [monitoring]
    metrics_port = ${toString cfg.monitoring.metricsPort}
    health_port = ${toString cfg.monitoring.healthPort}
    enable_tracing = ${if cfg.monitoring.enableTracing then "true" else "false"}
    
    [logging]
    level = "${cfg.logging.level}"
    format = "${cfg.logging.format}"
    
    ${cfg.extraConfig}
  '';

in {
  ###### Interface
  options.services.cim-claude-adapter = {
    enable = mkEnableOption "CIM Claude Adapter service";

    package = mkOption {
      type = types.package;
      default = pkgs.cim-claude-adapter;
      defaultText = literalExpression "pkgs.cim-claude-adapter";
      description = "The CIM Claude Adapter package to use.";
    };

    user = mkOption {
      type = types.str;
      default = "cim-claude-adapter";
      description = "User account under which the service runs.";
    };

    group = mkOption {
      type = types.str;
      default = "cim-claude-adapter";
      description = "Group under which the service runs.";
    };

    stateDir = mkOption {
      type = types.str;
      default = "/var/lib/cim-claude-adapter";
      description = "Directory to store application state.";
    };

    # Claude configuration
    claude = {
      apiKey = mkOption {
        type = types.str;
        description = ''
          Claude API key from Anthropic.
          Consider using agenix or sops-nix for secret management.
        '';
        example = "sk-ant-api03-...";
      };

      model = mkOption {
        type = types.str;
        default = "claude-3-sonnet-20240229";
        description = "Claude model to use.";
      };

      maxTokens = mkOption {
        type = types.int;
        default = 4000;
        description = "Maximum tokens per request.";
      };

      temperature = mkOption {
        type = types.float;
        default = 0.7;
        description = "Temperature for Claude responses (0.0-1.0).";
      };
    };

    # NATS configuration
    nats = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable integrated NATS infrastructure with JetStream.";
      };
      
      url = mkOption {
        type = types.str;
        default = "nats://localhost:4222";
        description = "NATS server URL.";
      };

      maxReconnects = mkOption {
        type = types.int;
        default = 10;
        description = "Maximum number of reconnection attempts.";
      };

      reconnectWait = mkOption {
        type = types.str;
        default = "5s";
        description = "Wait time between reconnection attempts.";
      };

      credentialsFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to NATS credentials file.";
        example = "/run/secrets/nats-credentials";
      };
      
      # Expose NATS infrastructure options
      infrastructure = {
        enable = mkOption {
          type = types.bool;
          default = cfg.nats.enable;
          description = "Enable NATS infrastructure with dedicated streams and stores.";
        };
        
        environment = mkOption {
          type = types.enum [ "development" "staging" "production" ];
          default = "production";
          description = "Environment for NATS resource scaling.";
        };
        
        replication = {
          replicas = mkOption {
            type = types.ints.between 1 5;
            default = 3;
            description = "Number of replicas for JetStream resources.";
          };
        };
        
        openFirewall = mkOption {
          type = types.bool;
          default = false;
          description = "Open firewall ports for NATS server.";
        };
      };
    };

    # Conversation configuration
    conversation = {
      maxPromptLength = mkOption {
        type = types.int;
        default = 50000;
        description = "Maximum prompt length in characters.";
      };

      maxExchanges = mkOption {
        type = types.int;
        default = 100;
        description = "Maximum exchanges per conversation.";
      };

      sessionTimeout = mkOption {
        type = types.str;
        default = "30m";
        description = "Session timeout duration.";
      };
    };

    # Monitoring configuration
    monitoring = {
      metricsPort = mkOption {
        type = types.port;
        default = 9090;
        description = "Port for Prometheus metrics endpoint.";
      };

      healthPort = mkOption {
        type = types.port;
        default = 8080;
        description = "Port for health check endpoint.";
      };

      enableTracing = mkOption {
        type = types.bool;
        default = true;
        description = "Enable distributed tracing.";
      };
    };

    # Logging configuration
    logging = {
      level = mkOption {
        type = types.enum [ "error" "warn" "info" "debug" "trace" ];
        default = "info";
        description = "Logging level.";
      };

      format = mkOption {
        type = types.enum [ "json" "pretty" ];
        default = "json";
        description = "Log format.";
      };
    };

    # Environment variables
    environment = mkOption {
      type = types.attrsOf types.str;
      default = { };
      description = "Additional environment variables for the service.";
      example = literalExpression ''
        {
          RUST_BACKTRACE = "1";
          OTEL_SERVICE_NAME = "cim-claude-adapter";
        }
      '';
    };

    # Extra configuration
    extraConfig = mkOption {
      type = types.lines;
      default = "";
      description = "Additional configuration to append to the config file.";
      example = ''
        [custom_section]
        custom_option = "value"
      '';
    };

    # Service configuration
    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to open firewall ports for health and metrics endpoints.";
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # User and group
    users.users.${cfg.user} = {
      description = "CIM Claude Adapter service user";
      group = cfg.group;
      home = cfg.stateDir;
      createHome = true;
      homeMode = "755";
      isSystemUser = true;
    };

    users.groups.${cfg.group} = { };

    # Systemd service
    systemd.services.cim-claude-adapter = {
      description = "CIM Claude Adapter - Event-driven Claude AI integration";
      documentation = [ "https://github.com/TheCowboyAI/cim-agent-claude" ];
      after = [ "network-online.target" ] ++ optional cfg.nats.infrastructure.enable "nats-server.service";
      wants = [ "network-online.target" ] ++ optional cfg.nats.infrastructure.enable "nats-server.service";
      requires = optional cfg.nats.infrastructure.enable "nats-jetstream-setup.service";
      wantedBy = [ "multi-user.target" ];

      environment = cfg.environment // {
        RUST_LOG = cfg.logging.level;
        CONFIG_FILE = toString configFile;
      };

      serviceConfig = {
        Type = "exec";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/cim-expert-service";
        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
        
        # Security settings
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RemoveIPC = true;
        PrivateTmp = true;
        
        # Working directory
        WorkingDirectory = cfg.stateDir;
        StateDirectory = baseNameOf cfg.stateDir;
        StateDirectoryMode = "0750";
        
        # Restart configuration
        Restart = "always";
        RestartSec = "10s";
        StartLimitInterval = "5min";
        StartLimitBurst = 3;
        
        # Resource limits
        LimitNOFILE = "65536";
        TasksMax = "4096";
      };

      # Health check
      unitConfig = {
        StartLimitIntervalSec = "5min";
        StartLimitBurst = 3;
      };
    };

    # Firewall configuration
    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.monitoring.healthPort cfg.monitoring.metricsPort ];
    };

    # Ensure package is available
    environment.systemPackages = [ cfg.package ];
    
    # Enable NATS infrastructure if requested
    services.cim-claude-nats = mkIf cfg.nats.infrastructure.enable {
      enable = true;
      inherit (cfg.nats.infrastructure) environment openFirewall;
      replication.replicas = cfg.nats.infrastructure.replication.replicas;
      
      # Use the same port as the adapter expects
      port = 
        if hasPrefix "nats://" cfg.nats.url then
          toInt (last (splitString ":" cfg.nats.url))
        else 4222;
      
      # Configure JetStream with appropriate resources based on environment
      jetstream = {
        enable = true;
        domain = "cim-claude";
        storeDir = "/var/lib/nats/jetstream";
        maxMemoryStore = 
          if cfg.nats.infrastructure.environment == "development" then "256MB"
          else if cfg.nats.infrastructure.environment == "staging" then "512MB"
          else "1GB";
        maxFileStore = 
          if cfg.nats.infrastructure.environment == "development" then "10GB"
          else if cfg.nats.infrastructure.environment == "staging" then "50GB"
          else "100GB";
      };
    };

    # Assertions
    assertions = [
      {
        assertion = cfg.claude.apiKey != "";
        message = "services.cim-claude-adapter.claude.apiKey must be set";
      }
      {
        assertion = cfg.claude.temperature >= 0.0 && cfg.claude.temperature <= 1.0;
        message = "services.cim-claude-adapter.claude.temperature must be between 0.0 and 1.0";
      }
      {
        assertion = cfg.claude.maxTokens > 0 && cfg.claude.maxTokens <= 8192;
        message = "services.cim-claude-adapter.claude.maxTokens must be between 1 and 8192";
      }
      {
        assertion = cfg.monitoring.healthPort != cfg.monitoring.metricsPort;
        message = "Health and metrics ports must be different";
      }
      {
        assertion = !cfg.nats.infrastructure.enable || cfg.nats.enable;
        message = "NATS infrastructure requires NATS to be enabled";
      }
    ];

    # Warnings
    warnings = optional (cfg.claude.apiKey != "" && !hasPrefix "/run/secrets" cfg.claude.apiKey) ''
      services.cim-claude-adapter.claude.apiKey is stored in the Nix store.
      Consider using agenix, sops-nix, or NixOS secrets for better security.
    '';
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <hello@cowboy-ai.com>" ];
    doc = ./cim-claude-adapter.md;
  };
}