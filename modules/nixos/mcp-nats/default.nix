{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types;
  
  # Build mcp-nats properly
  mcp-nats = pkgs.buildGoModule rec {
    pname = "mcp-nats";
    version = "0.1.0";

    src = pkgs.fetchFromGitHub {
      owner = "sinadarbouy";
      repo = "mcp-nats";
      rev = "cb7878c9565613ff4f0a1bf08d09f03824d21da7";
      sha256 = "08l6hkxpc898hz44mkvn8mdnql1zvrc6zkm8770sa893piik2lbz";
    };

    vendorHash = "sha256-OTb/6XtQ0JEOLs9ZBTGr90EgxZp5UvALXVTLjJSWHXk=";

    subPackages = [ "cmd/mcp-nats" ];

    # Skip vendor verification to avoid issues
    proxyVendor = true;
    
    # Add CGO_ENABLED=0 for static build
    env.CGO_ENABLED = "0";

    ldflags = [
      "-s"
      "-w"
    ];

    meta = with lib; {
      description = "Model Context Protocol (MCP) server for NATS messaging system integration";
      homepage = "https://github.com/sinadarbouy/mcp-nats";
      license = licenses.mit;
      maintainers = [];
      platforms = platforms.linux;
    };
  };
in {
  options.services.mcp-nats = {
    enable = mkEnableOption "mcp-nats";

    logLevel = mkOption {
      type = types.str;
      default = "info";
      description = "Logging level for mcp-nats server";
    };

    transport = mkOption {
      type = types.enum [ "stdio" "sse" ];
      default = "sse";
      description = "Transport type (stdio or sse)";
    };

    port = mkOption {
      type = types.port;
      default = 8002;
      description = "Port to run the mcp-nats server on (used with sse or http transport)";
    };

    natsUrl = mkOption {
      type = types.str;
      default = "nats://localhost:4222";
      description = "NATS server URL";
    };

    noAuthentication = mkOption {
      type = types.bool;
      default = true;
      description = "Allow anonymous connections to NATS";
    };

    natsUser = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "NATS username for authentication";
    };

    natsPassword = mkOption {
      type = types.nullOr types.str;
      default = null;
      description = "NATS password for authentication";
    };

    package = mkOption {
      type = types.package;
      default = mcp-nats;
      description = "The mcp-nats package to use";
    };
  };

  config = lib.mkIf config.services.mcp-nats.enable {
    # Add mcp-nats to system packages
    environment.systemPackages = [ config.services.mcp-nats.package ];

    # Open firewall port for SSE transport
    networking.firewall.allowedTCPPorts = lib.optionals 
      (config.services.mcp-nats.transport == "sse") 
      [ config.services.mcp-nats.port ];

    # Ensure directories exist
    systemd.tmpfiles.rules = [
      "d /var/log/mcp-nats 0755 mcp-nats mcp-nats -"
      "d /var/lib/mcp-nats 0755 mcp-nats mcp-nats -"
    ];

    systemd.services.mcp-nats = {
      description = "MCP NATS Server";
      after = ["network.target" "nats.service"];
      wants = ["nats.service"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        Type = "simple";
        ExecStart = let
          args = [
            "--transport ${config.services.mcp-nats.transport}"
            "--log-level ${config.services.mcp-nats.logLevel}"
          ] ++ lib.optionals config.services.mcp-nats.noAuthentication [
            "--no-authentication"
          ] ++ lib.optionals (config.services.mcp-nats.transport == "sse") [
            "-sse-address" "0.0.0.0:${toString config.services.mcp-nats.port}"
          ];
        in "${config.services.mcp-nats.package}/bin/mcp-nats ${lib.concatStringsSep " " args}";
        
        Restart = "on-failure";
        RestartSec = "5s";
        RuntimeDirectory = "mcp-nats";
        RuntimeDirectoryMode = "0755";
        StateDirectory = "mcp-nats";
        StateDirectoryMode = "0755";
        User = "mcp-nats";
        Group = "mcp-nats";
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        NoNewPrivileges = true;
        PrivateNetwork = false; # Need network access for NATS
        ReadWritePaths = [
          "/var/log/mcp-nats"
          "/var/lib/mcp-nats"
        ];
        WorkingDirectory = "/var/lib/mcp-nats";
      };

      environment = {
        NATS_URL = config.services.mcp-nats.natsUrl;
        LOG_LEVEL = config.services.mcp-nats.logLevel;
      } // lib.optionalAttrs (config.services.mcp-nats.natsUser != null) {
        NATS_USER = config.services.mcp-nats.natsUser;
      } // lib.optionalAttrs (config.services.mcp-nats.natsPassword != null) {
        NATS_PASSWORD = config.services.mcp-nats.natsPassword;
      } // lib.optionalAttrs config.services.mcp-nats.noAuthentication {
        NATS_NO_AUTHENTICATION = "true";
      };
    };

    # Create user and group for mcp-nats
    users.users.mcp-nats = {
      isSystemUser = true;
      group = "mcp-nats";
      home = "/var/lib/mcp-nats";
      createHome = true;
    };

    users.groups.mcp-nats = {};
  };
}