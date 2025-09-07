{ lib, config, ... }:

with lib;
let cfg = config.terminator;

in {
  options.terminator.enable = lib.mkEnableOption "Enable terminator";
  config = mkIf cfg.enable {
    programs.terminator = {
      enable = true;
      config = {
        global_config.borderless = false;
        profiles.default = {
          scrollback_lines = 5000;
          show_titlebar = false;
        };
      };
    };
  };
}
