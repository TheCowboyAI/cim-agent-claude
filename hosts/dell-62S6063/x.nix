{ pkgs, ... }:
{
  services.xserver = {
    enable = false;
  };

  # wayland.windowManager.hyprland = {
  #   enable = true;
  # };
  
  # Set Dolphin as default file manager
  xdg.mime = {
    enable = true;
    defaultApplications = {
      "inode/directory" = "org.kde.dolphin.desktop";
      "application/x-directory" = "org.kde.dolphin.desktop";
    };
  };
}