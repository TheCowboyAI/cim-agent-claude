{ lib, config, ... }:

with lib;
let cfg = config.direnv;

in {
  options.direnv.enable = lib.mkEnableOption "Enable direnv";
  config = mkIf cfg.enable {
    programs.direnv = {
      enable = true;
      nix-direnv.enable = true;
      enableZshIntegration = true;
    };
  };
}
