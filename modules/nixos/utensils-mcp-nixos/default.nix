{ config, lib, pkgs, ... }:

let
  cfg = config.services.utensils-mcp-nixos;
  
  # Create a custom fastmcp package without runtime dependency checks
  fastmcp-fixed = pkgs.python312Packages.fastmcp.overridePythonAttrs (old: {
    dontCheckRuntimeDeps = true;
    pythonImportsCheck = [ ];
    doCheck = false;
  });

  # Import the mcp-nixos package from the utensils repo using the existing flake
  mcp-nixos-pkg = pkgs.python312Packages.buildPythonApplication rec {
    pname = "mcp-nixos";
    version = "1.0.0";
    format = "pyproject";

    src = pkgs.fetchFromGitHub {
      owner = "utensils";
      repo = "mcp-nixos";
      rev = "46b4d4d3d6421bfbadc415532ef74433871e1cda";
      hash = "sha256-iWhsf1Myk6RyQ7IuNf4bWI3Sqq9pgmhKvEisCXtkxyw=";
    };

    nativeBuildInputs = with pkgs.python312Packages; [
      hatchling
    ];

    propagatedBuildInputs = with pkgs.python312Packages; [
      fastmcp-fixed
      requests
      beautifulsoup4
      openapi-core
      # Add other dependencies that might be missing
      httpx
      pydantic
      typing-extensions
    ];

    # Disable checks since they may require network access
    doCheck = false;
    pythonImportsCheck = [ ];
    dontCheckRuntimeDeps = true;

    meta = with lib; {
      description = "Model Context Protocol server for NixOS and Home Manager resources";
      homepage = "https://github.com/utensils/mcp-nixos";
      license = licenses.mit;
      maintainers = [ ];
      mainProgram = "mcp-nixos";
    };
  };

in
{
  options.services.utensils-mcp-nixos = {
    enable = lib.mkEnableOption "Utensils MCP-NixOS server for package and configuration information";

    transport = lib.mkOption {
      type = lib.types.enum [ "stdio" "http" "sse" ];
      default = "stdio";
      description = "Transport protocol to use";
    };

    port = lib.mkOption {
      type = lib.types.port;
      default = 8006;
      description = "Port for HTTP/SSE transport";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Host to bind to for HTTP/SSE transport";
    };

    logLevel = lib.mkOption {
      type = lib.types.enum [ "DEBUG" "INFO" "WARNING" "ERROR" ];
      default = "INFO";
      description = "Log level for the server";
    };
  };

  config = lib.mkIf cfg.enable {
    # Install the package system-wide
    environment.systemPackages = [ mcp-nixos-pkg ];

    systemd.services.utensils-mcp-nixos = {
      description = "Utensils MCP-NixOS Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      environment = {
        PYTHONPATH = "${mcp-nixos-pkg}/${pkgs.python312.sitePackages}";
        MCP_NIXOS_ENV = "production";
      };

      serviceConfig = {
        Type = "simple";
        User = "mcp-nixos";
        Group = "mcp-nixos";
        Restart = "always";
        RestartSec = "5s";
        StateDirectory = "mcp-nixos";
        StateDirectoryMode = "0755";

        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;

        # Allow read access to Nix store and system configuration
        ReadOnlyPaths = [ "/nix/store" "/etc/nixos" "/run/current-system" ];
      };

      script = let
        args = if cfg.transport == "stdio" then
          []
        else if cfg.transport == "http" then
          [ "--transport" "http" "--host" cfg.host "--port" (toString cfg.port) ]
        else # sse
          [ "--transport" "sse" "--host" cfg.host "--port" (toString cfg.port) ];
      in ''
        exec ${mcp-nixos-pkg}/bin/mcp-nixos ${lib.concatStringsSep " " args}
      '';
    };

    users.users.mcp-nixos = {
      isSystemUser = true;
      group = "mcp-nixos";
      description = "MCP NixOS server user";
    };
    users.groups.mcp-nixos = {};

    # Open firewall port if using HTTP/SSE transport
    networking.firewall.allowedTCPPorts = lib.mkIf (cfg.transport != "stdio") [ cfg.port ];
  };
}