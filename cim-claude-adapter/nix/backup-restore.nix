# CIM Claude Adapter - Backup and Restore Module
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cim-claude-backup;
  
  # Backup script for NATS data
  backupScript = pkgs.writeShellScript "cim-claude-backup" ''
    set -euo pipefail
    
    # Configuration
    BACKUP_DIR="''${BACKUP_DIR:-${cfg.backupDir}}"
    NATS_URL="''${NATS_URL:-${cfg.nats.url}}"
    RETENTION_DAYS=${toString cfg.retention.days}
    DATE=$(date -Iseconds)
    BACKUP_NAME="cim-claude-backup-$DATE"
    
    # Logging function
    log() {
      echo "[$(date -Iseconds)] $*" >&2
      logger -t cim-claude-backup "$*"
    }
    
    # Error handling
    cleanup() {
      local exit_code=$?
      if [ $exit_code -ne 0 ]; then
        log "ERROR: Backup failed with exit code $exit_code"
        ${optionalString (cfg.notifications.webhook != null) ''
          curl -X POST "${cfg.notifications.webhook}" \
            -H "Content-Type: application/json" \
            -d "{\"status\": \"error\", \"message\": \"Backup failed\", \"timestamp\": \"$(date -Iseconds)\"}"
        ''}
      fi
    }
    trap cleanup EXIT
    
    # Pre-flight checks
    log "Starting backup: $BACKUP_NAME"
    mkdir -p "$BACKUP_DIR"
    
    # Check NATS connectivity
    if ! ${pkgs.natscli}/bin/nats --server="$NATS_URL" account info >/dev/null 2>&1; then
      log "ERROR: Cannot connect to NATS server at $NATS_URL"
      exit 1
    fi
    
    # Create backup directory
    BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"
    mkdir -p "$BACKUP_PATH"
    
    # Backup JetStream streams
    log "Backing up JetStream streams..."
    ${concatMapStringsSep "\n" (stream: ''
      log "Backing up stream: ${stream}"
      if ${pkgs.natscli}/bin/nats --server="$NATS_URL" stream info "${stream}" >/dev/null 2>&1; then
        ${pkgs.natscli}/bin/nats --server="$NATS_URL" stream backup "${stream}" "$BACKUP_PATH/${stream}.backup"
        log "Successfully backed up stream: ${stream}"
      else
        log "WARNING: Stream ${stream} not found, skipping"
      fi
    '') cfg.nats.streams}
    
    # Backup KV stores
    log "Backing up KV stores..."
    ${concatMapStringsSep "\n" (kv: ''
      log "Backing up KV store: ${kv}"
      if ${pkgs.natscli}/bin/nats --server="$NATS_URL" kv status "${kv}" >/dev/null 2>&1; then
        ${pkgs.natscli}/bin/nats --server="$NATS_URL" kv get "${kv}" --all > "$BACKUP_PATH/${kv}.kv"
        log "Successfully backed up KV store: ${kv}"
      else
        log "WARNING: KV store ${kv} not found, skipping"
      fi
    '') cfg.nats.kvStores}
    
    # Backup Object stores
    log "Backing up Object stores..."
    ${concatMapStringsSep "\n" (obj: ''
      log "Backing up Object store: ${obj}"
      if ${pkgs.natscli}/bin/nats --server="$NATS_URL" object status "${obj}" >/dev/null 2>&1; then
        mkdir -p "$BACKUP_PATH/objects/${obj}"
        ${pkgs.natscli}/bin/nats --server="$NATS_URL" object get "${obj}" --all "$BACKUP_PATH/objects/${obj}/"
        log "Successfully backed up Object store: ${obj}"
      else
        log "WARNING: Object store ${obj} not found, skipping"
      fi
    '') cfg.nats.objectStores}
    
    # Backup configuration files
    log "Backing up configuration files..."
    mkdir -p "$BACKUP_PATH/config"
    ${concatMapStringsSep "\n" (configPath: ''
      if [ -f "${configPath}" ]; then
        cp -r "${configPath}" "$BACKUP_PATH/config/"
        log "Backed up config: ${configPath}"
      fi
    '') cfg.configPaths}
    
    # Create metadata file
    cat > "$BACKUP_PATH/metadata.json" << EOF
{
  "backup_name": "$BACKUP_NAME",
  "timestamp": "$DATE",
  "nats_url": "$NATS_URL",
  "version": "1.0.0",
  "components": {
    "streams": [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.nats.streams}],
    "kv_stores": [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.nats.kvStores}],
    "object_stores": [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.nats.objectStores}],
    "configs": [${concatMapStringsSep ", " (s: ''"${s}"'') cfg.configPaths}]
  },
  "retention_days": $RETENTION_DAYS
}
EOF
    
    # Compress backup
    log "Compressing backup..."
    cd "$BACKUP_DIR"
    ${if cfg.compression.type == "gzip" then ''
      tar -czf "$BACKUP_NAME.tar.gz" "$BACKUP_NAME"
      BACKUP_FILE="$BACKUP_NAME.tar.gz"
    '' else if cfg.compression.type == "zstd" then ''
      tar -c "$BACKUP_NAME" | ${pkgs.zstd}/bin/zstd -${toString cfg.compression.level} > "$BACKUP_NAME.tar.zst"
      BACKUP_FILE="$BACKUP_NAME.tar.zst"
    '' else ''
      tar -cf "$BACKUP_NAME.tar" "$BACKUP_NAME"
      BACKUP_FILE="$BACKUP_NAME.tar"
    ''}
    
    # Calculate checksum
    ${if cfg.checksum.enable then ''
      ${cfg.checksum.algorithm}sum "$BACKUP_FILE" > "$BACKUP_FILE.${cfg.checksum.algorithm}"
      log "Checksum created: $BACKUP_FILE.${cfg.checksum.algorithm}"
    '' else ""}
    
    # Cleanup temporary directory
    rm -rf "$BACKUP_NAME"
    
    # Upload to remote storage if configured
    ${optionalString (cfg.remote.enable) ''
      log "Uploading to remote storage..."
      case "${cfg.remote.type}" in
        "s3")
          ${pkgs.awscli2}/bin/aws s3 cp "$BACKUP_FILE" "s3://${cfg.remote.bucket}/${cfg.remote.path}/$BACKUP_FILE"
          ${optionalString cfg.checksum.enable ''
            ${pkgs.awscli2}/bin/aws s3 cp "$BACKUP_FILE.${cfg.checksum.algorithm}" "s3://${cfg.remote.bucket}/${cfg.remote.path}/$BACKUP_FILE.${cfg.checksum.algorithm}"
          ''}
          log "Upload completed to S3"
          ;;
        "restic")
          ${pkgs.restic}/bin/restic backup "$BACKUP_FILE" --repo "${cfg.remote.repo}"
          log "Upload completed to Restic repository"
          ;;
        "rsync")
          ${pkgs.rsync}/bin/rsync -av "$BACKUP_FILE" "${cfg.remote.destination}/"
          ${optionalString cfg.checksum.enable ''
            ${pkgs.rsync}/bin/rsync -av "$BACKUP_FILE.${cfg.checksum.algorithm}" "${cfg.remote.destination}/"
          ''}
          log "Upload completed via rsync"
          ;;
      esac
    ''}
    
    # Cleanup old backups
    log "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    find "$BACKUP_DIR" -name "cim-claude-backup-*" -type f -mtime +$RETENTION_DAYS -delete
    
    # Calculate backup size
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
    log "Backup completed successfully: $BACKUP_FILE ($BACKUP_SIZE)"
    
    # Send success notification
    ${optionalString (cfg.notifications.webhook != null) ''
      curl -X POST "${cfg.notifications.webhook}" \
        -H "Content-Type: application/json" \
        -d "{\"status\": \"success\", \"backup_file\": \"$BACKUP_FILE\", \"size\": \"$BACKUP_SIZE\", \"timestamp\": \"$DATE\"}"
    ''}
    
    ${optionalString (cfg.notifications.email != null) ''
      echo "CIM Claude Adapter backup completed successfully.
      
Backup file: $BACKUP_FILE
Size: $BACKUP_SIZE
Timestamp: $DATE
      
Components backed up:
- JetStream streams: ${concatStringsSep ", " cfg.nats.streams}
- KV stores: ${concatStringsSep ", " cfg.nats.kvStores}  
- Object stores: ${concatStringsSep ", " cfg.nats.objectStores}
      
Retention: $RETENTION_DAYS days" | mail -s "CIM Claude Backup Success" "${cfg.notifications.email}"
    ''}
  '';
  
  # Restore script
  restoreScript = pkgs.writeShellScript "cim-claude-restore" ''
    set -euo pipefail
    
    BACKUP_FILE="''${1:-}"
    NATS_URL="''${NATS_URL:-${cfg.nats.url}}"
    FORCE="''${FORCE:-false}"
    
    # Logging function
    log() {
      echo "[$(date -Iseconds)] $*" >&2
      logger -t cim-claude-restore "$*"
    }
    
    if [ -z "$BACKUP_FILE" ]; then
      log "ERROR: Backup file not specified"
      echo "Usage: $0 <backup-file>"
      exit 1
    fi
    
    if [ ! -f "$BACKUP_FILE" ]; then
      log "ERROR: Backup file not found: $BACKUP_FILE"
      exit 1
    fi
    
    # Verify checksum if available
    ${optionalString cfg.checksum.enable ''
      if [ -f "$BACKUP_FILE.${cfg.checksum.algorithm}" ]; then
        log "Verifying backup checksum..."
        if ! ${cfg.checksum.algorithm}sum -c "$BACKUP_FILE.${cfg.checksum.algorithm}"; then
          log "ERROR: Backup checksum verification failed"
          exit 1
        fi
        log "Backup checksum verified"
      fi
    ''}
    
    # Extract backup
    log "Extracting backup: $BACKUP_FILE"
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT
    
    cd "$TEMP_DIR"
    case "$BACKUP_FILE" in
      *.tar.gz)
        tar -xzf "$BACKUP_FILE"
        ;;
      *.tar.zst)
        ${pkgs.zstd}/bin/zstd -d < "$BACKUP_FILE" | tar -x
        ;;
      *.tar)
        tar -xf "$BACKUP_FILE"
        ;;
      *)
        log "ERROR: Unsupported backup format"
        exit 1
        ;;
    esac
    
    # Find extracted directory
    BACKUP_DIR=$(find . -type d -name "cim-claude-backup-*" | head -n1)
    if [ -z "$BACKUP_DIR" ]; then
      log "ERROR: Cannot find backup directory in archive"
      exit 1
    fi
    
    cd "$BACKUP_DIR"
    
    # Read metadata
    if [ -f "metadata.json" ]; then
      log "Backup metadata:"
      ${pkgs.jq}/bin/jq . metadata.json
    fi
    
    # Check NATS connectivity
    if ! ${pkgs.natscli}/bin/nats --server="$NATS_URL" account info >/dev/null 2>&1; then
      log "ERROR: Cannot connect to NATS server at $NATS_URL"
      exit 1
    fi
    
    # Warning about destructive operation
    if [ "$FORCE" != "true" ]; then
      log "WARNING: This operation will overwrite existing data!"
      log "Set FORCE=true to proceed with restore"
      exit 1
    fi
    
    # Restore JetStream streams
    log "Restoring JetStream streams..."
    for stream_backup in *.backup; do
      if [ -f "$stream_backup" ]; then
        stream_name=$(basename "$stream_backup" .backup)
        log "Restoring stream: $stream_name"
        ${pkgs.natscli}/bin/nats --server="$NATS_URL" stream restore "$stream_name" "$stream_backup" || log "WARNING: Failed to restore stream $stream_name"
      fi
    done
    
    # Restore KV stores
    log "Restoring KV stores..."
    for kv_backup in *.kv; do
      if [ -f "$kv_backup" ]; then
        kv_name=$(basename "$kv_backup" .kv)
        log "Restoring KV store: $kv_name"
        # Note: This is a simplified restore - in practice, you'd need to parse and restore individual keys
        log "WARNING: KV store restoration requires manual intervention for $kv_name"
      fi
    done
    
    # Restore Object stores
    log "Restoring Object stores..."
    if [ -d "objects" ]; then
      for obj_dir in objects/*; do
        if [ -d "$obj_dir" ]; then
          obj_name=$(basename "$obj_dir")
          log "Restoring Object store: $obj_name"
          ${pkgs.natscli}/bin/nats --server="$NATS_URL" object put "$obj_name" "$obj_dir"/* || log "WARNING: Failed to restore object store $obj_name"
        fi
      done
    fi
    
    # Restore configuration files (manual step)
    if [ -d "config" ]; then
      log "Configuration files available in: $PWD/config"
      log "Manual intervention required to restore configuration files"
    fi
    
    log "Restore completed. Please verify system state."
    
    # Send notification
    ${optionalString (cfg.notifications.webhook != null) ''
      curl -X POST "${cfg.notifications.webhook}" \
        -H "Content-Type: application/json" \
        -d "{\"status\": \"restore_completed\", \"backup_file\": \"$BACKUP_FILE\", \"timestamp\": \"$(date -Iseconds)\"}"
    ''}
  '';

in {
  ###### Interface
  options.services.cim-claude-backup = {
    enable = mkEnableOption "CIM Claude Adapter backup and restore system";

    # Backup configuration
    backupDir = mkOption {
      type = types.str;
      default = "/var/lib/cim-claude-backup";
      description = "Directory to store backups.";
    };

    # Schedule configuration
    schedule = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable scheduled backups.";
      };

      frequency = mkOption {
        type = types.str;
        default = "daily";
        description = "Backup frequency (systemd timer format).";
        example = "hourly";
      };

      randomDelay = mkOption {
        type = types.str;
        default = "1h";
        description = "Random delay to spread backup load.";
      };

      persistent = mkOption {
        type = types.bool;
        default = true;
        description = "Run missed backups on boot.";
      };
    };

    # NATS-specific backup configuration
    nats = {
      url = mkOption {
        type = types.str;
        default = "nats://localhost:4222";
        description = "NATS server URL for backup operations.";
      };

      streams = mkOption {
        type = types.listOf types.str;
        default = [
          "CIM_CLAUDE_CONV_CMD"
          "CIM_CLAUDE_CONV_EVT"
          "CIM_CLAUDE_ATTACH_CMD"
          "CIM_CLAUDE_ATTACH_EVT"
          "CIM_CLAUDE_CONV_QRY"
          "CIM_SYS_HEALTH_EVT"
        ];
        description = "JetStream streams to backup.";
      };

      kvStores = mkOption {
        type = types.listOf types.str;
        default = [
          "CIM_CLAUDE_CONV_KV"
          "CIM_CLAUDE_ATTACH_KV"
          "CIM_CLAUDE_SESSION_KV"
          "CIM_CLAUDE_CONFIG_KV"
          "CIM_CLAUDE_METRICS_KV"
        ];
        description = "KV stores to backup.";
      };

      objectStores = mkOption {
        type = types.listOf types.str;
        default = [
          "CIM_CLAUDE_ATTACH_OBJ_IMG"
          "CIM_CLAUDE_ATTACH_OBJ_DOC"
          "CIM_CLAUDE_ATTACH_OBJ_CODE"
          "CIM_CLAUDE_ATTACH_OBJ_AUDIO"
          "CIM_CLAUDE_ATTACH_OBJ_VIDEO"
          "CIM_CLAUDE_ATTACH_OBJ_BIN"
        ];
        description = "Object stores to backup.";
      };
    };

    # Configuration files to backup
    configPaths = mkOption {
      type = types.listOf types.str;
      default = [
        "/etc/cim-claude-adapter"
        "/etc/nats-server"
      ];
      description = "Configuration directories/files to backup.";
    };

    # Compression settings
    compression = {
      type = mkOption {
        type = types.enum [ "none" "gzip" "zstd" ];
        default = "zstd";
        description = "Compression algorithm for backups.";
      };

      level = mkOption {
        type = types.ints.between 1 22;
        default = 3;
        description = "Compression level (1-22 for zstd, 1-9 for gzip).";
      };
    };

    # Checksum verification
    checksum = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable checksum generation and verification.";
      };

      algorithm = mkOption {
        type = types.enum [ "sha256" "sha512" "md5" ];
        default = "sha256";
        description = "Checksum algorithm.";
      };
    };

    # Retention policy
    retention = {
      days = mkOption {
        type = types.int;
        default = 30;
        description = "Number of days to keep backups.";
      };

      maxBackups = mkOption {
        type = types.nullOr types.int;
        default = null;
        description = "Maximum number of backups to keep (null for unlimited).";
      };
    };

    # Remote storage configuration
    remote = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable remote backup storage.";
      };

      type = mkOption {
        type = types.enum [ "s3" "restic" "rsync" ];
        default = "s3";
        description = "Remote storage type.";
      };

      # S3 configuration
      bucket = mkOption {
        type = types.str;
        default = "";
        description = "S3 bucket name (for S3 storage).";
      };

      path = mkOption {
        type = types.str;
        default = "cim-claude-backups";
        description = "Remote storage path/prefix.";
      };

      # Restic configuration
      repo = mkOption {
        type = types.str;
        default = "";
        description = "Restic repository URL.";
      };

      # Rsync configuration
      destination = mkOption {
        type = types.str;
        default = "";
        description = "Rsync destination (user@host:/path).";
      };

      # Credentials
      credentialsFile = mkOption {
        type = types.nullOr types.path;
        default = null;
        description = "Path to credentials file for remote storage.";
      };
    };

    # Notification settings
    notifications = {
      webhook = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Webhook URL for backup notifications.";
      };

      email = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "Email address for backup notifications.";
      };

      onFailureOnly = mkOption {
        type = types.bool;
        default = false;
        description = "Only send notifications on backup failures.";
      };
    };

    # Health monitoring
    monitoring = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable backup monitoring and metrics.";
      };

      metricsPort = mkOption {
        type = types.port;
        default = 9091;
        description = "Port for backup metrics endpoint.";
      };
    };
  };

  ###### Implementation
  config = mkIf cfg.enable {
    # Create backup user and directories
    users.users.cim-claude-backup = {
      description = "CIM Claude Backup Service";
      group = "cim-claude-backup";
      home = cfg.backupDir;
      createHome = true;
      homeMode = "750";
      isSystemUser = true;
    };

    users.groups.cim-claude-backup = {};

    # Backup directories
    systemd.tmpfiles.rules = [
      "d ${cfg.backupDir} 0750 cim-claude-backup cim-claude-backup -"
      "d /var/log/cim-claude-backup 0755 cim-claude-backup cim-claude-backup -"
    ];

    # Backup service
    systemd.services.cim-claude-backup = {
      description = "CIM Claude Adapter Backup";
      after = [ "network-online.target" "nats-server.service" "cim-claude-adapter.service" ];
      wants = [ "network-online.target" ];
      
      serviceConfig = {
        Type = "oneshot";
        User = "cim-claude-backup";
        Group = "cim-claude-backup";
        ExecStart = backupScript;
        
        # Security settings
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.backupDir "/var/log/cim-claude-backup" ];
        
        # Environment
        Environment = [
          "BACKUP_DIR=${cfg.backupDir}"
          "NATS_URL=${cfg.nats.url}"
        ] ++ optional (cfg.remote.credentialsFile != null) "CREDENTIALS_FILE=${cfg.remote.credentialsFile}";
      };
    };

    # Scheduled backup timer
    systemd.timers.cim-claude-backup = mkIf cfg.schedule.enable {
      description = "CIM Claude Adapter Backup Timer";
      wantedBy = [ "timers.target" ];
      
      timerConfig = {
        OnCalendar = cfg.schedule.frequency;
        RandomizedDelaySec = cfg.schedule.randomDelay;
        Persistent = cfg.schedule.persistent;
      };
    };

    # Restore service (manual trigger)
    systemd.services.cim-claude-restore = {
      description = "CIM Claude Adapter Restore";
      after = [ "network-online.target" "nats-server.service" ];
      wants = [ "network-online.target" ];
      
      serviceConfig = {
        Type = "oneshot";
        User = "cim-claude-backup";
        Group = "cim-claude-backup";
        ExecStart = "${restoreScript} %i";
        
        # Security settings
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        ReadWritePaths = [ cfg.backupDir "/tmp" ];
        
        # Environment
        Environment = [
          "NATS_URL=${cfg.nats.url}"
        ];
      };
    };

    # Backup monitoring service
    systemd.services.cim-claude-backup-monitor = mkIf cfg.monitoring.enable {
      description = "CIM Claude Backup Monitoring";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      
      serviceConfig = {
        Type = "exec";
        User = "cim-claude-backup";
        Group = "cim-claude-backup";
        ExecStart = pkgs.writeShellScript "backup-monitor" ''
          set -euo pipefail
          
          # Simple HTTP server for metrics
          ${pkgs.python3}/bin/python3 -c "
import http.server
import json
import os
import time
from pathlib import Path

class BackupMetricsHandler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/metrics':
            self.send_response(200)
            self.send_header('Content-type', 'text/plain')
            self.end_headers()
            
            metrics = []
            backup_dir = Path('${cfg.backupDir}')
            
            # Count backups
            backups = list(backup_dir.glob('cim-claude-backup-*.tar.*'))
            metrics.append(f'cim_claude_backup_count {{}} {len(backups)}')
            
            # Latest backup timestamp
            if backups:
                latest = max(backups, key=os.path.getmtime)
                latest_time = os.path.getmtime(latest)
                metrics.append(f'cim_claude_backup_latest_timestamp {{}} {latest_time}')
                
                # Backup size
                size = os.path.getsize(latest)
                metrics.append(f'cim_claude_backup_latest_size_bytes {{}} {size}')
            
            # Disk usage
            statvfs = os.statvfs(backup_dir)
            free_bytes = statvfs.f_frsize * statvfs.f_bavail
            total_bytes = statvfs.f_frsize * statvfs.f_blocks
            metrics.append(f'cim_claude_backup_disk_free_bytes {{}} {free_bytes}')
            metrics.append(f'cim_claude_backup_disk_total_bytes {{}} {total_bytes}')
            
            self.wfile.write('\\n'.join(metrics).encode() + b'\\n')
        else:
            self.send_response(404)
            self.end_headers()

if __name__ == '__main__':
    os.chdir('${cfg.backupDir}')
    server = http.server.HTTPServer(('0.0.0.0', ${toString cfg.monitoring.metricsPort}), BackupMetricsHandler)
    server.serve_forever()
"
        '';
        
        Restart = "always";
        RestartSec = "10s";
      };
    };

    # Install backup and restore scripts
    environment.systemPackages = [
      (pkgs.writeShellScriptBin "cim-claude-backup" ''
        systemctl start cim-claude-backup
      '')
      (pkgs.writeShellScriptBin "cim-claude-restore" ''
        if [ $# -eq 0 ]; then
          echo "Usage: cim-claude-restore <backup-file>"
          echo "Available backups:"
          ls -la ${cfg.backupDir}/cim-claude-backup-*.tar.* 2>/dev/null || echo "No backups found"
          exit 1
        fi
        sudo systemd-run --uid=cim-claude-backup --gid=cim-claude-backup \
          --setenv=FORCE=true \
          ${restoreScript} "$1"
      '')
    ];

    # Package dependencies
    environment.systemPackages = with pkgs; [
      natscli
      gzip
      jq
    ] ++ optional (cfg.compression.type == "zstd") zstd
      ++ optional (cfg.remote.type == "s3") awscli2
      ++ optional (cfg.remote.type == "restic") restic
      ++ optional (cfg.remote.type == "rsync") rsync;

    # Logrotate for backup logs
    services.logrotate.settings."/var/log/cim-claude-backup/*.log" = {
      frequency = "weekly";
      rotate = 52;
      compress = true;
      delaycompress = true;
      missingok = true;
      notifempty = true;
      create = "640 cim-claude-backup cim-claude-backup";
    };

    # Assertions
    assertions = [
      {
        assertion = cfg.remote.enable && cfg.remote.type == "s3" -> cfg.remote.bucket != "";
        message = "S3 bucket must be specified when using S3 remote storage";
      }
      {
        assertion = cfg.remote.enable && cfg.remote.type == "restic" -> cfg.remote.repo != "";
        message = "Restic repository must be specified when using Restic remote storage";
      }
      {
        assertion = cfg.remote.enable && cfg.remote.type == "rsync" -> cfg.remote.destination != "";
        message = "Rsync destination must be specified when using rsync remote storage";
      }
      {
        assertion = cfg.compression.type == "gzip" -> cfg.compression.level <= 9;
        message = "Gzip compression level must be between 1-9";
      }
    ];

    # Warnings
    warnings = 
      optional (!cfg.schedule.enable) "Automatic backup scheduling is disabled" ++
      optional (!cfg.remote.enable) "Remote backup storage is disabled - backups are only stored locally" ++
      optional (cfg.notifications.webhook == null && cfg.notifications.email == null) "No backup notifications configured";
  };

  # Meta information
  meta = {
    maintainers = [ "Cowboy AI, LLC <backup@cowboy-ai.com>" ];
    doc = ./backup-restore.md;
  };
}