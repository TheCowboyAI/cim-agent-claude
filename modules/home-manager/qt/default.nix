{ lib, config, ... }:

with lib;
let cfg = config.qt;

in {
  options.qt.enable = lib.mkEnableOption "Enable qt";
  config = mkIf cfg.enable {
    programs.qt = {
      enable = true;
      platformTheme.name = "gtk";
      style.name = "adwaita-gtk";
    };
  };
}
