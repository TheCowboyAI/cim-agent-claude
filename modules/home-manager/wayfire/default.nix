{ lib, config, ... }:

with lib;
let cfg = config.wayfire;

in {
  options.wayfire.enable = lib.mkEnableOption "Enable wayfire";
  config = mkIf cfg.enable {
    programs.wayfire = {
      enable = true;
      plugins = [
        "wcm"
        "wf-shell"
        "wayfire-plugin_dbus_interface"
        "wayfire-plugins-extra"
      ];
    };
  };
}