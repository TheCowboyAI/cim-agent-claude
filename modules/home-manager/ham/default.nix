{ lib, config, pkgs, ... }:

with lib;
#  Ham Radio settings
let cfg = config.ham;

in {
  options.ham.enable = lib.mkEnableOption "Enable Ham Radio Options";
  config = mkIf cfg.enable {
    home.packages = with pkgs; [
      chirp
    ];

  };
}
