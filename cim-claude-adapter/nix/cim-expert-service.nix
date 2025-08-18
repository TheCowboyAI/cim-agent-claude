# CIM Expert Service - NixOS Module
# Provides CIM architectural expertise as a system service with conversation control
{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.cim-expert;
  
  # Build the CIM Expert service binary
  cim-expert-service = pkgs.rustPlatform.buildRustPackage rec {
    pname = "cim-expert-service";
    version = "0.1.0";
    
    src = lib.cleanSource ./..;
    
    cargoLock = {
      lockFile = ../Cargo.lock;
    };
    
    buildFeatures = [ "service" ];
    
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    
    buildInputs = with pkgs; [
      openssl
    ];
    
    # Set the hard-locked Anthropic API version at build time
    CIM_ANTHROPIC_API_VERSION = "2023-06-01";
    
    meta = with lib; {
      description = "CIM Expert Service - Architectural guidance for Composable Information Machines";
      homepage = "https://github.com/thecowboyai/cim-agent-claude";
      license = licenses.mit;
      maintainers = [ "Cowboy AI, LLC" ];
      platforms = platforms.unix;
    };
  };
  
  # Configuration file for the service
  serviceConfig = pkgs.writeText "cim-expert-config.toml" ''
    [service]
    bind_address = "${cfg.bindAddress}"
    port = ${toString cfg.port}
    log_level = "${cfg.logLevel}"
    max_concurrent_conversations = ${toString cfg.maxConcurrentConversations}
    
    [claude]
    api_version = "2023-06-01"
    max_tokens = ${toString cfg.claude.maxTokens}
    temperature = ${toString cfg.claude.temperature}
    timeout_seconds = ${toString cfg.claude.timeoutSeconds}
    max_retries = ${toString cfg.claude.maxRetries}
    
    [nats]
    servers = [${lib.concatMapStringsSep ", " (s: ''"${s}"'') cfg.nats.servers}]
    connection_timeout_seconds = ${toString cfg.nats.connectionTimeoutSeconds}
    request_timeout_seconds = ${toString cfg.nats.requestTimeoutSeconds}
    max_reconnect_attempts = ${toString cfg.nats.maxReconnectAttempts}
    
    [expert]
    enable_conversation_history = ${lib.boolToString cfg.expert.enableConversationHistory}
    max_conversation_length = ${toString cfg.expert.maxConversationLength}
    enable_audit_logging = ${lib.boolToString cfg.expert.enableAuditLogging}
    
    [web_interface]
    enable = ${lib.boolToString cfg.webInterface.enable}
    static_files_path = "${cfg.webInterface.staticFilesPath}"
    enable_api_docs = ${lib.boolToString cfg.webInterface.enableApiDocs}
  '';
  
in {
  options.services.cim-expert = {
    enable = lib.mkEnableOption "CIM Expert Service";
    
    package = lib.mkOption {
      type = lib.types.package;
      default = cim-expert-service;
      description = "The CIM Expert service package to use";
    };
    
    bindAddress = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Address to bind the service to";
    };
    
    port = lib.mkOption {
      type = lib.types.port;
      default = 8080;
      description = "Port for the CIM Expert service";
    };
    
    logLevel = lib.mkOption {
      type = lib.types.enum [ "trace" "debug" "info" "warn" "error" ];
      default = "info";
      description = "Log level for the service";
    };
    
    maxConcurrentConversations = lib.mkOption {
      type = lib.types.int;
      default = 10;
      description = "Maximum number of concurrent conversations";
    };
    
    claude = {
      apiKeyFile = lib.mkOption {
        type = lib.types.path;
        description = "Path to file containing Claude API key";
        example = "/run/secrets/claude-api-key";
      };
      
      maxTokens = lib.mkOption {
        type = lib.types.int;
        default = 800;
        description = "Maximum tokens per Claude response";
      };
      
      temperature = lib.mkOption {
        type = lib.types.float;
        default = 0.3;
        description = "Claude response temperature (0.0-1.0)";
      };
      
      timeoutSeconds = lib.mkOption {
        type = lib.types.int;
        default = 60;
        description = "Request timeout in seconds";
      };
      
      maxRetries = lib.mkOption {
        type = lib.types.int;
        default = 3;
        description = "Maximum retry attempts";
      };
    };
    
    nats = {
      servers = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ "nats://localhost:4222" ];
        description = "NATS server URLs";
      };
      
      connectionTimeoutSeconds = lib.mkOption {
        type = lib.types.int;
        default = 10;
        description = "NATS connection timeout";
      };
      
      requestTimeoutSeconds = lib.mkOption {
        type = lib.types.int;
        default = 30;
        description = "NATS request timeout";
      };
      
      maxReconnectAttempts = lib.mkOption {
        type = lib.types.int;
        default = 10;
        description = "Maximum NATS reconnection attempts";
      };
    };
    
    expert = {
      enableConversationHistory = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable conversation history tracking";
      };
      
      maxConversationLength = lib.mkOption {
        type = lib.types.int;
        default = 20;
        description = "Maximum messages per conversation";
      };
      
      enableAuditLogging = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable audit logging of all consultations";
      };
    };
    
    webInterface = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable web interface for conversation control";
      };
      
      staticFilesPath = lib.mkOption {
        type = lib.types.str;
        default = "${cim-expert-service}/share/cim-expert/static";
        description = "Path to static web files";
      };
      
      enableApiDocs = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = "Enable API documentation endpoint";
      };
    };
    
    user = lib.mkOption {
      type = lib.types.str;
      default = "cim-expert";
      description = "User to run the service as";
    };
    
    group = lib.mkOption {
      type = lib.types.str;
      default = "cim-expert";
      description = "Group to run the service as";
    };
  };
  
  config = lib.mkIf cfg.enable {
    # Create system user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "CIM Expert service user";
      home = "/var/lib/cim-expert";
      createHome = true;
    };
    
    users.groups.${cfg.group} = {};
    
    # SystemD service configuration
    systemd.services.cim-expert = {
      description = "CIM Expert Service - Architectural guidance for CIMs";
      wantedBy = [ "multi-user.target" ];
      after = [ "network-online.target" "nats.service" ];
      wants = [ "network-online.target" ];
      
      serviceConfig = {
        Type = "exec";
        User = cfg.user;
        Group = cfg.group;
        
        # Security settings
        NoNewPrivileges = true;
        ProtectSystem = "full";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;
        
        # Resource limits
        MemoryMax = "2G";
        CPUQuota = "200%";
        
        # Working directory
        WorkingDirectory = "/var/lib/cim-expert";
        
        # Environment
        Environment = [
          "CIM_EXPERT_CONFIG=${serviceConfig}"
          "RUST_LOG=${cfg.logLevel}"
          "CIM_ANTHROPIC_API_VERSION=2023-06-01"
        ];
        
        EnvironmentFile = cfg.claude.apiKeyFile;
        
        # Service executable
        ExecStart = "${cfg.package}/bin/cim-expert-service --config ${serviceConfig}";
        
        # Restart policy
        Restart = "always";
        RestartSec = 5;
        
        # Logging
        StandardOutput = "journal";
        StandardError = "journal";
      };
      
      # Health check
      serviceConfig.ExecStartPost = "${pkgs.curl}/bin/curl -f http://${cfg.bindAddress}:${toString cfg.port}/health || exit 1";
    };
    
    # Firewall configuration
    networking.firewall = lib.mkIf cfg.webInterface.enable {
      allowedTCPPorts = [ cfg.port ];
    };
    
    # Nginx reverse proxy (optional)
    services.nginx = lib.mkIf cfg.webInterface.enable {
      enable = lib.mkDefault true;
      virtualHosts."cim-expert.local" = {
        locations."/" = {
          proxyPass = "http://${cfg.bindAddress}:${toString cfg.port}";
          proxyWebsockets = true;
          extraConfig = ''
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
          '';
        };
      };
    };
    
    # Log rotation
    services.logrotate.settings.cim-expert = {
      files = [ "/var/log/cim-expert/*.log" ];
      frequency = "daily";
      rotate = 30;
      compress = true;
      delaycompress = true;
      missingok = true;
      notifempty = true;
      create = "640 ${cfg.user} ${cfg.group}";
    };
    
    # NATS dependency (ensure NATS is available)
    services.nats = lib.mkDefault {
      enable = true;
      jetstream = true;
      settings = {
        port = 4222;
        http_port = 8222;
        jetstream = {
          store_dir = "/var/lib/nats/jetstream";
          max_memory_store = 1073741824; # 1GB
          max_file_store = 10737418240;  # 10GB
        };
      };
    };
  };
}