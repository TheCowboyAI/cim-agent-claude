{
  description = "TheCowboyAI arxiv-mcp-server - NixOS module";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        pythonEnv = pkgs.python311.withPackages (ps: with ps; [
          # Core dependencies from pyproject.toml
          arxiv
          httpx
          python-dateutil
          pydantic
          mcp
          aiohttp
          python-dotenv
          pydantic-settings
          aiofiles
          uvicorn
          anyio
          black
          
          # Additional dependencies that might be needed
          setuptools
          wheel
          pip
        ]);
        
        arxiv-mcp-server = pkgs.stdenv.mkDerivation rec {
          pname = "arxiv-mcp-server";
          version = "0.3.1";
          
          src = pkgs.fetchFromGitHub {
            owner = "TheCowboyAI";
            repo = "arxiv-mcp-server";
            rev = "057e2000be7b56823239815b0fe7c7fc0dbced96";
            sha256 = "R7guwxeQBQViZR/qD0AUhsQiL8XptLs1SkPZESd8ETc=";
          };
          
          nativeBuildInputs = [ pythonEnv pkgs.makeWrapper ];
          
          installPhase = ''
            mkdir -p $out/lib/python${pythonEnv.python.pythonVersion}/site-packages
            cp -r src/arxiv_mcp_server $out/lib/python${pythonEnv.python.pythonVersion}/site-packages/
            
            mkdir -p $out/bin
            makeWrapper ${pythonEnv}/bin/python $out/bin/arxiv-mcp-server \
              --add-flags "-m arxiv_mcp_server" \
              --set PYTHONPATH "$out/lib/python${pythonEnv.python.pythonVersion}/site-packages:$PYTHONPATH"
          '';
          
          meta = with pkgs.lib; {
            description = "A flexible arXiv search and analysis service with MCP protocol support";
            homepage = "https://github.com/TheCowboyAI/arxiv-mcp-server";
            license = licenses.mit;
            maintainers = [ ];
            platforms = platforms.unix;
          };
        };
        
      in
      {
        packages.default = arxiv-mcp-server;
        packages.arxiv-mcp-server = arxiv-mcp-server;
        
        devShells.default = pkgs.mkShell {
          buildInputs = [ pythonEnv ];
        };
      }
    ) // {
      # NixOS module
      nixosModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.services.arxiv-mcp-cowboy;
        in
        {
          options.services.arxiv-mcp-cowboy = {
            enable = lib.mkEnableOption "TheCowboyAI ArXiv MCP Server";
            
            storagePath = lib.mkOption {
              type = lib.types.path;
              default = "/var/lib/arxiv-mcp/papers";
              description = "Directory to store downloaded papers";
            };
            
            transport = lib.mkOption {
              type = lib.types.enum [ "stdio" "http" "sse" ];
              default = "stdio";
              description = "Transport protocol to use";
            };
            
            port = lib.mkOption {
              type = lib.types.port;
              default = 8003;
              description = "Port for HTTP/SSE transport";
            };
            
            host = lib.mkOption {
              type = lib.types.str;
              default = "127.0.0.1";
              description = "Host to bind to for HTTP/SSE transport";
            };
          };
          
          config = lib.mkIf cfg.enable {
            systemd.services.arxiv-mcp-cowboy = {
              description = "TheCowboyAI ArXiv MCP Server";
              after = [ "network.target" ];
              wantedBy = [ "multi-user.target" ];
              
              environment = {
                ARXIV_STORAGE_PATH = cfg.storagePath;
              };
              
              serviceConfig = {
                Type = "simple";
                User = "arxiv-mcp";
                Group = "arxiv-mcp";
                Restart = "always";
                RestartSec = "5s";
                StateDirectory = "arxiv-mcp";
                StateDirectoryMode = "0755";
                
                # Security hardening
                NoNewPrivileges = true;
                ProtectSystem = "strict";
                ProtectHome = true;
                PrivateTmp = true;
                ProtectKernelTunables = true;
                ProtectKernelModules = true;
                ProtectControlGroups = true;
                
                ReadWritePaths = [ cfg.storagePath ];
              };
              
              script = let
                package = self.packages.${pkgs.system}.arxiv-mcp-server;
              in ''
                exec ${package}/bin/arxiv-mcp-server
              '';
            };
            
            users.users.arxiv-mcp = {
              isSystemUser = true;
              group = "arxiv-mcp";
              description = "ArXiv MCP server user";
            };
            users.groups.arxiv-mcp = {};
            
            # Open firewall port if using HTTP/SSE transport
            networking.firewall.allowedTCPPorts = lib.mkIf (cfg.transport != "stdio") [ cfg.port ];
          };
        };
    };
}