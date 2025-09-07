{ config, lib, pkgs, ... }:

{
  # Override the gurk-rs module to fix the home.package error
  disabledModules = [ "programs/gurk-rs.nix" ];
  
  # Provide a dummy gurk-rs module that does nothing
  options.programs.gurk-rs = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Dummy gurk-rs option to override the broken module";
    };
  };
  
  config = lib.mkIf config.programs.gurk-rs.enable {
    # Do nothing - this is just to prevent the broken module from loading
  };
}