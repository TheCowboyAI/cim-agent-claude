{ lib, config, ... }:

with lib;
let cfg = config.sway;

in {
  options.sway.enable = lib.mkEnableOption "Enable sway";
  config = mkIf cfg.enable {
    home.packages = with pkgs; [
      sway
      swaybg
      swayr
      swayws
      swayosd
      swaymux
      swayimg
      swaylock
      swayidle
      swaytools
      swaysettings
      sway-contrib.inactive-windows-transparency
      
      pamixer
      brightnessctl
      nerd-fonts.jetbrains-mono
    ];

    sway = {
      enable = true;
    };
  };
}