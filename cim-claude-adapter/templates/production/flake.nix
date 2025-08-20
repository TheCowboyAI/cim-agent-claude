# CIM Claude Adapter - Production Deployment Template
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{
  description = "CIM Claude Adapter Production Deployment";

  inputs = {
    cim-claude-adapter.url = "github:thecowboyai/cim-claude-adapter";
    nixpkgs.follows = "cim-claude-adapter/nixpkgs";
    
    # Security and secrets
    agenix.url = "github:ryantm/agenix";
    sops-nix.url = "github:Mic92/sops-nix";
  };

  outputs = { self, cim-claude-adapter, nixpkgs, agenix, sops-nix }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      # Production NixOS configuration
      nixosConfigurations = {
        # Primary production server
        cim-claude-prod-01 = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            # Hardware configuration (customize for your hardware)
            ./hardware-configuration.nix
            
            # CIM Claude Adapter modules
            cim-claude-adapter.nixosModules.cim-claude-adapter
            cim-claude-adapter.nixosModules.nats-infrastructure
            cim-claude-adapter.nixosModules.security-hardening
            cim-claude-adapter.nixosModules.monitoring
            cim-claude-adapter.nixosModules.backup-restore
            cim-claude-adapter.nixosModules.network-topology
            cim-claude-adapter.nixosModules.high-availability
            
            # Secrets management
            agenix.nixosModules.default
            sops-nix.nixosModules.sops
            
            # Production configuration
            ({
              # System configuration
              system.stateVersion = "24.11";
              
              # Networking
              networking = {
                hostName = "cim-claude-prod-01";
                domain = "cim-claude.internal";
                firewall.enable = true;
              };
              
              # CIM Claude Adapter configuration
              services.cim-claude-adapter = {
                enable = true;
                
                # Claude API configuration (using secrets)
                claude = {
                  apiKey = "/run/secrets/claude-api-key";
                  model = "claude-3-sonnet-20240229";
                  maxTokens = 8192;
                  temperature = 0.7;
                };
                
                # NATS infrastructure
                nats = {
                  enable = true;
                  infrastructure = {
                    enable = true;
                    environment = "production";
                    replication.replicas = 3;
                    openFirewall = false;
                  };
                };
                
                # Monitoring
                monitoring = {
                  metricsPort = 9090;
                  healthPort = 8080;
                  enableTracing = true;
                };
                
                # Logging
                logging = {
                  level = "info";
                  format = "json";
                };
                
                # Open firewall for load balancer
                openFirewall = false; # Managed by network topology
              };
              
              # Security hardening
              services.cim-claude-security = {
                enable = true;
                
                # TLS configuration
                tls = {
                  enable = true;
                  certificatePath = "/run/secrets/tls-cert";
                  keyPath = "/run/secrets/tls-key";
                  protocols = [ "TLSv1.2" "TLSv1.3" ];
                };
                
                # Authentication
                auth = {
                  enable = true;
                  method = "nats-jwt";
                  jwtConfig = {
                    issuer = "cim-claude-production";
                    audience = "cim-claude-api";
                    keyPath = "/run/secrets/jwt-key";
                  };
                };
                
                # Compliance
                compliance = {
                  enable = true;
                  standards = [ "SOC2" "GDPR" ];
                  auditLogPath = "/var/log/cim-claude-audit";
                  retentionPeriod = "7y";
                };
              };
              
              # Monitoring stack
              services.cim-claude-monitoring = {
                enable = true;
                
                prometheus = {
                  enable = true;
                  port = 9090;
                  retention = "30d";
                  storage = {
                    size = "100GB";
                    path = "/var/lib/prometheus2";
                  };
                };
                
                grafana = {
                  enable = true;
                  port = 3000;
                  adminPassword = "/run/secrets/grafana-admin-password";
                };
                
                alertmanager = {
                  enable = true;
                  port = 9093;
                  config = {
                    route = {
                      receiver = "production-alerts";
                      group_by = [ "alertname" "severity" ];
                    };
                    receivers = [{
                      name = "production-alerts";
                      email_configs = [{
                        to = "alerts@cowboy-ai.com";
                        subject = "[PROD] CIM Claude Alert: {{ .GroupLabels.alertname }}";
                      }];
                      webhook_configs = [{
                        url = "https://hooks.slack.com/services/YOUR/SLACK/WEBHOOK";
                      }];
                    }];
                  };
                };
                
                jaeger = {
                  enable = true;
                  collectorPort = 14268;
                  queryPort = 16686;
                };
                
                loki = {
                  enable = true;
                  port = 3100;
                  retention = "90d";
                };
              };
              
              # Backup and restore
              services.cim-claude-backup = {
                enable = true;
                backupDir = "/var/lib/cim-claude-backup";
                
                schedule = {
                  enable = true;
                  frequency = "daily";
                  randomDelay = "2h";
                  persistent = true;
                };
                
                nats = {
                  url = "nats://localhost:4222";
                  streams = [
                    "CIM_CLAUDE_CONV_CMD"
                    "CIM_CLAUDE_CONV_EVT"
                    "CIM_CLAUDE_ATTACH_CMD"
                    "CIM_CLAUDE_ATTACH_EVT"
                    "CIM_SYS_HEALTH_EVT"
                  ];
                };
                
                compression = {
                  type = "zstd";
                  level = 6;
                };
                
                retention = {
                  days = 90;
                  maxBackups = 30;
                };
                
                remote = {
                  enable = true;
                  type = "s3";
                  bucket = "cim-claude-backups-prod";
                  path = "daily-backups";
                  credentialsFile = "/run/secrets/aws-credentials";
                };
                
                notifications = {
                  webhook = "https://hooks.slack.com/services/YOUR/BACKUP/WEBHOOK";
                  email = "backup-alerts@cowboy-ai.com";
                  onFailureOnly = false;
                };
              };
              
              # Network topology
              services.cim-claude-network = {
                enable = true;
                architecture = "multi-tier";
                
                zones = {
                  dmz = {
                    enable = true;
                    cidr = "10.100.0.0/24";
                    interface = "eth0";
                    gateway = "10.100.0.1";
                    services = [ "nginx" "haproxy" ];
                    allowedPorts = [ 80 443 ];
                  };
                  
                  app = {
                    enable = true;
                    cidr = "10.101.0.0/24";
                    interface = "eth1";
                    gateway = "10.101.0.1";
                    services = [ "cim-claude-adapter" ];
                    allowedPorts = [ 8080 9090 ];
                  };
                  
                  data = {
                    enable = true;
                    cidr = "10.102.0.0/24";
                    interface = "eth2";
                    gateway = "10.102.0.1";
                    services = [ "nats-server" ];
                    allowedPorts = [ 4222 8222 ];
                  };
                  
                  mgmt = {
                    enable = true;
                    cidr = "10.103.0.0/24";
                    interface = "eth3";
                    gateway = "10.103.0.1";
                    services = [ "prometheus" "grafana" ];
                    allowedPorts = [ 3000 9090 9093 ];
                  };
                };
                
                loadBalancer = {
                  enable = true;
                  type = "nginx";
                  backends = [
                    {
                      name = "cim-claude-01";
                      address = "10.101.0.10";
                      port = 8080;
                      weight = 100;
                    }
                    {
                      name = "cim-claude-02";
                      address = "10.101.0.11";
                      port = 8080;
                      weight = 100;
                    }
                    {
                      name = "cim-claude-03";
                      address = "10.101.0.12";
                      port = 8080;
                      weight = 100;
                    }
                  ];
                  ssl = {
                    enable = true;
                    certificatePath = "/run/secrets/tls-cert";
                    keyPath = "/run/secrets/tls-key";
                  };
                };
                
                firewall = {
                  enable = true;
                  defaultPolicy = "drop";
                };
              };
              
              # High availability
              services.cim-claude-ha = {
                enable = true;
                
                thisNode = {
                  name = "cim-claude-prod-01";
                  ip = "10.101.0.10";
                  priority = 150;
                  datacenter = "primary";
                };
                
                cluster = {
                  nodes = [
                    {
                      name = "cim-claude-prod-01";
                      ip = "10.101.0.10";
                      priority = 150;
                      datacenter = "primary";
                    }
                    {
                      name = "cim-claude-prod-02";
                      ip = "10.101.0.11";
                      priority = 140;
                      datacenter = "primary";
                    }
                    {
                      name = "cim-claude-prod-03";
                      ip = "10.101.0.12";
                      priority = 130;
                      datacenter = "secondary";
                    }
                  ];
                  
                  interface = "eth1";
                  virtualIP = "10.101.0.100";
                  virtualIPMask = 24;
                  vrid = 51;
                  password = "/run/secrets/vrrp-password";
                };
                
                healthCheck = {
                  interval = 3;
                  timeout = 3;
                  maxLoad = 8.0;
                  maxMemory = 85.0;
                };
                
                dataReplication = {
                  enable = true;
                  mode = "active-passive";
                  syncInterval = "30s";
                };
                
                splitBrainProtection = {
                  enable = true;
                  quorum = {
                    enable = true;
                    size = 2;
                  };
                };
                
                failover = {
                  enable = true;
                  timeout = "30s";
                  maxFailovers = 3;
                };
                
                notifications = {
                  webhook = "https://hooks.slack.com/services/YOUR/HA/WEBHOOK";
                  email = "ha-alerts@cowboy-ai.com";
                };
              };
              
              # Secrets configuration
              age.secrets = {
                claude-api-key = {
                  file = ./secrets/claude-api-key.age;
                  owner = "cim-claude-adapter";
                  group = "cim-claude-adapter";
                };
                
                tls-cert = {
                  file = ./secrets/tls-cert.age;
                  owner = "cim-claude-adapter";
                  group = "cim-claude-adapter";
                };
                
                tls-key = {
                  file = ./secrets/tls-key.age;
                  owner = "cim-claude-adapter";
                  group = "cim-claude-adapter";
                };
                
                jwt-key = {
                  file = ./secrets/jwt-key.age;
                  owner = "cim-claude-adapter";
                  group = "cim-claude-adapter";
                };
                
                grafana-admin-password = {
                  file = ./secrets/grafana-admin-password.age;
                  owner = "grafana";
                  group = "grafana";
                };
                
                aws-credentials = {
                  file = ./secrets/aws-credentials.age;
                  owner = "cim-claude-backup";
                  group = "cim-claude-backup";
                };
                
                vrrp-password = {
                  file = ./secrets/vrrp-password.age;
                  owner = "root";
                  group = "root";
                  mode = "0400";
                };
              };
              
              # System hardening
              security = {
                sudo.wheelNeedsPassword = true;
                auditd.enable = true;
              };
              
              # Automatic security updates
              system.autoUpgrade = {
                enable = true;
                dates = "04:00";
                allowReboot = false;
                channel = "https://nixos.org/channels/nixos-24.11";
              };
              
              # Performance tuning
              boot.kernel.sysctl = {
                # Network performance
                "net.core.rmem_max" = 134217728;
                "net.core.wmem_max" = 134217728;
                "net.ipv4.tcp_rmem" = "4096 12582912 134217728";
                "net.ipv4.tcp_wmem" = "4096 12582912 134217728";
                "net.core.netdev_max_backlog" = 5000;
                
                # File system performance
                "vm.dirty_ratio" = 15;
                "vm.dirty_background_ratio" = 5;
                
                # Security
                "kernel.kptr_restrict" = 2;
                "kernel.dmesg_restrict" = 1;
                "net.ipv4.conf.all.log_martians" = 1;
              };
            })
          ];
        };
      };
      
      # Helper scripts
      packages.${system} = {
        deploy-script = pkgs.writeShellScriptBin "deploy-production" ''
          set -euo pipefail
          
          echo "🚀 Deploying CIM Claude Adapter to Production"
          echo "================================================"
          
          # Build configuration
          echo "Building NixOS configuration..."
          nixos-rebuild build --flake .#cim-claude-prod-01
          
          # Deploy to server (customize for your deployment method)
          echo "Deploying to production server..."
          # Example: Use nixos-rebuild switch --target-host for remote deployment
          # nixos-rebuild switch --flake .#cim-claude-prod-01 --target-host root@prod-server
          
          echo "✅ Production deployment completed"
        '';
        
        backup-script = pkgs.writeShellScriptBin "backup-production" ''
          set -euo pipefail
          
          echo "📦 Starting production backup..."
          ssh root@cim-claude-prod-01 "systemctl start cim-claude-backup"
          echo "✅ Backup initiated"
        '';
        
        health-check = pkgs.writeShellScriptBin "health-check-production" ''
          set -euo pipefail
          
          echo "🏥 Production Health Check"
          echo "=========================="
          
          # Check service status
          echo "Service Status:"
          ssh root@cim-claude-prod-01 "systemctl status cim-claude-adapter nats-server"
          
          # Check endpoints
          echo "Endpoint Health:"
          curl -f https://cim-claude.prod.cowboy-ai.com/health || echo "❌ Health check failed"
          curl -f https://cim-claude.prod.cowboy-ai.com/metrics >/dev/null && echo "✅ Metrics endpoint OK" || echo "❌ Metrics endpoint failed"
          
          # Check cluster status
          echo "Cluster Status:"
          ssh root@cim-claude-prod-01 "ip addr show | grep 10.101.0.100" && echo "✅ VIP active" || echo "❌ VIP not active"
        '';
      };
    };
}