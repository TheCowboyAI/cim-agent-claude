{ lib, config, pkgs, ... }:

with lib;
let cfg = config.brave;

in {
  options.brave.enable = lib.mkEnableOption "Enable brave";
  config = mkIf cfg.enable {
    programs.chromium = {
      enable = true;
      package = pkgs.brave;
    };
  };
}
