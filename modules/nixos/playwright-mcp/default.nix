{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types;
  
  cfg = config.services.playwright-mcp;
  
  # Create a wrapper script that handles browser installation and startup
  playwright-mcp-wrapper = pkgs.writeShellScriptBin "playwright-mcp-wrapped" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    # Setup environment
    export HOME="/var/lib/playwright-mcp"
    export PLAYWRIGHT_BROWSERS_PATH="$HOME/.cache/ms-playwright"
    export PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD="${if cfg.useBundledBrowsers then "false" else "true"}"
    
    # Create cache directory
    mkdir -p "$HOME/.cache/ms-playwright"
    
    # Install browsers on first run if using bundled browsers
    if [[ "${toString cfg.useBundledBrowsers}" == "1" && ! -d "$HOME/.cache/ms-playwright/chromium-*" ]]; then
      echo "Installing Playwright browsers..."
      ${pkgs.playwright-driver}/bin/playwright install chromium || {
        echo "Warning: Failed to install bundled browsers, will use system browsers"
      }
    fi
    
    # Run the MCP server in HTTP mode
    echo "Starting Playwright MCP server on http://localhost:${toString cfg.port}..."
    exec ${pkgs.playwright-mcp}/bin/mcp-server-playwright --port ${toString cfg.port}
  '';
in {
  options.services.playwright-mcp = {
    enable = mkEnableOption "playwright-mcp";

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = "Logging level for playwright-mcp server";
    };

    useBundledBrowsers = mkOption {
      type = types.bool;
      default = true;
      description = "Whether to download and use bundled Playwright browsers";
    };

    port = mkOption {
      type = types.port;
      default = 8080;
      description = "Port for the HTTP MCP server";
    };

    package = mkOption {
      type = types.package;
      default = pkgs.playwright-mcp;
      description = "The playwright-mcp package to use";
    };
  };

  config = lib.mkIf cfg.enable {
    # Ensure playwright-mcp package is available
    environment.systemPackages = [ cfg.package playwright-mcp-wrapper ];

    # Open firewall port for HTTP service
    networking.firewall.allowedTCPPorts = [ cfg.port ];

    # Add fonts and graphics libraries for browser rendering
    fonts.packages = with pkgs; [
      dejavu_fonts
      liberation_ttf
      ubuntu_font_family
    ];

    # Enable necessary programs for browser support
    programs.nix-ld = {
      enable = true;
      libraries = with pkgs; [
        stdenv.cc.cc.lib
        zlib
        # Browser dependencies
        at-spi2-atk
        atk
        cairo
        cups
        dbus
        expat
        fontconfig
        freetype
        gdk-pixbuf
        glib
        gtk3
        libdrm
        mesa
        nspr
        nss
        pango
        # X11 libraries
        xorg.libX11
        xorg.libXcomposite
        xorg.libXdamage
        xorg.libXext
        xorg.libXfixes
        xorg.libXrandr
        xorg.libXrender
        xorg.libXtst
        xorg.libxcb
        xorg.libxshmfence
        xorg.libXScrnSaver
      ];
    };

    # Create required directories
    systemd.tmpfiles.rules = [
      "d /var/lib/playwright-mcp 0755 playwright-mcp playwright-mcp -"
      "d /var/lib/playwright-mcp/.cache 0755 playwright-mcp playwright-mcp -"
      "d /var/lib/playwright-mcp/.cache/ms-playwright 0755 playwright-mcp playwright-mcp -"
      "d /var/log/playwright-mcp 0755 playwright-mcp playwright-mcp -"
    ];

    systemd.services.playwright-mcp = {
      description = "Playwright MCP Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${playwright-mcp-wrapper}/bin/playwright-mcp-wrapped";
        Restart = "always";
        RestartSec = 5;
        User = "playwright-mcp";
        Group = "playwright-mcp";
        
        # Directory management
        StateDirectory = "playwright-mcp";
        StateDirectoryMode = "0755";
        LogsDirectory = "playwright-mcp";
        LogsDirectoryMode = "0755";
        WorkingDirectory = "/var/lib/playwright-mcp";
        
        # Security settings
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        NoNewPrivileges = true;
        
        # Network and devices - HTTP server needs network access
        PrivateNetwork = false;
        PrivateDevices = false;
        
        # File system access
        ReadWritePaths = [
          "/var/lib/playwright-mcp"
          "/var/log/playwright-mcp"
        ];
        
        # Resource limits
        MemoryMax = "2G";
        TasksMax = 100;
      };

      environment = {
        # Logging
        LOG_LEVEL = cfg.logLevel;
        
        # HTTP Server configuration
        PORT = toString cfg.port;
        HOST = "0.0.0.0";
        
        # Playwright configuration
        PLAYWRIGHT_BROWSERS_PATH = "/var/lib/playwright-mcp/.cache/ms-playwright";
        PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = toString (!cfg.useBundledBrowsers);
        
        # System configuration
        HOME = "/var/lib/playwright-mcp";
        NODE_ENV = "production";
        
        # Library paths for browsers
        NIX_LD_LIBRARY_PATH = lib.makeLibraryPath [
          pkgs.stdenv.cc.cc.lib
          pkgs.zlib
          pkgs.fontconfig
          pkgs.freetype
          pkgs.glib
          pkgs.gtk3
          pkgs.cairo
          pkgs.pango
          pkgs.atk
          pkgs.gdk-pixbuf
          pkgs.xorg.libX11
          pkgs.xorg.libXext
          pkgs.xorg.libXrender
          pkgs.xorg.libXtst
          pkgs.xorg.libxcb
          pkgs.mesa
          pkgs.nss
          pkgs.nspr
        ];
        NIX_LD = lib.fileContents "${pkgs.stdenv.cc}/nix-support/dynamic-linker";
      };
    };

    # Create user and group for playwright-mcp
    users.users.playwright-mcp = {
      isSystemUser = true;
      group = "playwright-mcp";
      home = "/var/lib/playwright-mcp";
      createHome = true;
    };

    users.groups.playwright-mcp = {};
  };
}