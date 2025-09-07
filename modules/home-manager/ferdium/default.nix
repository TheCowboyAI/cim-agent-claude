{ lib, config, pkgs, ... }:

with lib;
let cfg = config.ferdium;

in {
  options.ferdium.enable = lib.mkEnableOption "Enable ferdium";
  config = mkIf cfg.enable {
    home.packages = with pkgs; [
        ferdium
    ];

  };
}
