{ lib, config, ... }:

with lib;
let cfg = config.CHANGEME;

in {
  options.CHANGEME.enable = lib.mkEnableOption "Enable CHANGEME";
  config = mkIf cfg.enable {
    programs.CHANGEME = {
      enable = true;
    };
  };
}