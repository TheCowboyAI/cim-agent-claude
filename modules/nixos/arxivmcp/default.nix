{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types;

  python = pkgs.python312;
  pythonPackages = python.pkgs;

  pythonEnv = python.withPackages (ps:
    with ps; [
      arxiv
      requests
      fastapi
      uvicorn
      pydantic
      typing-extensions
      python-multipart
      aiofiles
      aiohttp
      httpx
      mcp
      pydantic-settings
      python-dateutil
      python-dotenv
      sse-starlette
      pymupdf4llm  # This will use the native nixpkgs version 0.0.27
    ]);

  # Create HTTP wrapper for arxiv MCP server
  arxiv-http-wrapper = pkgs.writeShellScriptBin "arxiv-mcp-http" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    STORAGE_PATH="''${1:-/var/lib/arxivmcp/papers}"
    PORT="''${2:-8003}"
    
    echo "Starting ArXiv MCP server in HTTP mode on port $PORT..."
    echo "Storage path: $STORAGE_PATH"
    
    # Use uvicorn to run the server in HTTP mode
    exec ${pythonEnv}/bin/python -c "
import uvicorn
import sys
import os
sys.path.insert(0, '${config.services.arxivmcp.package}/${pythonEnv.sitePackages}')

# Set environment variables
os.environ['STORAGE_PATH'] = '$STORAGE_PATH'

# Import and run the MCP server with HTTP transport
try:
    from arxiv_mcp_server import create_app
    app = create_app()
    uvicorn.run(app, host='0.0.0.0', port=int('$PORT'), log_level='info')
except ImportError:
    print('Failed to import arxiv_mcp_server, falling back to stdio mode')
    import subprocess
    subprocess.run([
        '${pythonEnv}/bin/python', '-m', 'arxiv_mcp_server', 
        '--storage-path', '$STORAGE_PATH'
    ])
"
  '';
in {
  options.services.arxivmcp = {
    enable = mkEnableOption "arxivmcp";

    logLevel = mkOption {
      type = types.str;
      default = "INFO";
      description = "Logging level for arxivmcp server";
    };

    logFile = mkOption {
      type = types.path;
      default = "/var/lib/arxivmcp/log/server.log";
      description = "Path to log file";
    };

    storagePath = mkOption {
      type = types.path;
      default = "/var/lib/arxivmcp/papers";
      description = "Path where papers will be stored";
    };

    transport = mkOption {
      type = types.enum [ "stdio" "http" ];
      default = "http";
      description = "Transport type (stdio or http)";
    };

    port = mkOption {
      type = types.port;
      default = 8003;
      description = "Port to run the arxivmcp server on (used with http transport)";
    };

    package = mkOption {
      type = types.package;
      default = pythonPackages.buildPythonApplication rec {
        pname = "arxiv-mcp-server";
        version = "0.2.8";

        format = "pyproject";

        src = pkgs.fetchFromGitHub {
          owner = "blazickjp";
          repo = "arxiv-mcp-server";
          rev = "main";
          sha256 = "0dqighki3na398svpd79qlpj5i462i00zshzcmi0a1ch2z1jxf27";
        };

        nativeBuildInputs = with pythonPackages; [
          hatchling
          hatch-vcs
        ];

        propagatedBuildInputs = with pythonPackages; [
          arxiv
          requests
          fastapi
          uvicorn
          pydantic
          typing-extensions
          python-multipart
          aiofiles
          aiohttp
          httpx
          mcp
          pydantic-settings
          python-dateutil
          python-dotenv
          sse-starlette
          hatchling
          hatch-vcs
          pymupdf4llm  # Native nixpkgs version 0.0.27
          black
        ];

        doCheck = false;

        pythonImportsCheck = ["arxiv_mcp_server"];
      };
      description = "The arxivmcp package to use";
    };
  };

  config = lib.mkIf config.services.arxivmcp.enable {
    # Add packages to system
    environment.systemPackages = [ config.services.arxivmcp.package arxiv-http-wrapper ];

    # Open firewall port for HTTP transport
    networking.firewall.allowedTCPPorts = lib.optionals 
      (config.services.arxivmcp.transport == "http") 
      [ config.services.arxivmcp.port ];

    users.users.arxivmcp = {
      isSystemUser = true;
      group = "arxivmcp";
      home = "/var/lib/arxivmcp";
      createHome = true;
    };

    users.groups.arxivmcp = {};

    systemd.tmpfiles.rules = [
      "d /var/lib/arxivmcp/log 0750 arxivmcp arxivmcp -"
      "d /var/lib/arxivmcp/papers 0750 arxivmcp arxivmcp -"
    ];

    systemd.services.arxivmcp = {
      description = "ArXiv MCP Server";
      after = ["network.target"];
      wantedBy = ["multi-user.target"];

      serviceConfig = {
        Type = "simple";
        ExecStart = if config.services.arxivmcp.transport == "http" 
          then "${arxiv-http-wrapper}/bin/arxiv-mcp-http ${config.services.arxivmcp.storagePath} ${toString config.services.arxivmcp.port}"
          else "${pythonEnv}/bin/python -m arxiv_mcp_server --storage-path ${config.services.arxivmcp.storagePath}";
        Restart = "always";
        RestartSec = "5s";
        User = "arxivmcp";
        Group = "arxivmcp";
        RuntimeDirectory = "arxivmcp";
        StateDirectory = "arxivmcp";
        LogsDirectory = "arxivmcp";
        CacheDirectory = "arxivmcp";
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        NoNewPrivileges = true;
        RestrictSUIDSGID = true;
        PrivateDevices = false; # HTTP server needs device access
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        ProtectKernelModules = true;
        PrivateNetwork = false; # HTTP server needs network access
        ReadWritePaths = [
          "/var/lib/arxivmcp"
        ];
        WorkingDirectory = "/var/lib/arxivmcp";
      };

      environment = {
        LOG_LEVEL = config.services.arxivmcp.logLevel;
        LOG_FILE = config.services.arxivmcp.logFile;
        PYTHONPATH = "${config.services.arxivmcp.package}/${pythonEnv.sitePackages}";
        STORAGE_PATH = config.services.arxivmcp.storagePath;
      } // lib.optionalAttrs (config.services.arxivmcp.transport == "http") {
        PORT = toString config.services.arxivmcp.port;
        HOST = "0.0.0.0";
      };
    };
  };
}
