{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.claude-adapter;
  
  # Build the Rust package
  claude-adapter = pkgs.rustPlatform.buildRustPackage rec {
    pname = "simple-claude-adapter";
    version = "0.1.0";
    
    src = ./.;
    
    cargoLock = {
      lockFile = ./Cargo.lock;
    };
    
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    
    buildInputs = with pkgs; [
      openssl
    ];
    
    meta = with lib; {
      description = "Claude API to NATS adapter";
      license = licenses.mit;
      maintainers = [ ];
    };
  };

in {
  options.services.claude-adapter = {
    enable = mkEnableOption "Claude API to NATS adapter service";
    
    claudeApiKey = mkOption {
      type = types.str;
      description = "Claude API key from Anthropic";
      example = "sk-ant-api03-...";
    };
    
    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };
    
    logLevel = mkOption {
      type = types.enum [ "error" "warn" "info" "debug" "trace" ];
      default = "info";
      description = "Log level for the service";
    };
    
    user = mkOption {
      type = types.str;
      default = "claude-adapter";
      description = "User to run the service as";
    };
    
    group = mkOption {
      type = types.str;
      default = "claude-adapter";
      description = "Group to run the service as";
    };
  };

  config = mkIf cfg.enable {
    # Create user and group
    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "Claude adapter service user";
    };
    
    users.groups.${cfg.group} = {};

    # Systemd service
    systemd.services.claude-adapter = {
      description = "Claude API to NATS adapter";
      after = [ "network.target" ];
      wants = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      
      environment = {
        CLAUDE_API_KEY = cfg.claudeApiKey;
        NATS_URL = cfg.natsUrl;
        RUST_LOG = cfg.logLevel;
      };
      
      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${claude-adapter}/bin/simple-claude-adapter";
        Restart = "always";
        RestartSec = "5";
        
        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictSUIDSGID = true;
        RestrictRealtime = true;
        RestrictNamespaces = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        SystemCallArchitectures = "native";
      };
    };

    # Add to system packages
    environment.systemPackages = [ claude-adapter ];
  };
}