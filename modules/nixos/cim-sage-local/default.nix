{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-sage-local;
  
  # For now, we'll use the packages from the upstream cim-sage flake
  # until we can properly integrate naersk
  sage-service = pkgs.sage-service or null;
  sage-gui = pkgs.sage or null;
  
in {
  options.services.cim-sage-local = {
    enable = mkEnableOption "CIM SAGE Universal Agent System (local build)";
    
    port = mkOption {
      type = types.int;
      default = 8080;
      description = "Port for SAGE service";
    };
    
    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };
  };
  
  config = mkIf cfg.enable {
    # Install packages
    environment.systemPackages = lib.optional (sage-gui != null) sage-gui ++ 
      lib.optional (sage-service != null) sage-service ++ [
      (pkgs.writeShellScriptBin "sage-wayland" ''
        export WAYLAND_DISPLAY="''${WAYLAND_DISPLAY}"
        export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR}"
        export LD_LIBRARY_PATH="${lib.makeLibraryPath [
          pkgs.wayland
          pkgs.wayland-protocols  
          pkgs.libxkbcommon
          pkgs.libGL
          pkgs.vulkan-loader
        ]}:''${LD_LIBRARY_PATH:-}"
        exec ${sage-gui}/bin/sage "$@"
      '')
    ];
    
    # SAGE service
    systemd.services.cim-sage-local = {
      enable = true;
      description = "CIM SAGE - Universal Agent System (local build)";
      after = [ "network.target" "nats.service" ];
      wants = [ "nats.service" ];
      wantedBy = [ "multi-user.target" ];
      
      environment = {
        SAGE_NAME = "sage-${config.networking.hostName}";
        SAGE_ENVIRONMENT = "development";
        NATS_URL = cfg.natsUrl;
        ANTHROPIC_API_KEY_FILE = "/etc/cim-claude/api-key.txt";
        SAGE_PORT = toString cfg.port;
        # Enhanced logging and performance matching main config
        RUST_LOG = "info,sage=debug";
        SAGE_LOG_FORMAT = "json";
        SAGE_METRICS_ENABLED = "true";
      };
      
      serviceConfig = {
        Type = "simple";
        User = "sage";
        Group = "sage";
        ExecStart = lib.mkIf (sage-service != null) "${sage-service}/bin/sage-service";
        Restart = "always";
        RestartSec = "10";
        
        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
      };
    };
    
    # Create sage user and group
    users.users.sage = {
      isSystemUser = true;
      group = "sage";
      description = "CIM SAGE service user";
    };
    users.groups.sage = {};
    
    # Enable NATS
    services.nats = {
      enable = true;
      jetstream = true;
    };
  };
}
