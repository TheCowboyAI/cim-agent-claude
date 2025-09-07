#!/usr/bin/env bash

# show_latest_cursor_url.sh
# Script to display the latest Cursor download URL
# Uses the oslook/cursor-ai-downloads repository

set -e

# Architecture (x64 or arm64)
ARCH=${1:-"x64"}

if [[ "$ARCH" != "x64" && "$ARCH" != "arm64" ]]; then
  echo "Error: Architecture must be either 'x64' or 'arm64'" >&2
  echo "Usage: $0 [x64|arm64]" >&2
  exit 1
fi

# URL to the version-history.json file in the repo
VERSION_HISTORY_URL="https://raw.githubusercontent.com/oslook/cursor-ai-downloads/main/version-history.json"

# Check if jq is installed
if ! command -v jq &> /dev/null; then
  echo "Error: jq is required but not installed" >&2
  echo "Please install it with: nix-env -iA nixos.jq or use nix develop" >&2
  exit 1
fi

# Fetch the version history
TMP_FILE=$(mktemp)
curl -s "$VERSION_HISTORY_URL" -o "$TMP_FILE"

# Determine JSON structure
JSON_TYPE=$(jq -r 'type' "$TMP_FILE")

if [[ "$JSON_TYPE" == "array" ]]; then
  # New array format
  LINUX_ENTRIES=$(jq -r '[.[] | select(.linux.appImage.x64 != null)] | length' "$TMP_FILE")
  
  if [ "$LINUX_ENTRIES" -gt 0 ]; then
    # Get latest version
    LATEST_ENTRY=$(jq -r '[.[] | select(.linux.appImage.x64 != null)] | sort_by(.version | split(".") | map(tonumber)) | reverse | .[0]' "$TMP_FILE")
    
    # Extract version
    VERSION=$(echo "$LATEST_ENTRY" | jq -r '.version')
    
    # Extract download URL for the architecture
    DOWNLOAD_URL=$(echo "$LATEST_ENTRY" | jq -r ".linux.appImage.${ARCH}")
  else
    echo "Error: No Linux AppImage entries found" >&2
    rm -f "$TMP_FILE"
    exit 1
  fi
else
  # Original format with build_id
  LATEST_ENTRY=$(jq -r 'sort_by(.version | split(".") | map(tonumber)) | reverse | .[0]' "$TMP_FILE")
  
  # Extract version, build ID
  VERSION=$(echo "$LATEST_ENTRY" | jq -r '.version')
  BUILD_ID=$(echo "$LATEST_ENTRY" | jq -r '.build_id')
  
  # Construct the direct download URL
  DOWNLOAD_URL="https://downloader.cursor.sh/builds/${BUILD_ID}/linux/appImage/${ARCH}"
fi

# Cleanup
rm -f "$TMP_FILE"

# Just output the URL (for use in scripts)
echo "$DOWNLOAD_URL"

# If running interactively, show more info
if [ -t 1 ]; then
  echo "Latest version: $VERSION" >&2
  echo "Architecture: $ARCH" >&2
  echo "Full download command:" >&2
  echo "curl -L \"$DOWNLOAD_URL\" -o Cursor-${VERSION}-${ARCH}.AppImage" >&2
fi 