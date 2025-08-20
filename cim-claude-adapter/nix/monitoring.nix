# CIM Claude Adapter - Monitoring and Observability Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-monitoring;
  
  # Prometheus configuration
  prometheusConfig = {
    global = {
      scrape_interval = "15s";
      evaluation_interval = "15s";
    };
    
    rule_files = [
      "/etc/prometheus/rules/*.yml"
    ];
    
    scrape_configs = [
      {
        job_name = "cim-claude-adapter";
        static_configs = [{
          targets = [ "localhost:${toString config.services.cim-claude-adapter.monitoring.metricsPort}" ];
        }];
        scrape_interval = "10s";
        metrics_path = "/metrics";
      }
      {
        job_name = "nats-server";
        static_configs = [{
          targets = [ "localhost:8222" ];
        }];
        scrape_interval = "15s";
        metrics_path = "/varz";
      }
      {
        job_name = "node-exporter";
        static_configs = [{
          targets = [ "localhost:9100" ];
        }];
      }
    ] ++ cfg.extraScrapeConfigs;
    
    alerting = {
      alertmanagers = [{
        static_configs = [{
          targets = [ "localhost:9093" ];
        }];
      }];
    };
  };
  
  # Alert rules
  alertRules = {
    groups = [{
      name = "cim-claude-adapter";
      rules = [
        {
          alert = "CIMClaudeAdapterDown";
          expr = "up{job=\"cim-claude-adapter\"} == 0";
          for = "1m";
          labels = {
            severity = "critical";
            service = "cim-claude-adapter";
          };
          annotations = {
            summary = "CIM Claude Adapter is down";
            description = "CIM Claude Adapter has been down for more than 1 minute.";
          };
        }
        {
          alert = "CIMClaudeAdapterHighErrorRate";
          expr = "rate(cim_claude_adapter_errors_total[5m]) > 0.1";
          for = "2m";
          labels = {
            severity = "warning";
            service = "cim-claude-adapter";
          };
          annotations = {
            summary = "High error rate in CIM Claude Adapter";
            description = "Error rate is {{ $value }} errors per second.";
          };
        }
        {
          alert = "CIMClaudeAdapterHighLatency";
          expr = "histogram_quantile(0.95, rate(cim_claude_adapter_request_duration_seconds_bucket[5m])) > 5";
          for = "5m";
          labels = {
            severity = "warning";
            service = "cim-claude-adapter";
          };
          annotations = {
            summary = "High latency in CIM Claude Adapter";
            description = "95th percentile latency is {{ $value }}s.";
          };
        }
        {
          alert = "CIMClaudeAdapterHighMemoryUsage";
          expr = "process_resident_memory_bytes{job=\"cim-claude-adapter\"} / 1024 / 1024 > 1000";
          for = "5m";
          labels = {
            severity = "warning";
            service = "cim-claude-adapter";
          };
          annotations = {
            summary = "High memory usage in CIM Claude Adapter";
            description = "Memory usage is {{ $value }}MB.";
          };
        }
        {
          alert = "NATSServerDown";
          expr = "up{job=\"nats-server\"} == 0";
          for = "1m";
          labels = {
            severity = "critical";
            service = "nats-server";
          };
          annotations = {
            summary = "NATS Server is down";
            description = "NATS Server has been down for more than 1 minute.";
          };
        }
        {
          alert = "NATSHighConnectionCount";
          expr = "nats_server_connections > 1000";
          for = "5m";
          labels = {
            severity = "warning";
            service = "nats-server";
          };
          annotations = {
            summary = "NATS Server has high connection count";
            description = "NATS Server has {{ $value }} connections.";
          };
        }
        {
          alert = "JetStreamStorageHigh";
          expr = "nats_jetstream_storage_bytes_used / nats_jetstream_storage_bytes_limit > 0.8";
          for = "10m";
          labels = {
            severity = "warning";
            service = "nats-jetstream";
          };
          annotations = {
            summary = "JetStream storage usage is high";
            description = "JetStream storage usage is {{ $value | humanizePercentage }}.";
          };
        }
      ] ++ cfg.extraAlertRules;
    }];
  };
  
  # Grafana dashboard configuration
  grafanaDashboard = {
    dashboard = {
      id = null;
      title = "CIM Claude Adapter";
      tags = [ "cim" "claude" "adapter" ];
      timezone = "UTC";
      panels = [
        {
          title = "Request Rate";
          type = "stat";
          targets = [{
            expr = "rate(cim_claude_adapter_requests_total[5m])";
            legendFormat = "Requests/sec";
          }];
          gridPos = { h = 8; w = 12; x = 0; y = 0; };
        }
        {
          title = "Error Rate";
          type = "stat";
          targets = [{
            expr = "rate(cim_claude_adapter_errors_total[5m])";
            legendFormat = "Errors/sec";
          }];
          gridPos = { h = 8; w = 12; x = 12; y = 0; };
        }
        {
          title = "Response Time";
          type = "graph";
          targets = [{
            expr = "histogram_quantile(0.50, rate(cim_claude_adapter_request_duration_seconds_bucket[5m]))";
            legendFormat = "50th percentile";
          } {
            expr = "histogram_quantile(0.95, rate(cim_claude_adapter_request_duration_seconds_bucket[5m]))";
            legendFormat = "95th percentile";
          } {
            expr = "histogram_quantile(0.99, rate(cim_claude_adapter_request_duration_seconds_bucket[5m]))";
            legendFormat = "99th percentile";
          }];
          gridPos = { h = 8; w = 24; x = 0; y = 8; };
        }
        {
          title = "Memory Usage";
          type = "graph";
          targets = [{
            expr = "process_resident_memory_bytes{job=\"cim-claude-adapter\"} / 1024 / 1024";
            legendFormat = "RSS Memory (MB)";
          }];
          gridPos = { h = 8; w = 12; x = 0; y = 16; };
        }
        {
          title = "NATS Connections";
          type = "graph";
          targets = [{
            expr = "nats_server_connections";
            legendFormat = "Active Connections";
          }];
          gridPos = { h = 8; w = 12; x = 12; y = 16; };
        }
      ];
      time = {
        from = "now-1h";
        to = "now";
      };
      refresh = "30s";
    };
  };

in {
  ###### Interface
  options.services.cim-claude-monitoring = {
    enable = mkEnableOption "CIM Claude Adapter monitoring and observability stack";

    # Prometheus configuration
    prometheus = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Prometheus metrics collection.";
      };

      port = mkOption {
        type = types.port;
        default = 9090;
        description = "Prometheus server port.";
      };

      retention = mkOption {
        type = types.str;
        default = "15d";
        description = "Prometheus data retention period.";
      };

      storage = {
        size = mkOption {
          type = types.str;
          default = "50GB";
          description = "Prometheus storage size.";
        };

        path = mkOption {
          type = types.str;
          default = "/var/lib/prometheus2";
          description = "Prometheus data storage path.";
        };
      };
    };

    # Grafana configuration
    grafana = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Grafana dashboards.";
      };

      port = mkOption {
        type = types.port;
        default = 3000;
        description = "Grafana server port.";
      };

      adminPassword = mkOption {
        type = types.str;
        description = "Grafana admin password.";
        example = "supersecretpassword";
      };

      datasources = mkOption {
        type = types.listOf types.attrs;
        default = [{
          name = "Prometheus";
          type = "prometheus";
          url = "http://localhost:${toString cfg.prometheus.port}";
          isDefault = true;
        }];
        description = "Grafana data sources configuration.";
      };
    };

    # Alertmanager configuration
    alertmanager = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Alertmanager for alert routing.";
      };

      port = mkOption {
        type = types.port;
        default = 9093;
        description = "Alertmanager port.";
      };

      config = mkOption {
        type = types.attrs;
        default = {
          global = {
            smtp_smarthost = "localhost:587";
          };
          route = {
            group_by = [ "alertname" "severity" ];
            group_wait = "30s";
            group_interval = "5m";
            repeat_interval = "12h";
            receiver = "default";
          };
          receivers = [{
            name = "default";
            email_configs = [{
              to = "alerts@cowboy-ai.com";
              subject = "CIM Claude Adapter Alert: {{ .GroupLabels.alertname }}";
              body = "{{ range .Alerts }}{{ .Annotations.description }}{{ end }}";
            }];
          }];
        };
        description = "Alertmanager configuration.";
      };
    };

    # Jaeger tracing
    jaeger = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Jaeger distributed tracing.";
      };

      collectorPort = mkOption {
        type = types.port;
        default = 14268;
        description = "Jaeger collector port.";
      };

      queryPort = mkOption {
        type = types.port;
        default = 16686;
        description = "Jaeger query UI port.";
      };
    };

    # Loki log aggregation
    loki = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Loki log aggregation.";
      };

      port = mkOption {
        type = types.port;
        default = 3100;
        description = "Loki server port.";
      };

      retention = mkOption {
        type = types.str;
        default = "30d";
        description = "Log retention period.";
      };
    };

    # Node exporter for system metrics
    nodeExporter = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable Node Exporter for system metrics.";
      };

      port = mkOption {
        type = types.port;
        default = 9100;
        description = "Node Exporter port.";
      };
    };

    # Custom configurations
    extraScrapeConfigs = mkOption {
      type = types.listOf types.attrs;
      default = [];
      description = "Additional Prometheus scrape configurations.";
    };

    extraAlertRules = mkOption {
      type = types.listOf types.attrs;
      default = [];
      description = "Additional Prometheus alert rules.";
    };

    # Dashboard provisioning
    dashboards = mkOption {
      type = types.listOf types.path;
      default = [];
      description = "Additional Grafana dashboard files to provision.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open firewall ports for monitoring services.";
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Prometheus service
    services.prometheus = mkIf cfg.prometheus.enable {
      enable = true;
      port = cfg.prometheus.port;
      dataDir = cfg.prometheus.storage.path;
      retentionTime = cfg.prometheus.retention;
      
      extraFlags = [
        "--storage.tsdb.retention.size=${cfg.prometheus.storage.size}"
        "--web.enable-lifecycle"
        "--web.enable-admin-api"
      ];
      
      globalConfig = prometheusConfig.global;
      scrapeConfigs = prometheusConfig.scrape_configs;
      alerting = prometheusConfig.alerting;
      
      rules = [
        (pkgs.writeText "cim-claude-adapter-rules.yml" (builtins.toJSON alertRules))
      ];
    };

    # Alertmanager service
    services.prometheus.alertmanager = mkIf cfg.alertmanager.enable {
      enable = true;
      port = cfg.alertmanager.port;
      configuration = cfg.alertmanager.config;
      
      extraFlags = [
        "--web.external-url=http://localhost:${toString cfg.alertmanager.port}"
      ];
    };

    # Grafana service
    services.grafana = mkIf cfg.grafana.enable {
      enable = true;
      settings = {
        server = {
          http_port = cfg.grafana.port;
          domain = "localhost";
        };
        
        security = {
          admin_password = cfg.grafana.adminPassword;
        };
        
        analytics.reporting_enabled = false;
        
        "auth.anonymous" = {
          enabled = false;
        };
        
        users = {
          allow_sign_up = false;
          auto_assign_org_role = "Viewer";
        };
        
        dashboards = {
          default_home_dashboard_path = "/var/lib/grafana/dashboards/cim-claude-adapter.json";
        };
      };
      
      provision = {
        enable = true;
        datasources.settings = {
          apiVersion = 1;
          datasources = cfg.grafana.datasources;
        };
        
        dashboards.settings = {
          apiVersion = 1;
          providers = [{
            name = "CIM Claude Adapter";
            orgId = 1;
            folder = "CIM";
            type = "file";
            disableDeletion = false;
            updateIntervalSeconds = 10;
            options.path = "/var/lib/grafana/dashboards";
          }];
        };
      };
    };

    # Node Exporter service
    services.prometheus.exporters.node = mkIf cfg.nodeExporter.enable {
      enable = true;
      port = cfg.nodeExporter.port;
      enabledCollectors = [
        "systemd"
        "processes"
        "network_route"
        "mountstats"
        "filesystem"
        "diskstats"
        "netstat"
      ];
    };

    # Loki service
    services.loki = mkIf cfg.loki.enable {
      enable = true;
      configuration = {
        server = {
          http_listen_port = cfg.loki.port;
        };
        
        auth_enabled = false;
        
        ingester = {
          lifecycler = {
            address = "127.0.0.1";
            ring = {
              kvstore = {
                store = "inmemory";
              };
              replication_factor = 1;
            };
          };
          chunk_idle_period = "1h";
          max_chunk_age = "1h";
          chunk_target_size = 1048576;
          chunk_retain_period = "30s";
        };
        
        schema_config = {
          configs = [{
            from = "2020-10-24";
            store = "boltdb-shipper";
            object_store = "filesystem";
            schema = "v11";
            index = {
              prefix = "index_";
              period = "24h";
            };
          }];
        };
        
        storage_config = {
          boltdb_shipper = {
            active_index_directory = "/var/lib/loki/boltdb-shipper-active";
            cache_location = "/var/lib/loki/boltdb-shipper-cache";
            cache_ttl = "24h";
            shared_store = "filesystem";
          };
          
          filesystem = {
            directory = "/var/lib/loki/chunks";
          };
        };
        
        compactor = {
          working_directory = "/var/lib/loki";
          shared_store = "filesystem";
          compaction_interval = "10m";
          retention_enabled = true;
          retention_delete_delay = "2h";
          retention_delete_worker_count = 150;
        };
        
        limits_config = {
          retention_period = cfg.loki.retention;
          enforce_metric_name = false;
          reject_old_samples = true;
          reject_old_samples_max_age = "168h";
        };
      };
    };

    # Promtail for log shipping to Loki
    services.promtail = mkIf cfg.loki.enable {
      enable = true;
      configuration = {
        server = {
          http_listen_port = 9080;
          grpc_listen_port = 0;
        };
        
        positions = {
          filename = "/var/lib/promtail/positions.yaml";
        };
        
        clients = [{
          url = "http://localhost:${toString cfg.loki.port}/loki/api/v1/push";
        }];
        
        scrape_configs = [{
          job_name = "cim-claude-adapter";
          static_configs = [{
            targets = [ "localhost" ];
            labels = {
              job = "cim-claude-adapter";
              __path__ = "/var/log/cim-claude-adapter/*.log";
            };
          }];
          
          pipeline_stages = [{
            json = {
              expressions = {
                timestamp = "timestamp";
                level = "level";
                message = "message";
              };
            };
          } {
            timestamp = {
              source = "timestamp";
              format = "RFC3339";
            };
          } {
            labels = {
              level = "";
            };
          }];
        } {
          job_name = "systemd";
          journal = {
            max_age = "12h";
            labels = {
              job = "systemd-journal";
              host = "localhost";
            };
          };
          
          relabel_configs = [{
            source_labels = [ "__journal__systemd_unit" ];
            target_label = "unit";
          } {
            source_labels = [ "__journal_priority" ];
            target_label = "priority";
          }];
        }];
      };
    };

    # Jaeger service
    services.jaeger = mkIf cfg.jaeger.enable {
      enable = true;
      collector = {
        enable = true;
        port = cfg.jaeger.collectorPort;
      };
      query = {
        enable = true;
        port = cfg.jaeger.queryPort;
      };
    };

    # Create Grafana dashboard files
    systemd.tmpfiles.rules = mkIf cfg.grafana.enable [
      "d /var/lib/grafana/dashboards 0755 grafana grafana -"
    ];

    # Install default dashboard
    environment.etc."grafana/dashboards/cim-claude-adapter.json" = mkIf cfg.grafana.enable {
      text = builtins.toJSON grafanaDashboard;
      mode = "0644";
    };

    # Symlink custom dashboards
    systemd.services.grafana-dashboard-provisioner = mkIf (cfg.grafana.enable && length cfg.dashboards > 0) {
      description = "Provision custom Grafana dashboards";
      before = [ "grafana.service" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };
      
      script = concatMapStringsSep "\n" (dashboard:
        "ln -sf ${dashboard} /var/lib/grafana/dashboards/"
      ) cfg.dashboards;
    };

    # Firewall configuration
    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = 
        optional cfg.prometheus.enable cfg.prometheus.port ++
        optional cfg.grafana.enable cfg.grafana.port ++
        optional cfg.alertmanager.enable cfg.alertmanager.port ++
        optional cfg.jaeger.enable cfg.jaeger.queryPort ++
        optional cfg.loki.enable cfg.loki.port ++
        optional cfg.nodeExporter.enable cfg.nodeExporter.port;
    };

    # Package dependencies
    environment.systemPackages = with pkgs; [
      prometheus
    ] ++ optional cfg.grafana.enable grafana
      ++ optional cfg.alertmanager.enable alertmanager
      ++ optional cfg.jaeger.enable jaeger
      ++ optional cfg.loki.enable loki
      ++ optional cfg.loki.enable promtail;

    # Assertions
    assertions = [
      {
        assertion = cfg.grafana.enable -> cfg.grafana.adminPassword != "";
        message = "Grafana admin password must be set";
      }
      {
        assertion = cfg.prometheus.enable;
        message = "Prometheus is required for monitoring stack";
      }
    ];

    # Warnings
    warnings = 
      optional (!cfg.alertmanager.enable) "Alertmanager is disabled - alerts will not be routed" ++
      optional (!cfg.grafana.enable) "Grafana is disabled - no dashboards will be available";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <monitoring@cowboy-ai.com>" ];
    doc = ./monitoring.md;
  };
}