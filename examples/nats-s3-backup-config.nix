# Example configuration for backing up NATS JetStream to S3
{ config, ... }:

{
  # Import the backup module
  imports = [ ./modules/nixos/nats-s3-backup.nix ];
  
  # Enable NATS with JetStream
  services.nats = {
    enable = true;
    jetstream = true;
    settings = {
      jetstream = {
        store_dir = "/var/lib/nats/jetstream";
      };
    };
  };
  
  # Configure S3 backup
  services.natsS3Backup = {
    enable = true;
    interval = "hourly"; # or "daily", "weekly", etc.
    
    s3Config = {
      # For Wasabi
      endpoint = "s3.wasabisys.com";
      bucket = "my-nats-backups";
      region = "us-east-1";
      
      # For MinIO (local)
      # endpoint = "localhost:9000";
      # bucket = "nats-backup";
      # region = "us-east-1";
      
      # Store credentials securely
      accessKeyFile = "/run/secrets/s3-access-key";
      secretKeyFile = "/run/secrets/s3-secret-key";
    };
    
    jetStreamDir = "/var/lib/nats/jetstream";
  };
} 