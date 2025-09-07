{
  lib,
  config,
  pkgs,
  ...
}:
with lib; let
  cfg = config.mako;
in {
  options.mako = {
    enable = mkEnableOption "Enable mako notification daemon";
  };

  config = mkIf cfg.enable {
    services.mako = {
      enable = true;
      
      # Use settings instead of the deprecated extraConfig
      settings = {
        # Add your mako settings here as key/value pairs
        # For example:
        font = "monospace 10";
        width = 300;
        height = 100;
        margin = "10";
        padding = "15";
        border-size = 2;
        default-timeout = 5000;
      };
    };
  };
} 