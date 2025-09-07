#!/usr/bin/env bash

# fetch_latest_cursor.sh
# Script to download the latest Cursor AppImage
# Uses the oslook/cursor-ai-downloads repository

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Fetching and downloading latest Cursor...${NC}"

# Where to save the AppImage
OUTPUT_DIR=${1:-"$PWD"}
ARCH=${2:-"x64"}

if [[ "$ARCH" != "x64" && "$ARCH" != "arm64" ]]; then
  echo -e "${RED}Error: Architecture must be either 'x64' or 'arm64'${NC}"
  echo -e "Usage: $0 [output_directory] [x64|arm64]"
  exit 1
fi

# URL to the version-history.json file in the repo
VERSION_HISTORY_URL="https://raw.githubusercontent.com/oslook/cursor-ai-downloads/main/version-history.json"

# Temporary file
TMP_FILE=$(mktemp)

# Fetch the version history JSON file
echo -e "${YELLOW}Downloading version history...${NC}"
if ! curl -s "$VERSION_HISTORY_URL" -o "$TMP_FILE"; then
  echo -e "${RED}Error: Failed to download version history${NC}"
  rm -f "$TMP_FILE"
  exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
  echo -e "${RED}Error: jq is required but not installed${NC}"
  echo -e "Please install it with: nix-env -iA nixos.jq or use nix develop"
  rm -f "$TMP_FILE"
  exit 1
fi

# Debug information
echo -e "${YELLOW}Analyzing JSON structure...${NC}"

# Check if it has a versions array (new format)
if jq -e '.versions' "$TMP_FILE" > /dev/null 2>&1; then
  echo -e "${YELLOW}Found new format with versions array...${NC}"
  
  # Get the latest version from versions array
  LATEST_ENTRY=$(jq -r '.versions | sort_by(.version | split(".") | map(tonumber)) | reverse | .[0]' "$TMP_FILE")
  
  # Extract version
  VERSION=$(echo "$LATEST_ENTRY" | jq -r '.version')
  
  # Convert architecture format (x64 -> linux-x64)
  if [[ "$ARCH" == "x64" ]]; then
    PLATFORM_KEY="linux-x64"
  else
    PLATFORM_KEY="linux-arm64"
  fi
  
  # Extract download URL for the architecture
  DOWNLOAD_URL=$(echo "$LATEST_ENTRY" | jq -r ".platforms.\"${PLATFORM_KEY}\"")
  
  if [[ "$DOWNLOAD_URL" == "null" || -z "$DOWNLOAD_URL" ]]; then
    echo -e "${RED}Error: No Linux AppImage found for architecture ${ARCH}${NC}"
    rm -f "$TMP_FILE"
    exit 1
  fi
else
  # Check if it's a direct array (old format)
  JSON_TYPE=$(jq -r 'type' "$TMP_FILE")
  if [[ "$JSON_TYPE" == "array" ]]; then
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
fi

# Cleanup
rm -f "$TMP_FILE"

OUTPUT_FILE="${OUTPUT_DIR}/Cursor-${VERSION}-${ARCH}.AppImage"

echo -e "${GREEN}Latest Cursor version: ${YELLOW}${VERSION}${NC}"
echo -e "${GREEN}Download URL: ${YELLOW}${DOWNLOAD_URL}${NC}"
echo -e "${GREEN}Saving to: ${YELLOW}${OUTPUT_FILE}${NC}"

# Download the AppImage
echo -e "${YELLOW}Downloading AppImage...${NC}"
if ! curl -L "$DOWNLOAD_URL" -o "$OUTPUT_FILE"; then
  echo -e "${RED}Error: Failed to download AppImage${NC}"
  exit 1
fi

# Make it executable
chmod +x "$OUTPUT_FILE"

echo -e "${GREEN}Successfully downloaded Cursor ${VERSION}!${NC}"
echo -e "${GREEN}You can run it with: ${YELLOW}${OUTPUT_FILE}${NC}" 