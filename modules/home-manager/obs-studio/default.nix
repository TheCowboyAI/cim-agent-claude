{ lib, config, ... }:

with lib;
let cfg = config.obs-studio;

in {
  options.obs-studio.enable = lib.mkEnableOption "Enable obs-studio";
  config = mkIf cfg.enable {
    programs.obs-studio = {
      enable = true;
    };
  };
}