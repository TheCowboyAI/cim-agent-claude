# CIM Claude Adapter - NATS Infrastructure Configuration  
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-nats;
  
  # Generate NATS JetStream configuration
  jetStreamConfig = {
    jetstream = {
      domain = cfg.jetstream.domain;
      store_dir = cfg.jetstream.storeDir;
      max_memory_store = cfg.jetstream.maxMemoryStore;
      max_file_store = cfg.jetstream.maxFileStore;
    };
    
    # Cluster configuration
    cluster = mkIf (cfg.cluster.enable) {
      name = cfg.cluster.name;
      listen = "${cfg.cluster.bindAddress}:${toString cfg.cluster.port}";
      routes = cfg.cluster.routes;
      cluster_advertise = cfg.cluster.advertiseAddress;
    };
    
    # Monitoring
    monitor_port = cfg.monitoring.port;
    server_name = cfg.serverName;
  };

  # Stream definitions using CIM subject algebra
  streamDefinitions = [
    {
      name = "CIM_CLAUDE_CONV_CMD";
      subjects = [ "cim.claude.conv.cmd.>" ];
      description = "Claude conversation commands (start, send, end)";
      retention = "workqueue";
      storage = "file";
      replicas = cfg.replication.replicas;
      max_msgs = 1000000;
      max_bytes = "10GB";
      max_age = "30d";
      max_msg_size = "64MB";
      discard = "old";
      duplicate_window = "2m";
    }
    
    {
      name = "CIM_CLAUDE_CONV_EVT";
      subjects = [ "cim.claude.conv.evt.>" ];
      description = "Claude conversation events (permanent audit trail)";
      retention = "limits";
      storage = "file";
      replicas = cfg.replication.replicas;
      max_msgs = 10000000;
      max_bytes = "100GB";
      max_age = "2y";
      max_msg_size = "16MB";
      discard = "old";
      duplicate_window = "1h";
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_CMD";
      subjects = [ "cim.claude.attach.cmd.>" ];
      description = "Attachment upload and processing commands";
      retention = "workqueue";
      storage = "file";
      replicas = cfg.replication.replicas;
      max_msgs = 100000;
      max_bytes = "5GB";
      max_age = "7d";
      max_msg_size = "256MB";
      discard = "new";
      duplicate_window = "5m";
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_EVT";
      subjects = [ "cim.claude.attach.evt.>" ];
      description = "Attachment processing events";
      retention = "limits";
      storage = "file";
      replicas = cfg.replication.replicas;
      max_msgs = 1000000;
      max_bytes = "20GB";
      max_age = "1y";
      max_msg_size = "4MB";
      discard = "old";
      duplicate_window = "30m";
    }
    
    {
      name = "CIM_CLAUDE_CONV_QRY";
      subjects = [ "cim.claude.conv.qry.>" "cim.claude.attach.qry.>" ];
      description = "Query requests for conversation and attachment data";
      retention = "workqueue";
      storage = "memory";
      replicas = cfg.replication.replicas;
      max_msgs = 500000;
      max_bytes = "2GB";
      max_age = "24h";
      max_msg_size = "1MB";
      discard = "old";
      duplicate_window = "1m";
    }
    
    {
      name = "CIM_SYS_HEALTH_EVT";
      subjects = [ "cim.sys.health.evt.>" "cim.sys.metrics.evt.>" ];
      description = "System health and metrics events";
      retention = "limits";
      storage = "file";
      replicas = cfg.replication.replicas;
      max_msgs = 10000000;
      max_bytes = "10GB";
      max_age = "90d";
      max_msg_size = "1MB";
      discard = "old";
      duplicate_window = "10s";
    }
  ];

  # Object store definitions
  objectStoreDefinitions = [
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_IMG";
      description = "Image attachments and screenshots";
      max_bytes = "500GB";
      storage = "file";
      replicas = cfg.replication.replicas;
      ttl = "2y";
      compression = false;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_DOC";
      description = "Document attachments (PDF, text, office docs)";
      max_bytes = "200GB";
      storage = "file";
      replicas = cfg.replication.replicas;
      ttl = "3y";
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_CODE";
      description = "Code files and text attachments";
      max_bytes = "50GB";
      storage = "file";
      replicas = 2;
      ttl = "1y";
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_AUDIO";
      description = "Audio file attachments";
      max_bytes = "100GB";
      storage = "file";
      replicas = 2;
      ttl = "1y";
      compression = false;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_VIDEO";
      description = "Video file attachments";
      max_bytes = "1TB";
      storage = "file";
      replicas = 2;
      ttl = "6m";
      compression = false;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_OBJ_BIN";
      description = "Binary and archive file attachments";
      max_bytes = "100GB";
      storage = "file";
      replicas = 2;
      ttl = "90d";
      compression = true;
    }
  ];

  # KV store definitions
  kvStoreDefinitions = [
    {
      name = "CIM_CLAUDE_CONV_KV";
      description = "Conversation metadata, state, and quick lookups";
      max_bytes = "20GB";
      history = 10;
      ttl = "2y";
      storage = "file";
      replicas = cfg.replication.replicas;
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_ATTACH_KV";
      description = "Attachment metadata, references, and indexing data";
      max_bytes = "5GB";
      history = 5;
      ttl = "2y";
      storage = "file";
      replicas = cfg.replication.replicas;
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_SESSION_KV";
      description = "User session data and preferences";
      max_bytes = "10GB";
      history = 3;
      ttl = "30d";
      storage = "memory";
      replicas = cfg.replication.replicas;
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_CONFIG_KV";
      description = "Configuration settings and feature flags";
      max_bytes = "1GB";
      history = 20;
      storage = "file";
      replicas = cfg.replication.replicas;
      compression = true;
    }
    
    {
      name = "CIM_CLAUDE_METRICS_KV";
      description = "Aggregated usage metrics and analytics data";
      max_bytes = "50GB";
      history = 30;
      ttl = "3y";
      storage = "file";
      replicas = 2;
      compression = true;
    }
  ];

  # Consumer definitions
  consumerDefinitions = [
    {
      stream = "CIM_CLAUDE_CONV_CMD";
      name = "claude_conversation_commands";
      description = "Process conversation commands";
      filter_subject = "cim.claude.conv.cmd.>";
      deliver_policy = "all";
      ack_policy = "explicit";
      ack_wait = "30s";
      max_deliver = 5;
      replay_policy = "instant";
    }
    
    {
      stream = "CIM_CLAUDE_ATTACH_CMD";
      name = "claude_attachment_commands";
      description = "Process attachment commands";
      filter_subject = "cim.claude.attach.cmd.>";
      deliver_policy = "all";
      ack_policy = "explicit";
      ack_wait = "5m";
      max_deliver = 3;
      replay_policy = "instant";
    }
    
    {
      stream = "CIM_CLAUDE_CONV_EVT";
      name = "claude_event_logger";
      description = "Log conversation events for audit";
      filter_subject = "cim.claude.conv.evt.>";
      deliver_policy = "all";
      ack_policy = "explicit";
      ack_wait = "1m";
      max_deliver = 10;
      replay_policy = "instant";
    }
    
    {
      stream = "CIM_CLAUDE_CONV_EVT";
      name = "claude_metrics_collector";
      description = "Collect metrics from conversation events";
      filter_subject = "cim.claude.conv.evt.>";
      deliver_policy = "all";
      ack_policy = "explicit";
      ack_wait = "30s";
      max_deliver = 5;
      replay_policy = "instant";
      sample_freq = "10%";
    }
    
    {
      stream = "CIM_CLAUDE_CONV_QRY";
      name = "claude_query_processor";
      description = "Process query requests";
      filter_subject = "cim.claude.*.qry.>";
      deliver_policy = "all";
      ack_policy = "explicit";
      ack_wait = "10s";
      max_deliver = 3;
      replay_policy = "instant";
    }
    
    {
      stream = "CIM_SYS_HEALTH_EVT";
      name = "system_health_monitor";
      description = "Monitor system health events";
      filter_subject = "cim.sys.health.evt.>";
      deliver_policy = "new";
      ack_policy = "explicit";
      ack_wait = "5s";
      max_deliver = 2;
      replay_policy = "instant";
    }
  ];

  # Generate nats configuration file
  natsConfigFile = pkgs.writeText "nats-server.conf" (lib.generators.toJSON {} (jetStreamConfig // {
    inherit (cfg) port;
    accounts = cfg.accounts;
    authorization = cfg.authorization;
    tls = cfg.tls;
  }));

  # Generate JetStream setup script
  setupJetStreamScript = pkgs.writeShellScript "setup-jetstream.sh" ''
    set -euo pipefail
    
    NATS_CLI="${pkgs.natscli}/bin/nats"
    NATS_URL="nats://${cfg.bindAddress}:${toString cfg.port}"
    
    log() {
      echo "[$(date)] $*" >&2
    }
    
    wait_for_nats() {
      log "Waiting for NATS server to be ready..."
      for i in {1..30}; do
        if $NATS_CLI --server="$NATS_URL" account info >/dev/null 2>&1; then
          log "NATS server is ready"
          return 0
        fi
        log "NATS not ready yet, waiting... ($i/30)"
        sleep 2
      done
      log "ERROR: NATS server did not become ready in time"
      exit 1
    }
    
    create_streams() {
      log "Creating JetStream streams..."
      ${concatMapStringsSep "\n" (stream: ''
        log "Creating stream ${stream.name}..."
        $NATS_CLI --server="$NATS_URL" stream add ${stream.name} \
          --subjects="${concatStringsSep "," stream.subjects}" \
          --description="${stream.description}" \
          --retention=${stream.retention} \
          --storage=${stream.storage} \
          --replicas=${toString stream.replicas} \
          --max-msgs=${toString stream.max_msgs} \
          --max-bytes="${stream.max_bytes}" \
          --max-age="${stream.max_age}" \
          --max-msg-size="${stream.max_msg_size}" \
          --discard=${stream.discard} \
          --dupe-window="${stream.duplicate_window}" \
          --allow-rollup=false \
          || log "Stream ${stream.name} already exists or failed to create"
      '') streamDefinitions}
    }
    
    create_object_stores() {
      log "Creating Object Stores..."
      ${concatMapStringsSep "\n" (os: ''
        log "Creating object store ${os.name}..."
        $NATS_CLI --server="$NATS_URL" object add ${os.name} \
          --description="${os.description}" \
          --storage=${os.storage} \
          --replicas=${toString os.replicas} \
          --max-bytes="${os.max_bytes}" \
          ${optionalString (os.ttl != null) "--ttl=\"${os.ttl}\""} \
          ${optionalString os.compression "--compression=true"} \
          || log "Object store ${os.name} already exists or failed to create"
      '') objectStoreDefinitions}
    }
    
    create_kv_stores() {
      log "Creating KV Stores..."
      ${concatMapStringsSep "\n" (kv: ''
        log "Creating KV store ${kv.name}..."
        $NATS_CLI --server="$NATS_URL" kv add ${kv.name} \
          --description="${kv.description}" \
          --storage=${kv.storage} \
          --replicas=${toString kv.replicas} \
          --max-bytes="${kv.max_bytes}" \
          --history=${toString kv.history} \
          ${optionalString (kv.ttl != null) "--ttl=\"${kv.ttl}\""} \
          ${optionalString kv.compression "--compression=true"} \
          || log "KV store ${kv.name} already exists or failed to create"
      '') kvStoreDefinitions}
    }
    
    create_consumers() {
      log "Creating Consumers..."
      ${concatMapStringsSep "\n" (consumer: ''
        log "Creating consumer ${consumer.name} on stream ${consumer.stream}..."
        $NATS_CLI --server="$NATS_URL" consumer add ${consumer.stream} ${consumer.name} \
          --description="${consumer.description}" \
          --filter="${consumer.filter_subject}" \
          --deliver=${consumer.deliver_policy} \
          --ack=${consumer.ack_policy} \
          --wait="${consumer.ack_wait}" \
          --max-deliver=${toString consumer.max_deliver} \
          --replay=${consumer.replay_policy} \
          ${optionalString (consumer.sample_freq or null != null) "--sample=\"${consumer.sample_freq}\""} \
          || log "Consumer ${consumer.name} already exists or failed to create"
      '') consumerDefinitions}
    }
    
    main() {
      wait_for_nats
      create_streams
      create_object_stores
      create_kv_stores  
      create_consumers
      log "JetStream infrastructure setup completed successfully"
    }
    
    main "$@"
  '';

in {
  ###### Interface
  options.services.cim-claude-nats = {
    enable = mkEnableOption "CIM Claude NATS infrastructure with JetStream";

    package = mkOption {
      type = types.package;
      default = pkgs.nats-server;
      defaultText = literalExpression "pkgs.nats-server";
      description = "The NATS server package to use.";
    };

    serverName = mkOption {
      type = types.str;
      default = "cim-claude-nats";
      description = "Name for this NATS server instance.";
    };

    port = mkOption {
      type = types.port;
      default = 4222;
      description = "Port for NATS client connections.";
    };

    bindAddress = mkOption {
      type = types.str;
      default = "0.0.0.0";
      description = "Address to bind the NATS server to.";
    };

    # JetStream configuration
    jetstream = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable JetStream.";
      };

      domain = mkOption {
        type = types.str;
        default = "cim-claude";
        description = "JetStream domain name.";
      };

      storeDir = mkOption {
        type = types.str;
        default = "/var/lib/nats/jetstream";
        description = "Directory for JetStream storage.";
      };

      maxMemoryStore = mkOption {
        type = types.str;
        default = "1GB";
        description = "Maximum memory storage for JetStream.";
      };

      maxFileStore = mkOption {
        type = types.str;
        default = "100GB";
        description = "Maximum file storage for JetStream.";
      };
    };

    # Cluster configuration
    cluster = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable NATS clustering.";
      };

      name = mkOption {
        type = types.str;
        default = "cim-claude-cluster";
        description = "Cluster name.";
      };

      port = mkOption {
        type = types.port;
        default = 6222;
        description = "Port for cluster communication.";
      };

      bindAddress = mkOption {
        type = types.str;
        default = "0.0.0.0";
        description = "Address to bind cluster port to.";
      };

      advertiseAddress = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Address to advertise to other cluster members.";
      };

      routes = mkOption {
        type = types.listOf types.str;
        default = [];
        description = "List of other cluster members to connect to.";
        example = [ "nats://node1:6222" "nats://node2:6222" ];
      };
    };

    # Replication settings
    replication = {
      replicas = mkOption {
        type = types.ints.between 1 5;
        default = 3;
        description = "Number of replicas for streams and stores.";
      };
    };

    # Monitoring
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable HTTP monitoring endpoint.";
      };

      port = mkOption {
        type = types.port;
        default = 8222;
        description = "Port for HTTP monitoring endpoint.";
      };
    };

    # Security
    accounts = mkOption {
      type = types.attrs;
      default = {};
      description = "NATS account configuration.";
    };

    authorization = mkOption {
      type = types.attrs;
      default = {};
      description = "NATS authorization configuration.";
    };

    tls = mkOption {
      type = types.attrs;
      default = {};
      description = "NATS TLS configuration.";
    };

    # Environment-specific adjustments
    environment = mkOption {
      type = types.enum [ "development" "staging" "production" ];
      default = "production";
      description = "Environment type for resource scaling.";
    };

    # Auto-setup JetStream resources
    autoSetup = mkOption {
      type = types.bool;
      default = true;
      description = "Automatically create JetStream streams, object stores, and KV stores.";
    };

    openFirewall = mkOption {
      type = types.bool;
      default = false;
      description = "Open firewall ports for NATS.";
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Create nats user and group
    users.users.nats = {
      description = "NATS server user";
      group = "nats";
      home = "/var/lib/nats";
      createHome = true;
      homeMode = "755";
      isSystemUser = true;
    };

    users.groups.nats = {};

    # Create JetStream storage directory
    systemd.tmpfiles.rules = [
      "d ${cfg.jetstream.storeDir} 0755 nats nats -"
      "d /var/lib/nats 0755 nats nats -"
      "d /var/log/nats 0755 nats nats -"
    ];

    # NATS server service
    systemd.services.nats-server = {
      description = "NATS Server with JetStream for CIM Claude";
      documentation = [ "https://docs.nats.io/" ];
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "exec";
        User = "nats";
        Group = "nats";
        ExecStart = "${cfg.package}/bin/nats-server -c ${natsConfigFile}";
        ExecReload = "${pkgs.coreutils}/bin/kill -HUP $MAINPID";
        
        # Security settings
        NoNewPrivileges = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [ "AF_INET" "AF_INET6" "AF_UNIX" ];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        RemoveIPC = true;
        PrivateTmp = true;
        
        # Directories
        StateDirectory = "nats";
        StateDirectoryMode = "0755";
        LogsDirectory = "nats";
        LogsDirectoryMode = "0755";
        ReadWritePaths = [ cfg.jetstream.storeDir ];
        
        # Restart configuration  
        Restart = "always";
        RestartSec = "5s";
        StartLimitInterval = "5min";
        StartLimitBurst = 3;
        
        # Resource limits
        LimitNOFILE = "65536";
        TasksMax = "8192";
      };

      unitConfig = {
        StartLimitIntervalSec = "5min";
        StartLimitBurst = 3;
      };
    };

    # JetStream setup service
    systemd.services.nats-jetstream-setup = mkIf cfg.autoSetup {
      description = "Setup NATS JetStream infrastructure for CIM Claude";
      after = [ "nats-server.service" ];
      wants = [ "nats-server.service" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "oneshot";
        User = "nats";
        Group = "nats";
        ExecStart = setupJetStreamScript;
        RemainAfterExit = true;
        
        # Restart on failure
        Restart = "on-failure";
        RestartSec = "10s";
      };

      unitConfig = {
        # Only run after NATS is healthy
        After = "nats-server.service";
        Requisite = "nats-server.service";
      };
    };

    # Firewall configuration
    networking.firewall = mkIf cfg.openFirewall {
      allowedTCPPorts = [ cfg.port ] 
        ++ optional cfg.monitoring.enable cfg.monitoring.port
        ++ optional cfg.cluster.enable cfg.cluster.port;
    };

    # Package dependencies
    environment.systemPackages = [ cfg.package pkgs.natscli ];

    # Assertions
    assertions = [
      {
        assertion = cfg.replication.replicas <= 5;
        message = "NATS JetStream supports maximum 5 replicas";
      }
      {
        assertion = cfg.cluster.enable -> (length cfg.cluster.routes > 0);
        message = "Cluster routes must be specified when clustering is enabled";
      }
      {
        assertion = cfg.jetstream.enable;
        message = "JetStream must be enabled for CIM Claude adapter";
      }
    ];

    # Environment-specific warnings
    warnings = optional (cfg.environment == "development" && cfg.replication.replicas > 1) 
      "Development environment with multiple replicas may consume excessive resources";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <hello@cowboy-ai.com>" ];
    doc = ./nats-infrastructure.md;
  };
}