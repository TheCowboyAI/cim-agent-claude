#!/usr/bin/env bash

# get_cursor_download.sh
# Script to fetch the latest Cursor download URL for Linux (AppImage)
# Author: Your Name
# License: MIT

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Fetching latest Cursor download information...${NC}"

# URL to the version-history.json file in the repo
VERSION_HISTORY_URL="https://raw.githubusercontent.com/oslook/cursor-ai-downloads/main/version-history.json"

# Architecture (x64 or arm64)
ARCH=${1:-"x64"}

if [[ "$ARCH" != "x64" && "$ARCH" != "arm64" ]]; then
  echo -e "${RED}Error: Architecture must be either 'x64' or 'arm64'${NC}"
  echo -e "Usage: $0 [x64|arm64]"
  exit 1
fi

# Temporary file
TMP_FILE=$(mktemp)

# Fetch the version history JSON file
echo -e "${YELLOW}Downloading version history...${NC}"
if ! curl -s "$VERSION_HISTORY_URL" -o "$TMP_FILE"; then
  echo -e "${RED}Error: Failed to download version history${NC}"
  rm -f "$TMP_FILE"
  exit 1
fi

# Debug information
echo -e "${YELLOW}Analyzing JSON structure...${NC}"
JSON_TYPE=$(jq -r 'type' "$TMP_FILE")
echo -e "JSON structure is: ${BLUE}${JSON_TYPE}${NC}"

if [[ "$JSON_TYPE" == "array" ]]; then
  # Handle array format
  echo -e "${YELLOW}Found array format, looking for entries with Linux AppImage...${NC}"
  
  # Count Linux AppImage entries
  LINUX_ENTRIES=$(jq -r '[.[] | select(.linux.appImage.x64 != null)] | length' "$TMP_FILE")
  echo -e "Found ${BLUE}${LINUX_ENTRIES}${NC} entries with Linux AppImage support"
  
  if [ "$LINUX_ENTRIES" -gt 0 ]; then
    # Get latest version
    LATEST_ENTRY=$(jq -r '[.[] | select(.linux.appImage.x64 != null)] | sort_by(.version | split(".") | map(tonumber)) | reverse | .[0]' "$TMP_FILE")
    
    # Extract version
    VERSION=$(echo "$LATEST_ENTRY" | jq -r '.version')
    
    # Extract download URL for the architecture
    DOWNLOAD_URL=$(echo "$LATEST_ENTRY" | jq -r ".linux.appImage.${ARCH}")
  else
    echo -e "${RED}Error: No Linux AppImage entries found${NC}"
    rm -f "$TMP_FILE"
    exit 1
  fi
else
  # Original format with build_id
  echo -e "${YELLOW}Found original format, extracting version and build_id...${NC}"
  
  # Parse the JSON to get the latest version entry
  LATEST_ENTRY=$(jq -r 'sort_by(.version | split(".") | map(tonumber)) | reverse | .[0]' "$TMP_FILE")
  
  # Extract version, build ID
  VERSION=$(echo "$LATEST_ENTRY" | jq -r '.version')
  BUILD_ID=$(echo "$LATEST_ENTRY" | jq -r '.build_id')
  
  # Construct the direct download URL
  DOWNLOAD_URL="https://downloader.cursor.sh/builds/${BUILD_ID}/linux/appImage/${ARCH}"
fi

# Cleanup
rm -f "$TMP_FILE"

echo -e "${GREEN}Latest Cursor version: ${YELLOW}${VERSION}${NC}"
echo -e "${GREEN}Download URL: ${YELLOW}${DOWNLOAD_URL}${NC}"

# If you want to use this in a script
echo -e "${GREEN}To download directly:${NC}"
echo -e "curl -L \"${DOWNLOAD_URL}\" -o Cursor-${VERSION}-${ARCH}.AppImage"

# Return the URL so it can be captured by other scripts
echo "$DOWNLOAD_URL" 