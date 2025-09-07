# Example configuration for mounting S3 as NATS JetStream storage
{ config, pkgs, ... }:

{
  # Import the s3fs module
  imports = [ ./modules/nixos/nats-s3fs.nix ];
  
  # Configure S3FS mount
  services.natsS3fs = {
    enable = true;
    bucket = "nats-jetstream-storage";
    endpoint = "https://s3.wasabisys.com";
    mountPoint = "/var/lib/nats/jetstream";
    credentialsFile = "/etc/s3fs/credentials"; # Format: ACCESS_KEY:SECRET_KEY
    
    # Performance tuning options
    options = [
      "use_cache=/var/cache/s3fs"
      "ensure_diskfree=2048"      # Keep 2GB free
      "parallel_count=20"          # Parallel transfers
      "multipart_size=128"         # 128MB chunks
      "max_stat_cache_size=200000" # Larger stat cache
      "stat_cache_expire=600"      # 10 minute cache
      "enable_noobj_cache"         # Cache negative lookups
      "use_sse"                    # Server-side encryption
    ];
  };
  
  # NATS will automatically use the S3-backed mount
  services.nats = {
    enable = true;
    jetstream = true;
    settings = {
      jetstream = {
        store_dir = "/var/lib/nats/jetstream";
        # Consider limiting memory usage since S3 is slower
        max_memory_store = "1G";
      };
    };
  };
  
  # Create credentials file securely
  systemd.tmpfiles.rules = [
    "f /etc/s3fs/credentials 0600 root root - ACCESS_KEY:SECRET_KEY"
  ];
} 