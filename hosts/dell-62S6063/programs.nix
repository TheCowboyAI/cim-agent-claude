{ pkgs, lib, ... }:

let
  # Define paths to assets
  mountainWallpaper = ../../modules/nixos/stylix/wallpaper/mountain.jpg;
  
  # Theme colors
  tokyoDarkColors = {
    background = "#11121ddd";
    accent = "#a485dd";
    backgroundAlt = "#1b1c27";
  };
in
{
  programs = {
    hyprland.enable = true; #the real config is in home-manager
    mtr.enable = true;

    dconf.enable = true; #required for gnome-keyring
    seahorse.enable = true;
    regreet = {
      enable = false;
      settings = {
        background = {
          # Use the mountain scene image from the Nix store
          path = lib.mkForce "${mountainWallpaper}";
          fit = lib.mkForce "Fill";
        };
        GTK = {
          cursor_theme_name = lib.mkForce "Adwaita";
          icon_theme_name = "Adwaita";
          theme_name = lib.mkForce "Tokyonight-Dark";
        };
        commands = {
          reboot = ["systemctl" "reboot"];
          poweroff = ["systemctl" "poweroff"];
        };
        default_session = {
          command = "${pkgs.hyprland}/bin/Hyprland";
        };
        # Tokyo-dark inspired colors
        notifications = {
          border_width = 2;
          border_radius = 8;
          padding = 16;
          background_color = tokyoDarkColors.background;
          border_color = tokyoDarkColors.accent;
          progress_background_color = tokyoDarkColors.backgroundAlt;
          progress_foreground_color = tokyoDarkColors.accent;
          margin_top = 8;
          spacing = 8;
        };
      };
    };
  };
}
