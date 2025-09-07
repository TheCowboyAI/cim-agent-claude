{ config, lib, pkgs, ... }:

let
  cfg = config.services.mcp-nixos-server;
in

{
  options.services.mcp-nixos-server = {
    enable = lib.mkEnableOption "MCP NixOS Server for system administration";
    
    port = lib.mkOption {
      type = lib.types.port;
      default = 8004;
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
  };

  config = lib.mkIf cfg.enable {
    # Install the MCP NixOS server package
    environment.systemPackages = with pkgs; [
      (python3.withPackages (ps: with ps; [
        fastapi
        uvicorn
        mcp
      ]))
    ];

    # Create the MCP NixOS server systemd service
    systemd.services.mcp-nixos-server = {
      description = "MCP NixOS Server for system administration";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        User = "mcp-nixos";
        Group = "mcp-nixos";
        Restart = "always";
        RestartSec = "5s";
        
        # Security hardening
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        PrivateTmp = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        
        # Allow access to Nix store and system paths
        ReadWritePaths = [ "/var/lib/mcp-nixos" ];
        ReadOnlyPaths = [ "/nix/store" "/etc/nixos" ];
      };

      script = ''
        exec ${pkgs.python3}/bin/python3 -c "
import asyncio
import json
import subprocess
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

from fastapi import FastAPI, HTTPException
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.types import Resource, Tool
import uvicorn

app = FastAPI(title='MCP NixOS Server', version='1.0.0')

class NixOSMCPServer:
    def __init__(self):
        self.available_tools = [
            {
                'name': 'nixos_rebuild',
                'description': 'Rebuild NixOS system configuration',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'action': {
                            'type': 'string',
                            'enum': ['switch', 'boot', 'test', 'build'],
                            'description': 'Rebuild action to perform'
                        },
                        'flake_ref': {
                            'type': 'string',
                            'description': 'Flake reference (optional)'
                        }
                    },
                    'required': ['action']
                }
            },
            {
                'name': 'nix_search',
                'description': 'Search for Nix packages',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'query': {
                            'type': 'string',
                            'description': 'Search query for packages'
                        }
                    },
                    'required': ['query']
                }
            },
            {
                'name': 'systemctl_status',
                'description': 'Get systemd service status',
                'input_schema': {
                    'type': 'object',
                    'properties': {
                        'service': {
                            'type': 'string',
                            'description': 'Service name to check'
                        }
                    },
                    'required': ['service']
                }
            }
        ]
        
    async def handle_tool_call(self, tool_name: str, arguments: Dict[str, Any]) -> Dict[str, Any]:
        try:
            if tool_name == 'nixos_rebuild':
                return await self._nixos_rebuild(arguments)
            elif tool_name == 'nix_search':
                return await self._nix_search(arguments)
            elif tool_name == 'systemctl_status':
                return await self._systemctl_status(arguments)
            else:
                raise HTTPException(status_code=400, detail=f'Unknown tool: {tool_name}')
        except Exception as e:
            return {'error': str(e)}
    
    async def _nixos_rebuild(self, args: Dict[str, Any]) -> Dict[str, Any]:
        action = args['action']
        flake_ref = args.get('flake_ref', '.')
        
        cmd = ['sudo', 'nixos-rebuild', action]
        if flake_ref != '.':
            cmd.extend(['--flake', flake_ref])
            
        result = subprocess.run(cmd, capture_output=True, text=True)
        return {
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }
    
    async def _nix_search(self, args: Dict[str, Any]) -> Dict[str, Any]:
        query = args['query']
        cmd = ['nix', 'search', 'nixpkgs', query]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        return {
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }
    
    async def _systemctl_status(self, args: Dict[str, Any]) -> Dict[str, Any]:
        service = args['service']
        cmd = ['systemctl', 'status', service]
        
        result = subprocess.run(cmd, capture_output=True, text=True)
        return {
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }

server = NixOSMCPServer()

@app.get('/mcp/tools')
async def list_tools():
    return {'tools': server.available_tools}

@app.post('/mcp/tools/{tool_name}')
async def call_tool(tool_name: str, arguments: Dict[str, Any]):
    result = await server.handle_tool_call(tool_name, arguments)
    return {'result': result}

@app.get('/health')
async def health_check():
    return {'status': 'healthy'}

if __name__ == '__main__':
    uvicorn.run(app, host='${cfg.host}', port=${toString cfg.port}, log_level='${cfg.logLevel}')
"
      '';
    };

    # Create user and group for the service
    users.users.mcp-nixos = {
      isSystemUser = true;
      group = "mcp-nixos";
      description = "MCP NixOS server user";
    };
    users.groups.mcp-nixos = {};

    # Open firewall port
    networking.firewall.allowedTCPPorts = [ cfg.port ];
  };
}