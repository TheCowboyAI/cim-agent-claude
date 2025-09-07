{ lib, config, pkgs, ... }:

with lib;

let
  cfg = config.services.warp-terminal;
in {
  options.services.warp-terminal = {
    enable = mkEnableOption "Enable Warp AI-powered terminal";

    package = mkOption {
      type = types.package;
      default = pkgs.warp-terminal;
      defaultText = literalExpression "pkgs.warp-terminal";
      description = "The Warp terminal package to use.";
    };

    defaultTerminal = mkOption {
      type = types.bool;
      default = false;
      description = "Set Warp as the default terminal emulator.";
    };

    autoStart = mkOption {
      type = types.bool;
      default = false;
      description = "Automatically start Warp on login.";
    };

    settings = mkOption {
      type = types.attrs;
      default = {};
      example = literalExpression ''
        {
          theme = "dark";
          font_size = 14;
          ai_enabled = true;
        }
      '';
      description = "Configuration settings for Warp terminal.";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # Set up Warp as default terminal if requested
    environment.sessionVariables = mkIf cfg.defaultTerminal {
      TERMINAL = "${cfg.package}/bin/warp-terminal";
    };

    # Create systemd user service for auto-start
    systemd.user.services.warp-terminal = mkIf cfg.autoStart {
      description = "Warp AI Terminal";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/warp-terminal";
        Restart = "on-failure";
        RestartSec = 5;
      };
    };

    # Set as default terminal in XDG mime types
    xdg.mime = mkIf cfg.defaultTerminal {
      enable = true;
      defaultApplications = mkMerge [
        {
          "x-scheme-handler/terminal" = "dev.warp.Warp.desktop";
        }
      ];
    };

    # Create Warp configuration directory and settings file if settings are provided
    environment.etc = mkIf (cfg.settings != {}) {
      "warp-terminal/settings.json".text = builtins.toJSON cfg.settings;
    };
  };
}