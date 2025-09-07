{pkgs, lib, inputs, ...}: {
  # Base Host config
  system.stateVersion = "23.11";

  # TIME
  time.timeZone = lib.mkDefault "America/Phoenix";

  # Select internationalisation properties.
  i18n.defaultLocale = lib.mkDefault "en_US.UTF-8";

  nixpkgs.config.allowUnfree = true;

  # Nix limits - fixes too many open files.it
  systemd.services.nix-daemon.serviceConfig.LimitNOFILE = lib.mkForce 131072;
  boot.kernel.sysctl = { "fs.file-max" = 524288; };
  security.pam.loginLimits = [
    { domain = "*"; type = "soft"; item = "nofile"; value = "131072"; }
    { domain = "*"; type = "hard"; item = "nofile"; value = "131072"; }
  ];

  environment.variables = lib.mkDefault {
    BROWSER = "${pkgs.google-chrome}/bin/google-chrome-stable";
    MOZ_ENABLE_WAYLAND = "1";
    WAYLAND_DISPLAY = "1";
    EDITOR = "hx";
    DIRENV_LOG_FORMAT = "";
    ANKI_WAYLAND = "1";
    DISABLE_QT5_COMPAT = "0";
    WINIT_UNIX_BACKEND = "wayland";
    VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
  };

  # Ensure hyprshot is in PATH
  environment.sessionVariables = {
    PATH = [
      "$HOME/.local/bin"
      "$PATH"
      "${pkgs.hyprshot}/bin"
    ];
  };

  # let me change configs and use cachix
  nix.settings.trusted-users = ["root" "steele"];
  nix.settings.auto-optimise-store = false; # Temporarily disabled - causes vendor directory race conditions
  nix.gc = {
    automatic = true;
    randomizedDelaySec = "14m";
    options = "--delete-older-than 10d";
  };

  code-cursor.enable = true;
  services.claude-code = {
    enable = true;
    apiKeyFile = "/claude-code/anthropic.key";
  };
  services.playwright-mcp.enable = true;
  services.mcp-nats = {
    enable = true;
    transport = "sse";
    port = 8002;
  };
  # TheCowboyAI arxiv-mcp-server fork - installed as binary only
  
  # New HTTP-compatible MCP servers - disabled due to connection issues
  # services.research-mcp = {
  #   enable = true;
  #   port = 8005;
  #   logLevel = "info";
  # };
  # services.mcp-nixos-server = {
  #   enable = true;
  #   port = 8004;
  #   logLevel = "info";
  # };
  # services.filesystem-mcp = {
  #   enable = true;
  #   port = 8001;
  #   rootPath = "/home/steele";
  # };
  # services.cim-sage-local.enable = false; # Disabled - using direct packages instead

  # New utensils/mcp-nixos server for NixOS package information
  services.utensils-mcp-nixos = {
    enable = true;
    transport = "stdio";
    logLevel = "INFO";
  };

  # Install CIM SAGE with proper runtime wrapper and MCP servers using proper Nix packages
  environment.systemPackages = with pkgs; [
    
    # CIM SAGE packages
    sage
    (writeShellScriptBin "sage-wayland" ''
      # Fix Wayland display socket - Hyprland uses wayland-1
      export WAYLAND_DISPLAY="wayland-1"
      export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR}"
      export LD_LIBRARY_PATH="${lib.makeLibraryPath [
        wayland
        wayland-protocols  
        libxkbcommon
        libGL
        vulkan-loader
      ]}:''${LD_LIBRARY_PATH:-}"
      
      # Reduce log verbosity - only show warnings and errors
      export RUST_LOG="warn,sage=info"
      
      exec ${sage}/bin/sage "$@"
    '')
    (writeShellScriptBin "sage-quiet" ''
      # Fix Wayland display socket - Hyprland uses wayland-1
      export WAYLAND_DISPLAY="wayland-1"
      export XDG_RUNTIME_DIR="''${XDG_RUNTIME_DIR}"
      export LD_LIBRARY_PATH="${lib.makeLibraryPath [
        wayland
        wayland-protocols  
        libxkbcommon
        libGL
        vulkan-loader
      ]}:''${LD_LIBRARY_PATH:-}"
      
      # Suppress all debug output, only show errors
      export RUST_LOG="error"
      
      exec ${sage}/bin/sage "$@" 2>/dev/null
    '')
  ];

  # CIM SAGE systemd service - DISABLED FOR MAJOR UPDATE
  systemd.services.cim-sage = {
    enable = false;
    description = "CIM SAGE - Universal Agent System";
    after = [ "network.target" "nats.service" ];
    wants = [ "nats.service" ];
    wantedBy = [ "multi-user.target" ];
    
    # Load the API key using systemd credentials
    script = ''
      if [ -f /etc/cim-claude/api-key.txt ]; then
        export ANTHROPIC_API_KEY=$(cat /etc/cim-claude/api-key.txt)
      fi
      exec ${pkgs.sage-service}/bin/sage-service-wrapped
    '';
    
    environment = {
      SAGE_NAME = "sage-dell-62S6063";
      SAGE_ENVIRONMENT = "development";
      NATS_URL = "nats://localhost:4222";
      SAGE_PORT = "8080";
      # Enhanced logging and performance
      RUST_LOG = "info,sage=debug";
      SAGE_LOG_FORMAT = "json";
      SAGE_METRICS_ENABLED = "true";
    };
    
    serviceConfig = {
      Type = "simple";
      User = "sage";
      Group = "sage";
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
  
  # NATS server for SAGE - already provided elsewhere
  # services.nats = {
  #   enable = true;
  #   jetstream = true;
  # };


  services.yubikey = {
    enable = true;
    
    # Disable specific components as needed
    oath.enable = false;  # Disable OATH/TOTP support
    
    # Enable development tools
    development.enable = true;
    
    # Configure PAM
    pam = {
      enable = true;
    };
  };
}
