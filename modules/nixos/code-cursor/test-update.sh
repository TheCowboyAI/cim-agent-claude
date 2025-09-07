#!/usr/bin/env bash

# Current Cursor version
CURRENT_VERSION="0.50.4"

# Get current version from API
API_URL="https://www.cursor.com/api/download?platform=linux-x64&releaseTrack=stable"
LATEST_VERSION=$(curl -s "$API_URL" | jq -r '.version')
DOWNLOAD_URL=$(curl -s "$API_URL" | jq -r '.downloadUrl')

echo "Current Cursor version: $CURRENT_VERSION"
echo "Latest Cursor version: $LATEST_VERSION"

if [ "$CURRENT_VERSION" != "$LATEST_VERSION" ]; then
  echo "Update available: $CURRENT_VERSION -> $LATEST_VERSION"
  
  # Calculate the hash for the latest version
  echo "Fetching hash for the latest version..."
  echo "Download URL: $DOWNLOAD_URL"
  HASH=$(nix-prefetch-url --type sha256 "$DOWNLOAD_URL" 2>/dev/null)
  
  echo "To update, edit modules/nixos/code-cursor/default.nix and change:"
  echo "  cursorVersion = \"$LATEST_VERSION\";"
  echo "  cursorHash = \"$HASH\";"
else
  echo "You are using the latest version."
fi 