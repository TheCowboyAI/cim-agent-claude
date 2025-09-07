{ lib, config, pkgs, ... }:

with lib;
let cfg = config.thunar;

in {
  options.thunar.enable = lib.mkEnableOption "Enable thunar";
  config = mkIf cfg.enable {
    home.packages = with pkgs; [
        xfce.thunar
    ];
  };
}
