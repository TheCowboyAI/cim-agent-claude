{ lib, config, pkgs, ... }:

with lib;
let cfg = config.waybar;

in {
  options.waybar.enable = lib.mkEnableOption "Enable waybar";
  config = mkIf cfg.enable {
    packages = with pkgs; [
      waybar
    ];
  };
}