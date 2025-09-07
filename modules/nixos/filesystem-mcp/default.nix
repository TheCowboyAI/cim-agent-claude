{
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption types;
  
  cfg = config.services.filesystem-mcp;
  
  # Create a simple HTTP filesystem MCP server
  filesystem-mcp-server = pkgs.writeShellScriptBin "filesystem-mcp-server" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    PORT="''${1:-8001}"
    ROOT_PATH="''${2:-/home/steele}"
    
    echo "Starting Filesystem MCP server on http://localhost:$PORT"
    echo "Root path: $ROOT_PATH"
    
    # Simple HTTP MCP server using Python
    exec ${pkgs.python312}/bin/python3 -c "
import http.server
import socketserver
import json
import os
import urllib.parse
from datetime import datetime

class FilesystemMCPHandler(http.server.BaseHTTPRequestHandler):
    def __init__(self, *args, root_path='$ROOT_PATH', **kwargs):
        self.root_path = root_path
        super().__init__(*args, **kwargs)
    
    def do_GET(self):
        if self.path == '/mcp':
            self.handle_mcp_request()
        else:
            self.send_error(404)
    
    def do_POST(self):
        if self.path == '/mcp':
            self.handle_mcp_request()
        else:
            self.send_error(404)
    
    def handle_mcp_request(self):
        if self.command == 'GET':
            # Return MCP server info
            response = {
                'jsonrpc': '2.0',
                'result': {
                    'protocolVersion': '2024-11-05',
                    'capabilities': {
                        'tools': {'listChanged': True},
                        'resources': {'subscribe': True, 'listChanged': True}
                    },
                    'serverInfo': {
                        'name': 'filesystem-mcp-server',
                        'version': '1.0.0'
                    }
                },
                'id': 1
            }
        else:
            # Handle POST requests
            content_length = int(self.headers.get('Content-Length', 0))
            body = self.rfile.read(content_length).decode('utf-8') if content_length > 0 else '{}'
            
            try:
                request = json.loads(body) if body.strip() else {}
                method = request.get('method', 'tools/list')
                
                if method == 'initialize':
                    response = {
                        'jsonrpc': '2.0',
                        'id': request.get('id', 1),
                        'result': {
                            'protocolVersion': '2024-11-05',
                            'capabilities': {
                                'tools': {},
                                'resources': {'subscribe': True, 'listChanged': True}
                            },
                            'serverInfo': {
                                'name': 'filesystem-mcp-server',
                                'version': '1.0.0'
                            }
                        }
                    }
                elif method == 'tools/list':
                    response = {
                        'jsonrpc': '2.0',
                        'id': request.get('id', 1),
                        'result': {
                            'tools': [
                                {
                                    'name': 'read_file',
                                    'description': 'Read contents of a file',
                                    'inputSchema': {
                                        'type': 'object',
                                        'properties': {
                                            'path': {'type': 'string', 'description': 'File path to read'}
                                        },
                                        'required': ['path']
                                    }
                                },
                                {
                                    'name': 'list_directory',
                                    'description': 'List contents of a directory',
                                    'inputSchema': {
                                        'type': 'object',
                                        'properties': {
                                            'path': {'type': 'string', 'description': 'Directory path to list'}
                                        },
                                        'required': ['path']
                                    }
                                }
                            ]
                        }
                    }
                else:
                    response = {
                        'jsonrpc': '2.0',
                        'id': request.get('id', 1),
                        'error': {'code': -32601, 'message': f'Method not found: {method}'}
                    }
            except Exception as e:
                response = {
                    'jsonrpc': '2.0',
                    'id': 1,
                    'error': {'code': -32700, 'message': f'Parse error: {str(e)}'}
                }
        
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(json.dumps(response).encode())

def make_handler(root_path):
    def handler(*args, **kwargs):
        FilesystemMCPHandler(*args, root_path=root_path, **kwargs)
    return handler

PORT = int('$PORT')
ROOT_PATH = '$ROOT_PATH'
Handler = make_handler(ROOT_PATH)

with socketserver.TCPServer(('0.0.0.0', PORT), Handler) as httpd:
    print(f'Filesystem MCP Server running on http://0.0.0.0:{PORT}')
    httpd.serve_forever()
"
  '';
in {
  options.services.filesystem-mcp = {
    enable = mkEnableOption "filesystem-mcp";

    port = mkOption {
      type = types.port;
      default = 8001;
      description = "Port to run the filesystem MCP server on";
    };

    rootPath = mkOption {
      type = types.path;
      default = "/home/steele";
      description = "Root path for filesystem access";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ filesystem-mcp-server ];

    networking.firewall.allowedTCPPorts = [ cfg.port ];

    systemd.services.filesystem-mcp = {
      description = "Filesystem MCP Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${filesystem-mcp-server}/bin/filesystem-mcp-server ${toString cfg.port} ${cfg.rootPath}";
        Restart = "always";
        RestartSec = 5;
        User = "filesystem-mcp";
        Group = "filesystem-mcp";
        
        # Security settings
        ProtectSystem = "strict";
        ProtectHome = false; # Need home access for filesystem operations
        PrivateTmp = true;
        NoNewPrivileges = true;
        PrivateNetwork = false; # HTTP server needs network access
        
        # File system access
        ReadOnlyPaths = [ cfg.rootPath ];
        
        # Resource limits
        MemoryMax = "512M";
        TasksMax = 50;
      };

      environment = {
        PORT = toString cfg.port;
        ROOT_PATH = cfg.rootPath;
      };
    };

    # Create user and group
    users.users.filesystem-mcp = {
      isSystemUser = true;
      group = "filesystem-mcp";
    };

    users.groups.filesystem-mcp = {};
  };
}