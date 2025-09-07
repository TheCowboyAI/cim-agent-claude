#!/usr/bin/env bash
set -euo pipefail

# Create temporary directory for keys
TMPDIR=$(mktemp -d)
echo "Creating temporary directory for keys: $TMPDIR"

# Generate the key pair
echo "Generating Nix signing key pair..."
nix-store --generate-binary-cache-key "$(hostname)" "$TMPDIR/secret-key" "$TMPDIR/public-key"

# Set permissions
chmod 600 "$TMPDIR/secret-key"
chmod 644 "$TMPDIR/public-key"

# Output instructions
echo "Key generation complete!"
echo "Private key: $TMPDIR/secret-key"
echo "Public key: $TMPDIR/public-key"
echo ""
echo "IMPORTANT: These keys are in a temporary directory. You need to move them to a permanent location."
echo "Run these commands to move the keys to /var/sign:"
echo "  sudo mkdir -p /var/sign"
echo "  sudo cp \"$TMPDIR/secret-key\" /var/sign/"
echo "  sudo cp \"$TMPDIR/public-key\" /var/sign/"
echo "  sudo chmod 400 /var/sign/secret-key"
echo "  sudo chmod 644 /var/sign/public-key"
echo ""
echo "To use this key with nix-serve, the configuration is already set up with:"
echo "services.nix-serve.secretKeyFile = \"/var/sign/secret-key\";"
echo ""
echo "For client machines, add this to their nix.conf:"
echo "trusted-public-keys = $(cat "$TMPDIR/public-key")"
echo ""
echo "Keep a copy of the public key output for your records."