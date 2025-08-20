# CIM Claude Adapter - Network Topology and Security Zones Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-network;
  
  # Network zone definitions
  zones = {
    dmz = {
      name = "DMZ";
      description = "Demilitarized Zone - External facing services";
      cidr = "10.100.0.0/24";
      services = [ "load-balancer" "reverse-proxy" ];
      firewall_rules = "restrictive";
    };
    
    app = {
      name = "Application";
      description = "Application tier - CIM Claude Adapter instances";
      cidr = "10.101.0.0/24";
      services = [ "cim-claude-adapter" ];
      firewall_rules = "application";
    };
    
    data = {
      name = "Data";
      description = "Data tier - NATS and storage";
      cidr = "10.102.0.0/24";
      services = [ "nats-server" "postgresql" "redis" ];
      firewall_rules = "data";
    };
    
    mgmt = {
      name = "Management";
      description = "Management and monitoring";
      cidr = "10.103.0.0/24";
      services = [ "prometheus" "grafana" "logging" ];
      firewall_rules = "management";
    };
  };

in {
  ###### Interface
  options.services.cim-claude-network = {
    enable = mkEnableOption "CIM Claude Adapter network topology and security zones";

    # Network architecture
    architecture = mkOption {
      type = types.enum [ "single-node" "multi-tier" "kubernetes" "cloud-native" ];
      default = "multi-tier";
      description = "Network architecture pattern to deploy.";
    };

    # Zone configuration
    zones = mkOption {
      type = types.attrsOf (types.submodule {
        options = {
          enable = mkOption {
            type = types.bool;
            default = true;
            description = "Enable this network zone.";
          };

          cidr = mkOption {
            type = types.str;
            description = "CIDR block for this zone.";
            example = "10.100.0.0/24";
          };

          interface = mkOption {
            type = types.str;
            description = "Network interface for this zone.";
            example = "eth0";
          };

          vlan = mkOption {
            type = types.nullOr types.int;
            default = null;
            description = "VLAN ID for zone separation.";
          };

          gateway = mkOption {
            type = types.str;
            description = "Gateway IP for this zone.";
            example = "10.100.0.1";
          };

          dns = mkOption {
            type = types.listOf types.str;
            default = [ "8.8.8.8" "1.1.1.1" ];
            description = "DNS servers for this zone.";
          };

          services = mkOption {
            type = types.listOf types.str;
            default = [];
            description = "Services allowed in this zone.";
          };

          allowedPorts = mkOption {
            type = types.listOf types.int;
            default = [];
            description = "Allowed TCP ports for this zone.";
          };

          allowedUdpPorts = mkOption {
            type = types.listOf types.int;
            default = [];
            description = "Allowed UDP ports for this zone.";
          };
        };
      });
      default = {
        dmz = {
          cidr = "10.100.0.0/24";
          interface = "eth0";
          gateway = "10.100.0.1";
          services = [ "nginx" "haproxy" ];
          allowedPorts = [ 80 443 8080 ];
        };
        app = {
          cidr = "10.101.0.0/24";
          interface = "eth1";
          gateway = "10.101.0.1";
          services = [ "cim-claude-adapter" ];
          allowedPorts = [ 8080 9090 ];
        };
        data = {
          cidr = "10.102.0.0/24";
          interface = "eth2";
          gateway = "10.102.0.1";
          services = [ "nats-server" "postgresql" ];
          allowedPorts = [ 4222 5432 6379 ];
        };
        mgmt = {
          cidr = "10.103.0.0/24";
          interface = "eth3";
          gateway = "10.103.0.1";
          services = [ "prometheus" "grafana" ];
          allowedPorts = [ 3000 9090 9093 ];
        };
      };
      description = "Network zone configuration.";
    };

    # Load balancer configuration
    loadBalancer = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable load balancer in DMZ.";
      };

      type = mkOption {
        type = types.enum [ "nginx" "haproxy" "traefik" ];
        default = "nginx";
        description = "Load balancer implementation.";
      };

      backends = mkOption {
        type = types.listOf (types.submodule {
          options = {
            name = mkOption {
              type = types.str;
              description = "Backend name.";
            };
            
            address = mkOption {
              type = types.str;
              description = "Backend IP address.";
            };
            
            port = mkOption {
              type = types.port;
              description = "Backend port.";
            };
            
            weight = mkOption {
              type = types.int;
              default = 100;
              description = "Backend weight for load balancing.";
            };

            healthCheck = mkOption {
              type = types.str;
              default = "/health";
              description = "Health check endpoint.";
            };
          };
        });
        default = [];
        description = "Backend servers for load balancing.";
      };

      ssl = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable SSL termination.";
        };

        certificatePath = mkOption {
          type = types.path;
          description = "Path to SSL certificate.";
        };

        keyPath = mkOption {
          type = types.path;
          description = "Path to SSL private key.";
        };

        protocols = mkOption {
          type = types.listOf types.str;
          default = [ "TLSv1.2" "TLSv1.3" ];
          description = "Allowed TLS protocols.";
        };
      };
    };

    # Firewall configuration per zone
    firewall = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable zone-based firewall.";
      };

      defaultPolicy = mkOption {
        type = types.enum [ "accept" "drop" "reject" ];
        default = "drop";
        description = "Default firewall policy.";
      };

      rules = mkOption {
        type = types.listOf (types.submodule {
          options = {
            name = mkOption {
              type = types.str;
              description = "Rule name.";
            };

            source = mkOption {
              type = types.str;
              description = "Source zone or CIDR.";
            };

            destination = mkOption {
              type = types.str;
              description = "Destination zone or CIDR.";
            };

            protocol = mkOption {
              type = types.enum [ "tcp" "udp" "icmp" "any" ];
              default = "tcp";
              description = "Protocol to match.";
            };

            port = mkOption {
              type = types.nullOr types.int;
              default = null;
              description = "Destination port to match.";
            };

            action = mkOption {
              type = types.enum [ "accept" "drop" "reject" "log" ];
              default = "accept";
              description = "Action to take.";
            };
          };
        });
        default = [];
        description = "Custom firewall rules.";
      };

      rateLimiting = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable rate limiting.";
        };

        rules = mkOption {
          type = types.listOf (types.submodule {
            options = {
              source = mkOption {
                type = types.str;
                description = "Source to rate limit.";
              };

              limit = mkOption {
                type = types.str;
                description = "Rate limit (e.g., '100/min').";
              };

              burst = mkOption {
                type = types.int;
                default = 20;
                description = "Burst limit.";
              };
            };
          });
          default = [
            {
              source = "0.0.0.0/0";
              limit = "1000/min";
              burst = 50;
            }
          ];
          description = "Rate limiting rules.";
        };
      };
    };

    # Service discovery and registration
    serviceDiscovery = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable service discovery.";
      };

      backend = mkOption {
        type = types.enum [ "consul" "etcd" "nats" ];
        default = "nats";
        description = "Service discovery backend.";
      };

      ttl = mkOption {
        type = types.str;
        default = "30s";
        description = "Service registration TTL.";
      };

      healthCheckInterval = mkOption {
        type = types.str;
        default = "10s";
        description = "Health check interval for service discovery.";
      };
    };

    # Network monitoring
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable network monitoring.";
      };

      interfaces = mkOption {
        type = types.listOf types.str;
        default = [ "eth0" "eth1" "eth2" "eth3" ];
        description = "Network interfaces to monitor.";
      };

      snmp = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable SNMP monitoring.";
        };

        community = mkOption {
          type = types.str;
          default = "public";
          description = "SNMP community string.";
        };
      };
    };

    # DNS configuration
    dns = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable internal DNS resolution.";
      };

      domain = mkOption {
        type = types.str;
        default = "cim-claude.internal";
        description = "Internal DNS domain.";
      };

      records = mkOption {
        type = types.listOf (types.submodule {
          options = {
            name = mkOption {
              type = types.str;
              description = "DNS record name.";
            };

            type = mkOption {
              type = types.enum [ "A" "AAAA" "CNAME" "SRV" ];
              default = "A";
              description = "DNS record type.";
            };

            value = mkOption {
              type = types.str;
              description = "DNS record value.";
            };

            ttl = mkOption {
              type = types.int;
              default = 300;
              description = "DNS record TTL.";
            };
          };
        });
        default = [];
        description = "Custom DNS records.";
      };
    };

    # VPN configuration for remote access
    vpn = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable VPN server for remote access.";
      };

      type = mkOption {
        type = types.enum [ "wireguard" "openvpn" ];
        default = "wireguard";
        description = "VPN implementation.";
      };

      subnet = mkOption {
        type = types.str;
        default = "10.200.0.0/24";
        description = "VPN client subnet.";
      };

      port = mkOption {
        type = types.port;
        default = 51820;
        description = "VPN server port.";
      };
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Network interface configuration
    networking = {
      # Enable IP forwarding for multi-tier architecture
      firewall.enable = mkDefault false; # We manage firewall rules manually
      
      # Configure interfaces for each zone
      interfaces = mkMerge (mapAttrsToList (zoneName: zoneConfig:
        mkIf zoneConfig.enable {
          ${zoneConfig.interface} = {
            ipv4.addresses = [{
              address = head (splitString "/" zoneConfig.cidr);
              prefixLength = toInt (last (splitString "/" zoneConfig.cidr));
            }];
          };
        }
      ) cfg.zones);

      # Default gateway configuration
      defaultGateway = {
        address = cfg.zones.dmz.gateway;
        interface = cfg.zones.dmz.interface;
      };

      # DNS configuration
      nameservers = cfg.zones.dmz.dns;
    };

    # NGINX load balancer configuration
    services.nginx = mkIf (cfg.loadBalancer.enable && cfg.loadBalancer.type == "nginx") {
      enable = true;
      
      upstreams = {
        cim_claude_backend = {
          servers = mkMerge (map (backend: {
            "${backend.address}:${toString backend.port}" = {
              weight = backend.weight;
            };
          }) cfg.loadBalancer.backends);
        };
      };

      virtualHosts."cim-claude.local" = {
        listen = [
          { addr = "0.0.0.0"; port = 80; }
        ] ++ optional cfg.loadBalancer.ssl.enable {
          addr = "0.0.0.0"; port = 443; ssl = true;
        };

        serverName = "cim-claude.local";

        locations = {
          "/" = {
            proxyPass = "http://cim_claude_backend";
            proxySetHeaders = {
              "Host" = "$host";
              "X-Real-IP" = "$remote_addr";
              "X-Forwarded-For" = "$proxy_add_x_forwarded_for";
              "X-Forwarded-Proto" = "$scheme";
            };
          };

          "/health" = {
            proxyPass = "http://cim_claude_backend/health";
            extraConfig = ''
              access_log off;
            '';
          };

          "/metrics" = {
            proxyPass = "http://cim_claude_backend/metrics";
            extraConfig = ''
              allow 10.103.0.0/24;  # Management zone
              deny all;
            '';
          };
        };

        extraConfig = ''
          # Rate limiting
          limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
          limit_req zone=api burst=20 nodelay;
          
          # Security headers
          add_header X-Frame-Options SAMEORIGIN;
          add_header X-Content-Type-Options nosniff;
          add_header X-XSS-Protection "1; mode=block";
          add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
        '';

        sslCertificate = mkIf cfg.loadBalancer.ssl.enable cfg.loadBalancer.ssl.certificatePath;
        sslCertificateKey = mkIf cfg.loadBalancer.ssl.enable cfg.loadBalancer.ssl.keyPath;
        
        sslProtocols = mkIf cfg.loadBalancer.ssl.enable (concatStringsSep " " cfg.loadBalancer.ssl.protocols);
        
        sslCiphers = mkIf cfg.loadBalancer.ssl.enable "ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305";
      };

      appendHttpConfig = ''
        # Logging format with security info
        log_format security '$remote_addr - $remote_user [$time_local] '
                          '"$request" $status $bytes_sent '
                          '"$http_referer" "$http_user_agent" '
                          '$request_time $upstream_response_time';
        
        access_log /var/log/nginx/access.log security;
        
        # Rate limiting zones
        limit_req_zone $binary_remote_addr zone=login:10m rate=1r/s;
        limit_req_zone $binary_remote_addr zone=global:10m rate=100r/s;
        
        # Connection limiting
        limit_conn_zone $binary_remote_addr zone=conn_limit_per_ip:10m;
        limit_conn conn_limit_per_ip 20;
      '';
    };

    # Advanced firewall with zone-based rules
    networking.firewall = mkIf cfg.firewall.enable {
      enable = true;
      
      extraCommands = ''
        # Flush existing rules
        iptables -t nat -F
        iptables -t mangle -F
        iptables -F
        iptables -X
        
        # Default policies
        iptables -P INPUT ${cfg.firewall.defaultPolicy}
        iptables -P FORWARD ${cfg.firewall.defaultPolicy}
        iptables -P OUTPUT ACCEPT
        
        # Allow loopback
        iptables -A INPUT -i lo -j ACCEPT
        iptables -A OUTPUT -o lo -j ACCEPT
        
        # Allow established connections
        iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
        
        # Zone-based rules
        ${concatMapStringsSep "\n" (zoneName: zoneConfig:
          let
            zoneInterface = zoneConfig.interface;
            zoneCIDR = zoneConfig.cidr;
          in
          mkIf zoneConfig.enable ''
            # Rules for ${zoneName} zone (${zoneInterface})
            ${concatMapStringsSep "\n" (port: 
              "iptables -A INPUT -i ${zoneInterface} -p tcp --dport ${toString port} -j ACCEPT"
            ) zoneConfig.allowedPorts}
            
            ${concatMapStringsSep "\n" (port: 
              "iptables -A INPUT -i ${zoneInterface} -p udp --dport ${toString port} -j ACCEPT"
            ) zoneConfig.allowedUdpPorts}
            
            # Inter-zone communication rules
            ${optionalString (zoneName == "dmz") ''
              # DMZ to App zone
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.app.cidr} -p tcp --dport 8080 -j ACCEPT
              # DMZ to Data zone (health checks only)
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.data.cidr} -p tcp --dport 8222 -j ACCEPT
            ''}
            
            ${optionalString (zoneName == "app") ''
              # App to Data zone
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.data.cidr} -p tcp --dport 4222 -j ACCEPT
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.data.cidr} -p tcp --dport 5432 -j ACCEPT
              # App to Management zone (metrics)
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.mgmt.cidr} -p tcp --dport 9090 -j ACCEPT
            ''}
            
            ${optionalString (zoneName == "mgmt") ''
              # Management to all zones (monitoring)
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.dmz.cidr} -j ACCEPT
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.app.cidr} -j ACCEPT
              iptables -A FORWARD -s ${zoneCIDR} -d ${cfg.zones.data.cidr} -j ACCEPT
            ''}
          ''
        ) cfg.zones}
        
        # Rate limiting rules
        ${optionalString cfg.firewall.rateLimiting.enable (
          concatMapStringsSep "\n" (rule:
            "iptables -A INPUT -s ${rule.source} -m limit --limit ${rule.limit} --limit-burst ${toString rule.burst} -j ACCEPT"
          ) cfg.firewall.rateLimiting.rules
        )}
        
        # Custom firewall rules
        ${concatMapStringsSep "\n" (rule:
          "iptables -A INPUT -s ${rule.source} -d ${rule.destination} -p ${rule.protocol} ${optionalString (rule.port != null) "--dport ${toString rule.port}"} -j ${rule.action}"
        ) cfg.firewall.rules}
        
        # Log dropped packets
        iptables -A INPUT -j LOG --log-prefix "CIM-FIREWALL-DROP: " --log-level 4
        iptables -A FORWARD -j LOG --log-prefix "CIM-FORWARD-DROP: " --log-level 4
      '';
    };

    # Service discovery with NATS
    systemd.services.cim-claude-service-discovery = mkIf (cfg.serviceDiscovery.enable && cfg.serviceDiscovery.backend == "nats") {
      description = "CIM Claude Service Discovery";
      after = [ "network.target" "nats-server.service" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "exec";
        ExecStart = pkgs.writeShellScript "service-discovery" ''
          set -euo pipefail
          
          # Register services with NATS
          while true; do
            # Register CIM Claude Adapter instances
            ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 pub "cim.discovery.register" '{
              "service": "cim-claude-adapter",
              "address": "10.101.0.10",
              "port": 8080,
              "zone": "app",
              "health_check": "/health",
              "timestamp": "'$(date -Iseconds)'"
            }'
            
            # Register NATS server
            ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 pub "cim.discovery.register" '{
              "service": "nats-server",
              "address": "10.102.0.10",
              "port": 4222,
              "zone": "data",
              "health_check": "/varz",
              "timestamp": "'$(date -Iseconds)'"
            }'
            
            sleep ${cfg.serviceDiscovery.ttl}
          done
        '';
        
        Restart = "always";
        RestartSec = "10s";
      };
    };

    # DNS server for internal resolution
    services.dnsmasq = mkIf cfg.dns.enable {
      enable = true;
      settings = {
        domain = cfg.dns.domain;
        expand-hosts = true;
        
        # Zone-specific host resolution
        address = mkMerge (mapAttrsToList (zoneName: zoneConfig:
          mkIf zoneConfig.enable {
            "/${zoneName}.${cfg.dns.domain}/${head (splitString "/" zoneConfig.cidr)}";
          }
        ) cfg.zones);
        
        # Custom DNS records
        host-record = map (record:
          "${record.name}.${cfg.dns.domain},${record.value}"
        ) cfg.dns.records;
        
        # Upstream DNS servers
        server = cfg.zones.dmz.dns;
        
        # Logging
        log-queries = true;
        log-dhcp = true;
      };
    };

    # Network monitoring with Prometheus exporters
    services.prometheus.exporters = mkIf cfg.monitoring.enable {
      node = {
        enable = true;
        enabledCollectors = [
          "systemd"
          "network"
          "netclass"
          "netdev"
          "netstat"
        ];
      };
      
      snmp = mkIf cfg.monitoring.snmp.enable {
        enable = true;
        configurationPath = pkgs.writeText "snmp.yml" ''
          modules:
            default:
              walk:
                - 1.3.6.1.2.1.2.2.1.10  # ifInOctets
                - 1.3.6.1.2.1.2.2.1.16  # ifOutOctets
              lookups:
                - source_indexes: [ifIndex]
                  lookup: 1.3.6.1.2.1.2.2.1.2
        '';
      };
    };

    # WireGuard VPN server
    networking.wireguard.interfaces.wg0 = mkIf (cfg.vpn.enable && cfg.vpn.type == "wireguard") {
      ips = [ "10.200.0.1/24" ];
      listenPort = cfg.vpn.port;
      
      # Generate keys with: wg genkey | tee privatekey | wg pubkey > publickey
      privateKeyFile = "/etc/wireguard/private.key";
      
      peers = [
        # Example peer - replace with actual client configurations
        {
          publicKey = "EXAMPLE_CLIENT_PUBLIC_KEY";
          allowedIPs = [ "10.200.0.2/32" ];
        }
      ];
      
      postSetup = ''
        # Enable NAT for VPN clients
        iptables -t nat -A POSTROUTING -s ${cfg.vpn.subnet} -o eth0 -j MASQUERADE
        iptables -A FORWARD -i wg0 -j ACCEPT
        iptables -A FORWARD -o wg0 -j ACCEPT
      '';
      
      postShutdown = ''
        # Cleanup NAT rules
        iptables -t nat -D POSTROUTING -s ${cfg.vpn.subnet} -o eth0 -j MASQUERADE
        iptables -D FORWARD -i wg0 -j ACCEPT
        iptables -D FORWARD -o wg0 -j ACCEPT
      '';
    };

    # Package dependencies
    environment.systemPackages = with pkgs; [
      iptables
      tcpdump
      netcat
      nmap
      wireshark
      dig
      whois
    ] ++ optional (cfg.loadBalancer.type == "nginx") nginx
      ++ optional cfg.dns.enable dnsmasq
      ++ optional (cfg.vpn.enable && cfg.vpn.type == "wireguard") wireguard-tools;

    # Assertions
    assertions = [
      {
        assertion = cfg.loadBalancer.enable -> length cfg.loadBalancer.backends > 0;
        message = "Load balancer backends must be configured";
      }
      {
        assertion = cfg.loadBalancer.ssl.enable -> (cfg.loadBalancer.ssl.certificatePath != null && cfg.loadBalancer.ssl.keyPath != null);
        message = "SSL certificate and key paths must be specified when SSL is enabled";
      }
      {
        assertion = all (zone: zone.cidr != "") (attrValues cfg.zones);
        message = "All zones must have CIDR blocks configured";
      }
    ];

    # Warnings
    warnings = 
      optional (!cfg.firewall.enable) "Zone-based firewall is disabled - network security may be compromised" ++
      optional (!cfg.loadBalancer.ssl.enable) "SSL is disabled on load balancer - consider enabling for production";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <network@cowboy-ai.com>" ];
    doc = ./network-topology.md;
  };
}