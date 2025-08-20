# CIM Claude Adapter - High Availability and Clustering Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-ha;
  
  # Cluster node configuration
  thisNode = cfg.thisNode;
  otherNodes = filter (node: node.name != thisNode.name) cfg.cluster.nodes;
  
  # Keepalived configuration
  keepalivedConfig = pkgs.writeText "keepalived.conf" ''
    global_defs {
      router_id ${thisNode.name}
      enable_script_security
    }
    
    vrrp_script chk_cim_claude {
      script "${cfg.healthCheck.script}"
      interval ${toString cfg.healthCheck.interval}
      timeout ${toString cfg.healthCheck.timeout}
      rise 2
      fall 3
    }
    
    vrrp_instance CIM_CLAUDE_VIP {
      state ${if thisNode.priority > (foldl max 0 (map (n: n.priority) otherNodes)) then "MASTER" else "BACKUP"}
      interface ${cfg.cluster.interface}
      virtual_router_id ${toString cfg.cluster.vrid}
      priority ${toString thisNode.priority}
      advert_int 1
      authentication {
        auth_type PASS
        auth_pass ${cfg.cluster.password}
      }
      virtual_ipaddress {
        ${cfg.cluster.virtualIP}/${toString cfg.cluster.virtualIPMask}
      }
      track_script {
        chk_cim_claude
      }
      notify_master "${cfg.callbacks.master}"
      notify_backup "${cfg.callbacks.backup}"
      notify_fault "${cfg.callbacks.fault}"
    }
  '';
  
  # Cluster health check script
  healthCheckScript = pkgs.writeShellScript "cim-claude-health-check" ''
    set -euo pipefail
    
    # Check CIM Claude Adapter health
    if ! curl -f -s http://localhost:8080/health >/dev/null 2>&1; then
      logger -t keepalived "CIM Claude Adapter health check failed"
      exit 1
    fi
    
    # Check NATS connectivity
    if ! ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 account info >/dev/null 2>&1; then
      logger -t keepalived "NATS health check failed"
      exit 1
    fi
    
    # Check system resources
    load_avg=$(cat /proc/loadavg | cut -d' ' -f1)
    if (( $(echo "$load_avg > ${toString cfg.healthCheck.maxLoad}" | bc -l) )); then
      logger -t keepalived "System load too high: $load_avg"
      exit 1
    fi
    
    # Check memory usage
    mem_used=$(free | awk '/^Mem:/{print $3/$2*100}')
    if (( $(echo "$mem_used > ${toString cfg.healthCheck.maxMemory}" | bc -l) )); then
      logger -t keepalived "Memory usage too high: $mem_used%"
      exit 1
    fi
    
    logger -t keepalived "Health check passed"
    exit 0
  '';
  
  # Master callback script
  masterCallbackScript = pkgs.writeShellScript "cim-claude-master-callback" ''
    set -euo pipefail
    
    logger -t keepalived "Transitioning to MASTER state"
    
    # Enable services that should only run on master
    ${optionalString cfg.services.backups.masterOnly ''
      systemctl start cim-claude-backup.timer
    ''}
    
    ${optionalString cfg.services.metrics.masterOnly ''
      systemctl start cim-claude-metrics-aggregator
    ''}
    
    # Update service discovery
    ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 pub "cim.cluster.master" '{
      "node": "${thisNode.name}",
      "ip": "${thisNode.ip}",
      "timestamp": "'$(date -Iseconds)'"
    }' || true
    
    # Send notification
    ${optionalString (cfg.notifications.webhook != null) ''
      curl -X POST "${cfg.notifications.webhook}" \
        -H "Content-Type: application/json" \
        -d "{\"event\": \"master_elected\", \"node\": \"${thisNode.name}\", \"ip\": \"${thisNode.ip}\", \"timestamp\": \"$(date -Iseconds)\"}" || true
    ''}
  '';
  
  # Backup callback script
  backupCallbackScript = pkgs.writeShellScript "cim-claude-backup-callback" ''
    set -euo pipefail
    
    logger -t keepalived "Transitioning to BACKUP state"
    
    # Disable services that should only run on master
    ${optionalString cfg.services.backups.masterOnly ''
      systemctl stop cim-claude-backup.timer || true
    ''}
    
    ${optionalString cfg.services.metrics.masterOnly ''
      systemctl stop cim-claude-metrics-aggregator || true
    ''}
    
    # Update service discovery
    ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 pub "cim.cluster.backup" '{
      "node": "${thisNode.name}",
      "ip": "${thisNode.ip}",
      "timestamp": "'$(date -Iseconds)'"
    }' || true
  '';
  
  # Fault callback script
  faultCallbackScript = pkgs.writeShellScript "cim-claude-fault-callback" ''
    set -euo pipefail
    
    logger -t keepalived "Entering FAULT state"
    
    # Send critical alert
    ${optionalString (cfg.notifications.webhook != null) ''
      curl -X POST "${cfg.notifications.webhook}" \
        -H "Content-Type: application/json" \
        -d "{\"event\": \"node_fault\", \"node\": \"${thisNode.name}\", \"ip\": \"${thisNode.ip}\", \"timestamp\": \"$(date -Iseconds)\", \"severity\": \"critical\"}" || true
    ''}
    
    ${optionalString (cfg.notifications.email != null) ''
      echo "CRITICAL: CIM Claude cluster node ${thisNode.name} has entered FAULT state.
      
Timestamp: $(date -Iseconds)
Node IP: ${thisNode.ip}
      
This indicates a serious issue with the service health checks.
Please investigate immediately." | mail -s "CIM Claude Cluster FAULT - ${thisNode.name}" "${cfg.notifications.email}" || true
    ''}
  '';

in {
  ###### Interface
  options.services.cim-claude-ha = {
    enable = mkEnableOption "CIM Claude Adapter high availability and clustering";

    # This node configuration
    thisNode = {
      name = mkOption {
        type = types.str;
        description = "Name of this cluster node.";
        example = "cim-claude-01";
      };

      ip = mkOption {
        type = types.str;
        description = "IP address of this node.";
        example = "10.101.0.10";
      };

      priority = mkOption {
        type = types.ints.between 1 255;
        description = "VRRP priority for this node (higher = preferred master).";
        example = 150;
      };

      datacenter = mkOption {
        type = types.str;
        default = "primary";
        description = "Datacenter or availability zone for this node.";
      };

      rack = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Rack identifier for this node.";
      };
    };

    # Cluster configuration
    cluster = {
      nodes = mkOption {
        type = types.listOf (types.submodule {
          options = {
            name = mkOption {
              type = types.str;
              description = "Node name.";
            };

            ip = mkOption {
              type = types.str;
              description = "Node IP address.";
            };

            priority = mkOption {
              type = types.ints.between 1 255;
              description = "VRRP priority.";
            };

            datacenter = mkOption {
              type = types.str;
              default = "primary";
              description = "Datacenter or availability zone.";
            };

            rack = mkOption {
              type = types.nullOr types.str;
              default = null;
              description = "Rack identifier.";
            };
          };
        });
        description = "Cluster node configuration.";
        example = [
          {
            name = "cim-claude-01";
            ip = "10.101.0.10";
            priority = 150;
            datacenter = "primary";
          }
          {
            name = "cim-claude-02";
            ip = "10.101.0.11";
            priority = 140;
            datacenter = "primary";
          }
          {
            name = "cim-claude-03";
            ip = "10.101.0.12";
            priority = 130;
            datacenter = "secondary";
          }
        ];
      };

      interface = mkOption {
        type = types.str;
        default = "eth1";
        description = "Network interface for VRRP.";
      };

      virtualIP = mkOption {
        type = types.str;
        description = "Virtual IP address for the cluster.";
        example = "10.101.0.100";
      };

      virtualIPMask = mkOption {
        type = types.ints.between 8 32;
        default = 24;
        description = "Subnet mask for virtual IP.";
      };

      vrid = mkOption {
        type = types.ints.between 1 255;
        default = 51;
        description = "VRRP ID (must be unique per network).";
      };

      password = mkOption {
        type = types.str;
        description = "VRRP authentication password.";
        example = "secure-cluster-password";
      };
    };

    # Health checking configuration
    healthCheck = {
      script = mkOption {
        type = types.str;
        default = toString healthCheckScript;
        description = "Health check script path.";
      };

      interval = mkOption {
        type = types.int;
        default = 3;
        description = "Health check interval in seconds.";
      };

      timeout = mkOption {
        type = types.int;
        default = 3;
        description = "Health check timeout in seconds.";
      };

      maxLoad = mkOption {
        type = types.float;
        default = 5.0;
        description = "Maximum system load average before failover.";
      };

      maxMemory = mkOption {
        type = types.float;
        default = 90.0;
        description = "Maximum memory usage percentage before failover.";
      };
    };

    # Service management
    services = {
      backups = {
        masterOnly = mkOption {
          type = types.bool;
          default = true;
          description = "Only run backups on master node.";
        };
      };

      metrics = {
        masterOnly = mkOption {
          type = types.bool;
          default = true;
          description = "Only run metrics aggregation on master node.";
        };
      };

      logging = {
        masterOnly = mkOption {
          type = types.bool;
          default = false;
          description = "Only run log aggregation on master node.";
        };
      };
    };

    # Load balancing configuration
    loadBalancing = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable load balancing between cluster nodes.";
      };

      algorithm = mkOption {
        type = types.enum [ "round-robin" "least-connections" "weighted" "health-check" ];
        default = "health-check";
        description = "Load balancing algorithm.";
      };

      sessionAffinity = mkOption {
        type = types.bool;
        default = false;
        description = "Enable session affinity (sticky sessions).";
      };

      healthCheckPath = mkOption {
        type = types.str;
        default = "/health";
        description = "Health check endpoint for load balancer.";
      };
    };

    # Data replication
    dataReplication = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable data replication between nodes.";
      };

      mode = mkOption {
        type = types.enum [ "active-passive" "active-active" "master-slave" ];
        default = "active-passive";
        description = "Data replication mode.";
      };

      syncInterval = mkOption {
        type = types.str;
        default = "30s";
        description = "Data synchronization interval.";
      };

      compressionEnabled = mkOption {
        type = types.bool;
        default = true;
        description = "Enable compression for replication traffic.";
      };
    };

    # Split-brain protection
    splitBrainProtection = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable split-brain protection mechanisms.";
      };

      quorum = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable quorum-based split-brain protection.";
        };

        size = mkOption {
          type = types.int;
          default = 2;
          description = "Minimum quorum size for cluster operations.";
        };
      };

      witness = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable witness node for tie-breaking.";
        };

        address = mkOption {
          type = types.str;
          default = "";
          description = "Witness node address.";
        };
      };
    };

    # Automatic failover and recovery
    failover = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable automatic failover.";
      };

      timeout = mkOption {
        type = types.str;
        default = "30s";
        description = "Failover timeout period.";
      };

      maxFailovers = mkOption {
        type = types.int;
        default = 3;
        description = "Maximum failovers per hour before manual intervention required.";
      };

      recovery = {
        autoReintegration = mkOption {
          type = types.bool;
          default = true;
          description = "Automatically reintegrate recovered nodes.";
        };

        probationPeriod = mkOption {
          type = types.str;
          default = "5m";
          description = "Probation period before full reintegration.";
        };
      };
    };

    # Callbacks and notifications
    callbacks = {
      master = mkOption {
        type = types.str;
        default = toString masterCallbackScript;
        description = "Script to run when becoming master.";
      };

      backup = mkOption {
        type = types.str;
        default = toString backupCallbackScript;
        description = "Script to run when becoming backup.";
      };

      fault = mkOption {
        type = types.str;
        default = toString faultCallbackScript;
        description = "Script to run when entering fault state.";
      };
    };

    # Notifications
    notifications = {
      webhook = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Webhook URL for cluster events.";
      };

      email = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Email address for critical alerts.";
      };

      slack = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable Slack notifications.";
        };

        webhookUrl = mkOption {
          type = types.str;
          default = "";
          description = "Slack webhook URL.";
        };

        channel = mkOption {
          type = types.str;
          default = "#alerts";
          description = "Slack channel for notifications.";
        };
      };
    };

    # Monitoring and metrics
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable cluster monitoring.";
      };

      port = mkOption {
        type = types.port;
        default = 9092;
        description = "Port for cluster metrics.";
      };

      scrapeInterval = mkOption {
        type = types.str;
        default = "15s";
        description = "Metrics scrape interval.";
      };
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Keepalived service for VRRP
    services.keepalived = {
      enable = true;
      vrrpInstances = {
        CIM_CLAUDE_VIP = {
          state = if thisNode.priority > (foldl max 0 (map (n: n.priority) otherNodes)) then "MASTER" else "BACKUP";
          interface = cfg.cluster.interface;
          virtualRouterId = cfg.cluster.vrid;
          priority = thisNode.priority;
          virtualIps = [{
            addr = cfg.cluster.virtualIP;
            prefixLen = cfg.cluster.virtualIPMask;
          }];
          
          extraConfig = ''
            advert_int 1
            authentication {
              auth_type PASS
              auth_pass ${cfg.cluster.password}
            }
            track_script {
              chk_cim_claude
            }
            notify_master "${cfg.callbacks.master}"
            notify_backup "${cfg.callbacks.backup}"
            notify_fault "${cfg.callbacks.fault}"
          '';
        };
      };
      
      vrrpScripts = {
        chk_cim_claude = {
          script = cfg.healthCheck.script;
          interval = cfg.healthCheck.interval;
          timeout = cfg.healthCheck.timeout;
          rise = 2;
          fall = 3;
        };
      };
    };

    # Cluster communication service
    systemd.services.cim-claude-cluster = {
      description = "CIM Claude Cluster Communication";
      after = [ "network.target" "nats-server.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        User = "cim-claude-adapter";
        Group = "cim-claude-adapter";
        ExecStart = pkgs.writeShellScript "cluster-service" ''
          set -euo pipefail
          
          # Cluster heartbeat and coordination
          while true; do
            # Publish node status
            ${pkgs.natscli}/bin/nats --server=nats://localhost:4222 pub "cim.cluster.heartbeat" '{
              "node": "${thisNode.name}",
              "ip": "${thisNode.ip}",
              "datacenter": "${thisNode.datacenter}",
              "rack": ${if thisNode.rack != null then ''"${thisNode.rack}"'' else "null"},
              "priority": ${toString thisNode.priority},
              "load": "'$(cat /proc/loadavg | cut -d' ' -f1)'",
              "memory": "'$(free | awk '/^Mem:/{print $3/$2*100}')'",
              "timestamp": "'$(date -Iseconds)'"
            }' || true
            
            # Check cluster state
            cluster_size=$(${pkgs.natscli}/bin/nats --server=nats://localhost:4222 sub "cim.cluster.heartbeat" --count=1 --timeout=5s | wc -l || echo "0")
            
            # Log cluster status
            logger -t cim-cluster "Cluster size: $cluster_size, This node: ${thisNode.name}"
            
            sleep ${toString cfg.healthCheck.interval}
          done
        '';
        
        Restart = "always";
        RestartSec = "10s";
      };
    };

    # Data replication service
    systemd.services.cim-claude-replication = mkIf cfg.dataReplication.enable {
      description = "CIM Claude Data Replication";
      after = [ "cim-claude-adapter.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        User = "cim-claude-adapter";
        Group = "cim-claude-adapter";
        ExecStart = pkgs.writeShellScript "replication-service" ''
          set -euo pipefail
          
          # Data replication logic
          while true; do
            # In active-passive mode, only master replicates
            is_master=$(ip addr show ${cfg.cluster.interface} | grep -q "${cfg.cluster.virtualIP}" && echo "true" || echo "false")
            
            if [ "$is_master" = "true" ] || [ "${cfg.dataReplication.mode}" = "active-active" ]; then
              # Replicate NATS data to other nodes
              ${concatMapStringsSep "\n" (node:
                optionalString (node.name != thisNode.name) ''
                  # Replicate to ${node.name}
                  logger -t replication "Replicating data to ${node.name} (${node.ip})"
                  
                  # Example: Stream backup and restore to peer
                  # In practice, this would use NATS clustering or custom replication
                  timeout 30 ${pkgs.natscli}/bin/nats --server=nats://${node.ip}:4222 account info >/dev/null 2>&1 || {
                    logger -t replication "Failed to connect to ${node.name}"
                    continue
                  }
                ''
              ) cfg.cluster.nodes}
            fi
            
            sleep ${cfg.dataReplication.syncInterval}
          done
        '';
        
        Restart = "always";
        RestartSec = "30s";
      };
    };

    # Split-brain detection service
    systemd.services.cim-claude-split-brain-monitor = mkIf cfg.splitBrainProtection.enable {
      description = "CIM Claude Split-Brain Monitor";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        ExecStart = pkgs.writeShellScript "split-brain-monitor" ''
          set -euo pipefail
          
          while true; do
            # Check for multiple masters
            masters=$(timeout 5 ${pkgs.nmap}/bin/nmap -p 22 ${concatMapStringsSep " " (node: node.ip) cfg.cluster.nodes} 2>/dev/null | \
              grep -c "Host is up" || echo "0")
            
            active_vips=$(${concatMapStringsSep " && " (node: 
              "timeout 2 ping -c 1 ${cfg.cluster.virtualIP} >/dev/null 2>&1"
            ) cfg.cluster.nodes} && echo "1" || echo "0")
            
            if [ "$masters" -gt 1 ]; then
              logger -t split-brain "WARNING: Potential split-brain detected - $masters active masters"
              
              # Send alert
              ${optionalString (cfg.notifications.webhook != null) ''
                curl -X POST "${cfg.notifications.webhook}" \
                  -H "Content-Type: application/json" \
                  -d "{\"event\": \"split_brain_detected\", \"masters\": $masters, \"timestamp\": \"$(date -Iseconds)\", \"severity\": \"critical\"}" || true
              ''}
            fi
            
            sleep 30
          done
        '';
        
        Restart = "always";
        RestartSec = "60s";
      };
    };

    # Cluster metrics service
    systemd.services.cim-claude-cluster-metrics = mkIf cfg.monitoring.enable {
      description = "CIM Claude Cluster Metrics";
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        ExecStart = pkgs.writeShellScript "cluster-metrics" ''
          set -euo pipefail
          
          # Simple HTTP server for cluster metrics
          ${pkgs.python3}/bin/python3 -c "
import http.server
import json
import subprocess
import time
from pathlib import Path

class ClusterMetricsHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            
            metrics = []
            
            # Cluster node status
            is_master = subprocess.run(['ip', 'addr', 'show', '${cfg.cluster.interface}'], 
                                     capture_output=True, text=True).returncode == 0 and \
                       '${cfg.cluster.virtualIP}' in subprocess.run(['ip', 'addr', 'show', '${cfg.cluster.interface}'], 
                                                                   capture_output=True, text=True).stdout
            
            metrics.append(f'cim_cluster_node_is_master {{node=\"${thisNode.name}\"}} {1 if is_master else 0}')
            metrics.append(f'cim_cluster_node_priority {{node=\"${thisNode.name}\"}} ${toString thisNode.priority}')
            
            # Cluster size (approximated by reachable nodes)
            cluster_size = 0
            ${concatMapStringsSep "\n" (node: ''
              try:
                  result = subprocess.run(['ping', '-c', '1', '-W', '1', '${node.ip}'], 
                                        capture_output=True, timeout=2)
                  if result.returncode == 0:
                      cluster_size += 1
                      metrics.append(f'cim_cluster_node_reachable {{node=\"${node.name}\",ip=\"${node.ip}\"}} 1')
                  else:
                      metrics.append(f'cim_cluster_node_reachable {{node=\"${node.name}\",ip=\"${node.ip}\"}} 0')
              except:
                  metrics.append(f'cim_cluster_node_reachable {{node=\"${node.name}\",ip=\"${node.ip}\"}} 0')
            '') cfg.cluster.nodes}
            
            metrics.append(f'cim_cluster_size {{}} {cluster_size}')
            
            # Health check status
            try:
                result = subprocess.run(['${cfg.healthCheck.script}'], capture_output=True, timeout=5)
                health_status = 1 if result.returncode == 0 else 0
            except:
                health_status = 0
            
            metrics.append(f'cim_cluster_health_check_status {{node=\"${thisNode.name}\"}} {health_status}')
            
            self.wfile.write('\\n'.join(metrics).encode() + b'\\n')
        else:
            self.send_response(404)
            self.end_headers()

if __name__ == '__main__':
    server = http.server.HTTPServer(('0.0.0.0', ${toString cfg.monitoring.port}), ClusterMetricsHandler)
    server.serve_forever()
"
        '';
        
        Restart = "always";
        RestartSec = "10s";
      };
    };

    # Enable IP forwarding for VRRP
    boot.kernel.sysctl = {
      "net.ipv4.ip_forward" = 1;
      "net.ipv4.ip_nonlocal_bind" = 1;
    };

    # Package dependencies
    environment.systemPackages = with pkgs; [
      keepalived
      iputils
      iproute2
      nmap
      bc
      natscli
    ];

    # Firewall rules for cluster communication
    networking.firewall = {
      allowedTCPPorts = [ cfg.monitoring.port ];
      allowedUDPPorts = [ 112 ]; # VRRP
      extraCommands = ''
        # Allow VRRP multicast
        iptables -A INPUT -d 224.0.0.18/32 -j ACCEPT
        iptables -A OUTPUT -d 224.0.0.18/32 -j ACCEPT
        
        # Allow cluster communication
        ${concatMapStringsSep "\n" (node:
          optionalString (node.name != thisNode.name) ''
            iptables -A INPUT -s ${node.ip} -j ACCEPT
            iptables -A OUTPUT -d ${node.ip} -j ACCEPT
          ''
        ) cfg.cluster.nodes}
      '';
    };

    # Logrotate for cluster logs
    services.logrotate.settings."/var/log/cluster.log" = {
      frequency = "daily";
      rotate = 30;
      compress = true;
      delaycompress = true;
      missingok = true;
      notifempty = true;
    };

    # Assertions
    assertions = [
      {
        assertion = length cfg.cluster.nodes >= 2;
        message = "At least 2 nodes required for clustering";
      }
      {
        assertion = any (node: node.name == thisNode.name) cfg.cluster.nodes;
        message = "This node must be included in cluster node list";
      }
      {
        assertion = cfg.splitBrainProtection.quorum.enable -> cfg.splitBrainProtection.quorum.size <= length cfg.cluster.nodes;
        message = "Quorum size cannot exceed cluster size";
      }
      {
        assertion = cfg.cluster.password != "";
        message = "VRRP password must be configured";
      }
    ];

    # Warnings
    warnings = 
      optional (length cfg.cluster.nodes < 3) "Cluster has less than 3 nodes - consider adding more for better fault tolerance" ++
      optional (!cfg.splitBrainProtection.enable) "Split-brain protection is disabled - this may lead to data corruption" ++
      optional (!cfg.dataReplication.enable) "Data replication is disabled - data may be lost during failover";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <cluster@cowboy-ai.com>" ];
    doc = ./high-availability.md;
  };
}