{
  lib,
  config,
  pkgs,
  cursor-version-history,
  ...
}:
with lib; let
  cfg = config.code-cursor;
  
  # Specify the desired cursor version - update this manually when needed
  cursorVersion = "1.5.11";
  # SRI hash must be manually verified and updated for security
  cursorHash = "sha256-PlZPgcDe6KmEcQYDk1R4uXh1R34mKuPLBh/wbOAYrAY=";
  
  # Parse version history JSON
  versionData = builtins.fromJSON (builtins.readFile "${cursor-version-history}/version-history.json");
  
  # Find the URL for the specified version
  getUrlForVersion = version:
    let
      matchingVersions = filter (v: v.version == version) versionData.versions;
      versionEntry = if (length matchingVersions > 0) then (head matchingVersions) else null;
    in
      if versionEntry != null && versionEntry.platforms ? "linux-x64"
      then versionEntry.platforms."linux-x64"
      else "https://downloads.cursor.com/production/faa03b17cce93e8a80b7d62d57f5eda6bb6ab9fa/linux/x64/Cursor-${version}-x86_64.AppImage";
  
  # Get the latest available version
  latestVersion = (head versionData.versions).version;
  
  # Check if an update is available
  updateAvailable = cursorVersion != latestVersion;
  
  # Get URL for current version
  cursorUrl = getUrlForVersion cursorVersion;
  
  # Build the cursor package
  cursorPackage = pkgs.appimageTools.wrapType2 {
    pname = "cursor";
    version = cursorVersion;
    src = pkgs.fetchurl {
      url = cursorUrl;
      hash = cursorHash;
    };
    extraPkgs = pkgs: with pkgs; [
      xorg.libxshmfence
      libglvnd
      libsecret
      nodePackages.node-gyp
      libappindicator-gtk3
      nss
    ];
    extraInstallCommands = ''
      mkdir -p $out/lib
      ln -s ${pkgs.libsecret}/lib/libsecret-1.so.0 $out/lib/
    '';
  };

  # Create a simple script to check for cursor updates
  checkUpdatesScript = pkgs.writeScriptBin "check-cursor-updates" ''
    #!/usr/bin/env bash
    
    # Get current version from API
    API_URL="https://www.cursor.com/api/download?platform=linux-x64&releaseTrack=stable"
    LATEST_VERSION=$(${pkgs.curl}/bin/curl -sL "$API_URL" | ${pkgs.jq}/bin/jq -r '.version')
    DOWNLOAD_URL=$(${pkgs.curl}/bin/curl -sL "$API_URL" | ${pkgs.jq}/bin/jq -r '.downloadUrl')
    
    echo "Current Cursor version: ${cursorVersion}"
    echo "Latest Cursor version: $LATEST_VERSION"
    
    if [ "${cursorVersion}" != "$LATEST_VERSION" ]; then
      echo "Update available: ${cursorVersion} -> $LATEST_VERSION"
      
      # Calculate the SRI hash for the latest version
      echo "Fetching SRI hash for the latest version..."
      HASH=$(nix-prefetch-url --type sha256 "$DOWNLOAD_URL" 2>/dev/null)
      SRI_HASH=$(nix hash to-sri --type sha256 "$HASH" 2>/dev/null)
      
      echo "To update, edit modules/nixos/code-cursor/default.nix and change:"
      echo "  cursorVersion = \"$LATEST_VERSION\";"
      echo "  cursorHash = \"$SRI_HASH\";"
    else
      echo "You are using the latest version."
    fi
  '';
  
  # Load cursor settings
  cursorSettingsFile = ./settings.json;
  cursorSettings = builtins.readFile cursorSettingsFile;
  
  # Create a wrapper script
  cursorWrapper = pkgs.writeScriptBin "cursor-with-settings" (builtins.readFile ./cursor-wrapper.sh);
  
  # Create a package with the settings file
  cursorSettingsPackage = pkgs.writeTextFile {
    name = "cursor-settings";
    text = cursorSettings;
    destination = "/etc/cursor/settings.json";
  };
in {
  # Module options
  options.code-cursor = {
    enable = mkEnableOption "Enable Cursor AI code editor";
    
    package = mkOption {
      type = types.package;
      default = cursorPackage;
      description = "The Cursor package to use.";
    };
    
    notifyUpdates = mkOption {
      type = types.bool;
      default = true;
      description = "Show a notification when Cursor updates are available.";
    };
  };
  
  # Module configuration
  config = mkIf cfg.enable {
    environment.variables = {
      ELECTRON_OZONE_PLATFORM_HINT = "wayland";
      NIXFMT_PATH = "${pkgs.nixfmt-rfc-style}/bin/nixfmt-rfc-style";
    };
    
    environment.systemPackages = with pkgs; [
      appimage-run
      poppler_utils
      nodejs_22
      nixfmt-rfc-style
      cursorPackage
      cursorWrapper
      cursorSettingsPackage
      checkUpdatesScript
      jq
    ];
    
    programs.appimage = {
      enable = true;
      binfmt = true;
    };
    
    # Create directory and link the settings file
    system.activationScripts.cursorSettings = {
      text = ''
        mkdir -p /etc/cursor
        ln -sf ${cursorSettingsPackage}/etc/cursor/settings.json /etc/cursor/settings.json
      '';
      deps = [];
    };
    
    # Optionally display a message about available updates during system rebuild
    system.activationScripts.cursorUpdateNotify = mkIf cfg.notifyUpdates {
      text = ''
        API_URL="https://www.cursor.com/api/download?platform=linux-x64&releaseTrack=stable"
        LATEST_VERSION=$(${pkgs.curl}/bin/curl -sL "$API_URL" | ${pkgs.jq}/bin/jq -r '.version' 2>/dev/null || echo "${cursorVersion}")
        
        if [ "${cursorVersion}" != "$LATEST_VERSION" ] && [ -n "$LATEST_VERSION" ]; then
          echo -e "\033[1;33mCursor update available: ${cursorVersion} -> $LATEST_VERSION\033[0m"
          echo -e "Run \033[1mcheck-cursor-updates\033[0m for update instructions."
        fi
      '';
      deps = ["cursorSettings"];
    };
  };
}
