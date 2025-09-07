{ lib, config, pkgs, ... }:

with lib;
let cfg = config.gtk;

in {
  config = mkIf cfg.enable {
    
    # use stylix for settings in gtk.
    gtk = {
    };
  };
}
