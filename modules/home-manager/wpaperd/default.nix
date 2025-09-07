{ lib, config, ... }:

with lib;
let cfg = config.wpaperd;

in {
  options.wpaperd.enable = lib.mkEnableOption "Enable wpaperd";
  config = mkIf cfg.enable {
    services.wpaperd = {
      enable = true;
      settings = {
        default = {
          path = "~/.config/wallpapers";
          sorting = "random";
        };
      };
    };
  };
} 