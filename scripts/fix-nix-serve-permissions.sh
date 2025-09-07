#!/usr/bin/env bash
set -euo pipefail

# Check if the secret key exists
if [ ! -f /var/sign/secret-key ]; then
  echo "Error: Secret key file doesn't exist at /var/sign/secret-key"
  exit 1
fi

# Make keys readable by the nix-serve user
# The systemd service is typically running with DynamicUser=true
# We need to make the directory and key accessible to the service
echo "Fixing permissions on signing key..."
sudo chmod 755 /var/sign
sudo chmod 644 /var/sign/secret-key
sudo chmod 644 /var/sign/public-key

echo "Setting up proper ownership..."
# Temporary fix to make sure the key is accessible by all, including the dynamically created nix-serve user
sudo chmod o+r /var/sign/secret-key

# Print status
echo "Current permissions:"
ls -la /var/sign/

echo "Done! Try rebuilding with: sudo nixos-rebuild switch" 