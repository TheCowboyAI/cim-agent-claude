{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.natsS3fs;
in {
  options.services.natsS3fs = {
    enable = mkEnableOption "Mount S3 bucket as NATS JetStream storage";
    
    bucket = mkOption {
      type = types.str;
      example = "nats-storage";
      description = "S3 bucket to mount";
    };
    
    endpoint = mkOption {
      type = types.str;
      example = "https://s3.wasabisys.com";
      description = "S3-compatible endpoint URL";
    };
    
    mountPoint = mkOption {
      type = types.path;
      default = "/var/lib/nats/jetstream";
      description = "Where to mount the S3 bucket";
    };
    
    credentialsFile = mkOption {
      type = types.path;
      description = "File containing ACCESS_KEY:SECRET_KEY";
    };
    
    cacheDir = mkOption {
      type = types.path;
      default = "/var/cache/s3fs";
      description = "Local cache directory for s3fs";
    };
    
    options = mkOption {
      type = types.listOf types.str;
      default = [
        "use_cache=${cfg.cacheDir}"
        "ensure_diskfree=1024"
        "parallel_count=10"
        "multipart_size=64"
        "max_stat_cache_size=100000"
        "stat_cache_expire=300"
      ];
      description = "Additional s3fs mount options";
    };
  };
  
  config = mkIf cfg.enable {
    # Create cache directory
    systemd.tmpfiles.rules = [
      "d ${cfg.cacheDir} 0700 nats nats -"
    ];
    
    # Mount S3 bucket before NATS starts
    systemd.services.nats-s3fs-mount = {
      description = "Mount S3 bucket for NATS JetStream";
      before = [ "nats.service" ];
      wantedBy = [ "multi-user.target" ];
      
      serviceConfig = {
        Type = "forking";
        ExecStart = ''
          ${pkgs.s3fs}/bin/s3fs ${cfg.bucket} ${cfg.mountPoint} \
            -o passwd_file=${cfg.credentialsFile} \
            -o url=${cfg.endpoint} \
            -o ${concatStringsSep "," cfg.options} \
            -o allow_other \
            -o uid=$(id -u nats) \
            -o gid=$(id -g nats)
        '';
        ExecStop = "${pkgs.fuse}/bin/fusermount -u ${cfg.mountPoint}";
        Restart = "on-failure";
        RestartSec = "10s";
      };
      
      preStart = ''
        mkdir -p ${cfg.mountPoint}
        chown nats:nats ${cfg.mountPoint}
      '';
    };
    
    # Ensure NATS starts after mount
    systemd.services.nats = {
      after = [ "nats-s3fs-mount.service" ];
      requires = [ "nats-s3fs-mount.service" ];
    };
    
    # Install s3fs package
    environment.systemPackages = [ pkgs.s3fs ];
  };
} 