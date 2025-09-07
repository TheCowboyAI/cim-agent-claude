{ lib, config, pkgs, ... }:

with lib;
let cfg = config.ags;

in {
  options.ags.enable = lib.mkEnableOption "Enable ags";
  config = mkIf cfg.enable {

    #home.file.".config/ags/config.js" = ./config.js;

    programs.ags = {
      enable = true;

      # null or path, leave as null if you don't want hm to manage the config
      #configDir = .config/ags;

      # additional packages to add to gjs's runtime
      extraPackages = with pkgs; [
        gtksourceview
        webkitgtk_4_1
        accountsservice
        sassc
      ];
    };
  };
}
