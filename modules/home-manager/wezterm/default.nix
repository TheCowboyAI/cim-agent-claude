{ lib, config, ... }:

with lib;
let cfg = config.wezterm;

in {
  options.wezterm.enable = lib.mkEnableOption "Enable wezterm";
  config = mkIf cfg.enable {
    programs.wezterm = {
      enable = true;
      enableZshIntegration = true;
      enableBashIntegration = true;
      extraConfig = builtins.readFile ./wezterm.lua;
    };
  };
}
