{ config, lib, pkgs, ... }:

let
  cfg = config.services.research-mcp;
  
  # Create Python environment with required dependencies
  pythonEnv = pkgs.python3.withPackages (ps: with ps; [
    fastapi
    uvicorn
    requests
    feedparser
    beautifulsoup4
    python-dateutil
    aiohttp
    pydantic
  ]);
  
  # Create the Python server script in the Nix store
  serverScript = pkgs.writeScript "research-mcp-server.py" ''
    #!${pythonEnv}/bin/python3
    
    ${builtins.readFile ./server.py}
  '';
  
in

{
  options.services.research-mcp = {
    enable = lib.mkEnableOption "Research MCP Server for ArXiv and academic papers";
    
    port = lib.mkOption {
      type = lib.types.port;
      default = 8005;
      description = "Port to run the MCP server on";
    };
    
    host = lib.mkOption {
      type = lib.types.str;
      default = "0.0.0.0";
      description = "Host to bind the server to";
    };
    
    logLevel = lib.mkOption {
      type = lib.types.enum [ "debug" "info" "warn" "error" ];
      default = "info";
      description = "Log level for the server";
    };
    
    cachePath = lib.mkOption {
      type = lib.types.path;
      default = "/var/lib/research-mcp/cache";
      description = "Directory to cache downloaded papers";
    };
  };

  config = lib.mkIf cfg.enable {
    # Create the Research MCP server systemd service
    systemd.services.research-mcp = {
      description = "Research MCP Server for ArXiv and academic papers";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = "research-mcp";
        Group = "research-mcp";
        Restart = "always";
        RestartSec = "5s";
        StateDirectory = "research-mcp";
        StateDirectoryMode = "0755";
        CacheDirectory = "research-mcp";
        
        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        
        # Allow network access and cache directory
        PrivateNetwork = false;
        ReadWritePaths = [ "/var/lib/research-mcp" "/var/cache/research-mcp" ];
      };

      script = ''
        exec ${serverScript} \
          --host ${cfg.host} \
          --port ${toString cfg.port} \
          --cache-path ${cfg.cachePath} \
          --log-level ${cfg.logLevel}
      '';
    };

    # Create user and group for the service
    users.users.research-mcp = {
      isSystemUser = true;
      group = "research-mcp";
      description = "Research MCP server user";
    };
    users.groups.research-mcp = {};

    # Open firewall port
    networking.firewall.allowedTCPPorts = [ cfg.port ];
  };
}