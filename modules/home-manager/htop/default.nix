{ lib, config, ... }:

with lib;
let cfg = config.htop;

in {
  options.htop.enable = lib.mkEnableOption "Enable htop";
  config = mkIf cfg.enable {
    programs.htop = {
      enable = true;
    };
  };
}
