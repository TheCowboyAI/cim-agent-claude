{ lib, config, pkgs, ... }:

with lib;

let
  cfg = config.services.claude-code;
  
  # Override the nixpkgs claude-code to use the latest version
  claude-code-latest = pkgs.claude-code.overrideAttrs (oldAttrs: rec {
    version = "1.0.84";
    src = pkgs.fetchurl {
      url = "https://registry.npmjs.org/@anthropic-ai/claude-code/-/claude-code-${version}.tgz";
      sha256 = "sha256-m4yrbnak2Et48CkJmOKZ7zfvi4j+WL+ZMeTTx5rERR0=";
    };
  });
  
  # Create a wrapper script that sets the API key
  claudeWrapper = pkgs.writeShellScriptBin "claude" ''
    ${lib.optionalString (cfg.apiKeyFile != null) ''
      if [ -f "${cfg.apiKeyFile}" ]; then
        export ANTHROPIC_API_KEY=$(cat "${cfg.apiKeyFile}")
      fi
    ''}
    exec ${cfg.package}/bin/claude "$@"
  '';
  
in {
  options.services.claude-code = {
    enable = mkEnableOption "Enable Claude Code service";

    apiKeyFile = mkOption {
      type = types.nullOr types.path;
      description = "Path to the file containing the API key for Claude Code.";
      default = null;
    };

    package = mkOption {
      type = types.package;
      default = claude-code-latest;
      defaultText = literalExpression "claude-code-latest";
      description = "The claude-code package to use.";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ 
      (if cfg.apiKeyFile != null then claudeWrapper else cfg.package)
    ];
  };
} 