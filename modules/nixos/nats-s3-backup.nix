{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.natsS3Backup;
in {
  options.services.natsS3Backup = {
    enable = mkEnableOption "NATS JetStream S3 backup";
    
    interval = mkOption {
      type = types.str;
      default = "hourly";
      description = "Systemd timer interval for backups";
    };
    
    s3Config = {
      endpoint = mkOption {
        type = types.str;
        example = "s3.wasabisys.com";
        description = "S3-compatible endpoint";
      };
      
      bucket = mkOption {
        type = types.str;
        example = "nats-backup";
        description = "S3 bucket name";
      };
      
      region = mkOption {
        type = types.str;
        default = "us-east-1";
        description = "S3 region";
      };
      
      accessKeyFile = mkOption {
        type = types.path;
        description = "File containing S3 access key";
      };
      
      secretKeyFile = mkOption {
        type = types.path;
        description = "File containing S3 secret key";
      };
    };
    
    jetStreamDir = mkOption {
      type = types.path;
      default = "/var/lib/nats/jetstream";
      description = "JetStream data directory to backup";
    };
  };
  
  config = mkIf cfg.enable {
    systemd.services.nats-s3-backup = {
      description = "Backup NATS JetStream to S3";
      after = [ "network.target" "nats.service" ];
      
      script = ''
        # Load S3 credentials
        export AWS_ACCESS_KEY_ID=$(cat ${cfg.s3Config.accessKeyFile})
        export AWS_SECRET_ACCESS_KEY=$(cat ${cfg.s3Config.secretKeyFile})
        
        # Configure rclone on the fly
        export RCLONE_CONFIG_S3_TYPE=s3
        export RCLONE_CONFIG_S3_PROVIDER=Other
        export RCLONE_CONFIG_S3_ENDPOINT=${cfg.s3Config.endpoint}
        export RCLONE_CONFIG_S3_REGION=${cfg.s3Config.region}
        export RCLONE_CONFIG_S3_ACCESS_KEY_ID=$AWS_ACCESS_KEY_ID
        export RCLONE_CONFIG_S3_SECRET_ACCESS_KEY=$AWS_SECRET_ACCESS_KEY
        
        # Sync JetStream data to S3
        ${pkgs.rclone}/bin/rclone sync \
          ${cfg.jetStreamDir} \
          s3:${cfg.s3Config.bucket}/jetstream-backup/$(hostname)/$(date +%Y%m%d-%H%M%S)/ \
          --transfers 4 \
          --checkers 8 \
          --fast-list
          
        # Optional: Keep only last N backups
        ${pkgs.rclone}/bin/rclone delete \
          s3:${cfg.s3Config.bucket}/jetstream-backup/$(hostname)/ \
          --min-age 7d
      '';
      
      serviceConfig = {
        Type = "oneshot";
        User = "nats";
        Group = "nats";
      };
    };
    
    systemd.timers.nats-s3-backup = {
      description = "Timer for NATS S3 backup";
      wantedBy = [ "timers.target" ];
      timerConfig = {
        OnCalendar = cfg.interval;
        Persistent = true;
      };
    };
  };
} 