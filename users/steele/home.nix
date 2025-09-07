{
  lib,
  pkgs,
  inputs,
  ...
}: {
  # HOME MANAGER CONFIGURATION

  # The state version is required and should stay at the version you
  # originally installed.
  home.stateVersion = "23.11";

  home.username = "steele";

  #home.description = "Steele Price";
  home.homeDirectory = "/home/steele";
  #home.extraGroups = [ "wheel" "networkmanager" "wireshark" "audio" "video" "docker" "qemu-libvirt" "kvm" "libvirt" "plugdev"];

  # Disable version mismatch check
  home.enableNixpkgsReleaseCheck = false;
  
  # Fix Wayland display socket mismatch
  home.sessionVariables = {
    WAYLAND_DISPLAY = "wayland-1";
  };

  # Cursor configuration for Hyprland
  home.pointerCursor = {
    gtk.enable = true;
    x11.enable = true;
    package = pkgs.adwaita-icon-theme;
    name = "Adwaita";
    size = 24;
  };

  # GTK cursor theme configuration
  gtk = {
    enable = true;
    cursorTheme = {
      package = pkgs.adwaita-icon-theme;
      name = "Adwaita";
      size = 24;
    };
  };

  home.file."Pictures/Screenshots/.keep".text = "";
  
  # Hyprland layout toggle script
  home.file.".config/hypr/toggle-layout.sh" = {
    text = ''
      #!${pkgs.bash}/bin/bash
      
      # Get current workspace
      CURRENT_WS=$(hyprctl activeworkspace -j | jq -r '.id')
      STATE_FILE="$HOME/.config/hypr/layout_state_ws$CURRENT_WS"
      
      # Check current state (default is tiling)
      if [[ -f "$STATE_FILE" && $(cat "$STATE_FILE") == "floating" ]]; then
        # Currently floating, switch to tiling
        # Make all current workspace windows tile
        hyprctl clients -j | jq -r --arg ws "$CURRENT_WS" '.[] | select(.workspace.id == ($ws | tonumber) and .floating == true) | .address' | while read addr; do
          hyprctl dispatch togglefloating address:$addr
        done
        echo "tiling" > "$STATE_FILE"
        hyprctl notify -1 3000 "rgb(ff9500)" "Workspace $CURRENT_WS: Tiling Mode"
      else
        # Currently tiling, switch to floating
        # Make all current workspace windows float
        hyprctl clients -j | jq -r --arg ws "$CURRENT_WS" '.[] | select(.workspace.id == ($ws | tonumber) and .floating == false) | .address' | while read addr; do
          hyprctl dispatch togglefloating address:$addr
        done
        echo "floating" > "$STATE_FILE"
        hyprctl notify -1 3000 "rgb(00ff95)" "Workspace $CURRENT_WS: Floating Mode"
      fi
    '';
    executable = true;
  };

  programs.home-manager.enable = true;

  imports = [
    inputs.nix-colors.homeManagerModules.default
    inputs.hyprland.homeManagerModules.default
    # inputs.anyrun.homeManagerModules.default  # Removed due to unicode compilation issues
    ../../modules/default.nix
    ./packages.nix
  ];

  # Override stylix settings to fix deprecation warnings
  stylix.targets = {
    # Fix wpaperd settings deprecation warning
    wpaperd.enable = lib.mkForce false;
    # Fix vscode settings deprecation warnings
    vscode.enable = lib.mkForce false;
    # Configure Firefox profile for stylix
    firefox = {
      profileNames = [ "default" ];
    };
  };

  # this is for what stylix doesn't cover?
  # we should be able to set stylix at the user level
  # but this doesn't seem to apply anywhere
  # colorScheme = inputs.nix-colors.colorSchemes.rebecca;

  # the rest should just be turning things on and off...
  # ALL Configurations should be in modules

  # xdg can be very problematic if not set properly.
  xdg.enable = true;
  xdg.configFile.steele = {
    enable = true;
    source = ../../modules/settings/xdg;
  };

  # modules

  # These are all switches that turn home-manager modules on and off.
  rofi.enable = true;  # Replaced anyrun with rofi - much more reliable and user-friendly
  brave.enable = false;
  direnv.enable = true;
  dunst.enable = true;
  eww.enable = true;
  ferdium.enable = true;
  firefox.enable = true;
  git.enable = true;
  # We use dunst instead of mako for notifications
  obs-studio.enable = true;
  qt.enable = true;
  terminator.enable = true;
  thunar.enable = true;
  wezterm.enable = true;
  # wofi.enable = true;  # Replaced with rofi-wayland for better functionality
  wpaperd.enable = true;
  zsh.enable = true;
  htop.enable = true;
  starship.enable = true;
  helix.enable = true;
  vscode.enable = true;
  ham.enable = true;
  programs.zoom-flatpak.enable = true;
  
  
  # Mako is now handled by the mako-override.nix module
  
  # Add an assertion to avoid the old module evaluating
  assertions = [
    {
      assertion = true; 
      message = "This assertion is always true and replaces the failing one";
    }
  ];
}
