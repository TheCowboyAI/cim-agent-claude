{ lib, config, ... }:

with lib;
let cfg = config.git;

in {
  options.git.enable = lib.mkEnableOption "Enable git";
  config = mkIf cfg.enable {
    programs.git = {
      enable = true;
      userName = "Steele";
      userEmail = "steele@thecowboy.ai";
      extraConfig = {
        init = { defaultBranch = "main"; };
      };
    };
  };
}
