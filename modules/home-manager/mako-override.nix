{
  lib,
  config,
  pkgs,
  ...
}: {
  # Override and disable the mako module completely
  disabledModules = [ "services/mako.nix" ];
  
  # Re-declare the mako options we need
  options.services.mako = lib.mkOption {
    type = lib.types.submodule {
      options = {
        enable = lib.mkEnableOption "mako notification daemon";
        settings = lib.mkOption {
          type = lib.types.attrs;
          default = {};
          description = "Settings for mako";
        };
      };
    };
    default = {};
    description = "Mako notification daemon service";
  };

  # Set the configuration
  config.services.mako = {
    enable = false;
    settings = {};
  };
} 