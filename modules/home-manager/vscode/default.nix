{ lib
, config
, pkgs
, ...
}:
with lib; let
  cfg = config.vscode;
  mkt = import ./vsextensions.nix;
  settings = builtins.readFile ./settings.json;
in
{
  options.vscode.enable = lib.mkEnableOption "Enable vscode";
  # if we enable it, use this...
  config = mkIf cfg.enable {
    programs.vscode = {
      enable = true;
      # Use profiles.default instead of the top-level settings
      profiles.default = {
        userSettings = builtins.fromJSON settings;
        extensions = with pkgs.vscode-extensions;
          pkgs.vscode-utils.extensionsFromVscodeMarketplace mkt.extensions;
      };
      # Ensure Alejandra extension is properly configured
      mutableExtensionsDir = false;
    };
    
    # Set environment variables for VSCode to find Alejandra
    home.sessionVariables = {
      ALEJANDRA_PATH = "${pkgs.alejandra}/bin/alejandra";
    };
  };
}
