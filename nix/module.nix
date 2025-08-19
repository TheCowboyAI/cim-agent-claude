# NixOS Module for CIM Agent Claude
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-agent-claude;
  
  # Helper function to create systemd environment
  mkEnvironment = envVars: mapAttrsToList (name: value: "${name}=${toString value}") envVars;

in {
  options.services.cim-agent-claude = {
    enable = mkEnableOption "CIM Agent Claude service";

    package = mkOption {
      type = types.package;
      description = "The CIM Agent Claude package to use";
    };

    adapter = {
      enable = mkEnableOption "CIM Claude Adapter service" // { default = cfg.enable; };
      
      user = mkOption {
        type = types.str;
        default = "cim-claude";
        description = "User to run the CIM Claude Adapter service as";
      };
      
      group = mkOption {
        type = types.str;
        default = "cim-claude";
        description = "Group to run the CIM Claude Adapter service as";
      };

      nats = {
        url = mkOption {
          type = types.str;
          default = "nats://localhost:4222";
          description = "NATS server URL";
        };
        
        subject_prefix = mkOption {
          type = types.str;
          default = "cim.claude";
          description = "NATS subject prefix for CIM Claude events";
        };
      };

      claude = {
        apiKeyFile = mkOption {
          type = types.nullOr types.path;
          default = null;
          description = "Path to file containing Claude API key";
        };
        
        baseUrl = mkOption {
          type = types.str;
          default = "https://api.anthropic.com";
          description = "Claude API base URL";
        };
        
        model = mkOption {
          type = types.str;
          default = "claude-3-5-sonnet-20241022";
          description = "Claude model to use";
        };
        
        maxTokens = mkOption {
          type = types.int;
          default = 4096;
          description = "Maximum tokens per request";
        };
        
        temperature = mkOption {
          type = types.float;
          default = 0.7;
          description = "Temperature for Claude responses (0.0-1.0)";
        };
      };

      server = {
        port = mkOption {
          type = types.port;
          default = 8080;
          description = "Port for the adapter server";
        };
        
        host = mkOption {
          type = types.str;
          default = "127.0.0.1";
          description = "Host to bind the adapter server to";
        };
        
        cleanupIntervalSeconds = mkOption {
          type = types.int;
          default = 300;
          description = "Interval in seconds between conversation cleanup runs";
        };
        
        healthCheckIntervalSeconds = mkOption {
          type = types.int;
          default = 30;
          description = "Interval in seconds between health checks";
        };
      };

      observability = {
        logLevel = mkOption {
          type = types.enum [ "TRACE" "DEBUG" "INFO" "WARN" "ERROR" ];
          default = "INFO";
          description = "Log level for the service";
        };
        
        metricsEnabled = mkOption {
          type = types.bool;
          default = true;
          description = "Enable metrics collection";
        };
        
        metricsPort = mkOption {
          type = types.port;
          default = 9090;
          description = "Port for metrics endpoint";
        };
        
        tracingEnabled = mkOption {
          type = types.bool;
          default = false;
          description = "Enable distributed tracing";
        };
      };

      environmentFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to environment file containing additional secrets";
      };
    };

    web = {
      enable = mkEnableOption "CIM Claude Web GUI" // { default = cfg.enable; };
      
      package = mkOption {
        type = types.package;
        description = "The CIM Claude GUI WASM package to use";
      };
      
      virtualHost = mkOption {
        type = types.str;
        default = "cim-claude.local";
        description = "Virtual host name for the web interface";
      };
      
      port = mkOption {
        type = types.port;
        default = 8081;
        description = "Port for the web interface";
      };
      
      enableSSL = mkOption {
        type = types.bool;
        default = false;
        description = "Enable SSL for the web interface";
      };
      
      sslCertificate = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to SSL certificate file";
      };
      
      sslCertificateKey = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to SSL certificate key file";
      };
    };
  };

  config = mkIf cfg.enable {
    
    # User and group for the adapter service
    users.users = mkIf cfg.adapter.enable {
      "${cfg.adapter.user}" = {
        description = "CIM Claude Adapter service user";
        isSystemUser = true;
        group = cfg.adapter.group;
        home = "/var/lib/cim-claude";
        createHome = true;
      };
    };

    users.groups = mkIf cfg.adapter.enable {
      "${cfg.adapter.group}" = {};
    };

    # Systemd service for the adapter
    systemd.services.cim-claude-adapter = mkIf cfg.adapter.enable {
      description = "CIM Claude Adapter - Event-driven Claude AI integration service";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "simple";
        User = cfg.adapter.user;
        Group = cfg.adapter.group;
        Restart = "always";
        RestartSec = 10;
        
        # Security settings
        NoNewPrivileges = true;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ "/var/lib/cim-claude" ];
        
        # Resource limits
        MemoryMax = "1G";
        TasksMax = 1000;
        
        # Execution
        ExecStart = "${cfg.package}/bin/cim-claude-adapter";
        WorkingDirectory = "/var/lib/cim-claude";
        
        # Environment file
        EnvironmentFile = mkIf (cfg.adapter.environmentFile != null) cfg.adapter.environmentFile;
      };
      
      environment = {
        # NATS Configuration
        NATS_URL = cfg.adapter.nats.url;
        NATS_SUBJECT_PREFIX = cfg.adapter.nats.subject_prefix;
        
        # Claude API Configuration
        CLAUDE_BASE_URL = cfg.adapter.claude.baseUrl;
        CLAUDE_MODEL = cfg.adapter.claude.model;
        CLAUDE_MAX_TOKENS = toString cfg.adapter.claude.maxTokens;
        CLAUDE_TEMPERATURE = toString cfg.adapter.claude.temperature;
        
        # Server Configuration
        SERVER_HOST = cfg.adapter.server.host;
        SERVER_PORT = toString cfg.adapter.server.port;
        SERVER_CLEANUP_INTERVAL_SECONDS = toString cfg.adapter.server.cleanupIntervalSeconds;
        SERVER_HEALTH_CHECK_INTERVAL_SECONDS = toString cfg.adapter.server.healthCheckIntervalSeconds;
        
        # Observability Configuration
        LOG_LEVEL = cfg.adapter.observability.logLevel;
        METRICS_ENABLED = if cfg.adapter.observability.metricsEnabled then "true" else "false";
        METRICS_PORT = toString cfg.adapter.observability.metricsPort;
        TRACING_ENABLED = if cfg.adapter.observability.tracingEnabled then "true" else "false";
        
        # Rust logging
        RUST_LOG = "${cfg.adapter.observability.logLevel}";
      };
      
      preStart = mkIf (cfg.adapter.claude.apiKeyFile != null) ''
        if [ -f "${cfg.adapter.claude.apiKeyFile}" ]; then
          export CLAUDE_API_KEY="$(cat "${cfg.adapter.claude.apiKeyFile}")"
        else
          echo "Warning: Claude API key file not found at ${cfg.adapter.claude.apiKeyFile}"
        fi
      '';
    };

    # Nginx configuration for the web interface
    services.nginx = mkIf cfg.web.enable {
      enable = true;
      
      virtualHosts."${cfg.web.virtualHost}" = {
        listen = [
          { addr = "0.0.0.0"; port = cfg.web.port; }
        ] ++ (optional cfg.web.enableSSL { addr = "0.0.0.0"; port = 443; ssl = true; });
        
        root = "${cfg.web.package}/share/cim-claude-gui-web";
        
        locations = {
          "/" = {
            tryFiles = "$uri $uri/ /index.html";
            extraConfig = ''
              add_header Cache-Control "no-cache, no-store, must-revalidate";
              add_header Pragma "no-cache";
              add_header Expires "0";
              
              # WASM MIME types
              location ~* \.wasm$ {
                add_header Content-Type "application/wasm";
                add_header Cache-Control "public, max-age=31536000, immutable";
              }
              
              # JavaScript modules
              location ~* \.js$ {
                add_header Content-Type "application/javascript";
                add_header Cache-Control "public, max-age=31536000, immutable";
              }
            '';
          };
          
          # API proxy to adapter service
          "/api/" = {
            proxyPass = "http://${cfg.adapter.server.host}:${toString cfg.adapter.server.port}/";
            proxyWebsockets = true;
            extraConfig = ''
              proxy_set_header Host $host;
              proxy_set_header X-Real-IP $remote_addr;
              proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
              proxy_set_header X-Forwarded-Proto $scheme;
            '';
          };
          
          # WebSocket proxy to NATS WebSocket endpoint
          "/nats-ws" = {
            proxyPass = "http://127.0.0.1:8222/";
            proxyWebsockets = true;
            extraConfig = ''
              proxy_set_header Host $host;
              proxy_set_header X-Real-IP $remote_addr;
              proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
              proxy_set_header X-Forwarded-Proto $scheme;
              
              # WebSocket specific headers
              proxy_set_header Upgrade $http_upgrade;
              proxy_set_header Connection "upgrade";
              
              # Timeouts for WebSocket connections
              proxy_connect_timeout 7d;
              proxy_send_timeout 7d;
              proxy_read_timeout 7d;
            '';
          };
        };
        
        # SSL configuration
        sslCertificate = mkIf cfg.web.enableSSL cfg.web.sslCertificate;
        sslCertificateKey = mkIf cfg.web.enableSSL cfg.web.sslCertificateKey;
        
        extraConfig = ''
          # Security headers
          add_header X-Frame-Options "SAMEORIGIN" always;
          add_header X-Content-Type-Options "nosniff" always;
          add_header X-XSS-Protection "1; mode=block" always;
          add_header Referrer-Policy "strict-origin-when-cross-origin" always;
          
          # WASM support
          location ~* \.wasm$ {
            add_header Cross-Origin-Embedder-Policy "require-corp";
            add_header Cross-Origin-Opener-Policy "same-origin";
          }
        '';
      };
    };

    # Firewall configuration
    networking.firewall = {
      allowedTCPPorts = mkMerge [
        (mkIf cfg.adapter.enable [ cfg.adapter.server.port ])
        (mkIf (cfg.adapter.enable && cfg.adapter.observability.metricsEnabled) [ cfg.adapter.observability.metricsPort ])
        (mkIf cfg.web.enable [ cfg.web.port ])
        (mkIf (cfg.web.enable && cfg.web.enableSSL) [ 443 ])
        (mkIf (cfg.adapter.enable && cfg.adapter.nats.url == "nats://localhost:4222") [ 4222 8222 ]) # NATS TCP and WebSocket ports
      ];
    };

    # Ensure NATS is available if using default local NATS
    services.nats = mkIf (cfg.adapter.enable && cfg.adapter.nats.url == "nats://localhost:4222") {
      enable = mkDefault true;
      jetstream = true;
      port = 4222;
      
      # Enable WebSocket support for web GUI
      settings = {
        websocket = {
          port = 8222;  # Different port to avoid conflict with adapter
          no_tls = true;
          # Allow CORS for local development
          same_origin = false;
          allowed_origins = [ "http://localhost:8081" "http://127.0.0.1:8081" ];
        };
        # JetStream configuration for persistent storage
        jetstream = {
          store_dir = "/var/lib/nats/jetstream";
          max_memory_store = "1GB";
          max_file_store = "10GB";
        };
      };
    };

    # System packages
    environment.systemPackages = mkMerge [
      (mkIf cfg.adapter.enable [ cfg.package ])
      (mkIf cfg.web.enable [ cfg.web.package ])
    ];
  };

  meta = {
    maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
    description = "NixOS module for CIM Agent Claude - Event-driven Claude AI integration";
  };
}