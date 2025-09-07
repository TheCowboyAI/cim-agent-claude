{ lib, config, pkgs, ... }:

with lib;

let
  cfg = config.services.spacedrive;
  
  # Wrapper to ensure SSL certificates are available
  spacedriveWrapped = pkgs.writeShellScriptBin "spacedrive" ''
    export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
    export SSL_CERT_DIR="${pkgs.cacert}/etc/ssl/certs"
    exec ${cfg.package}/bin/spacedrive "$@"
  '';
in {
  options.services.spacedrive = {
    enable = mkEnableOption "Enable Spacedrive file explorer";

    package = mkOption {
      type = types.package;
      default = pkgs.spacedrive;
      defaultText = literalExpression "pkgs.spacedrive";
      description = "The Spacedrive package to use.";
    };

    autoStart = mkOption {
      type = types.bool;
      default = false;
      description = "Whether to automatically start Spacedrive on login.";
    };

    defaultFileManager = mkOption {
      type = types.bool;
      default = true;
      description = "Set Spacedrive as the default file manager.";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ 
      spacedriveWrapped
      pkgs.cacert
    ];

    systemd.user.services.spacedrive = mkIf cfg.autoStart {
      description = "Spacedrive file explorer";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];
      environment = {
        SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
        SSL_CERT_DIR = "${pkgs.cacert}/etc/ssl/certs";
      };
      serviceConfig = {
        ExecStart = "${spacedriveWrapped}/bin/spacedrive";
        Restart = "on-failure";
        RestartSec = 5;
      };
    };

    # Set Spacedrive as default file manager
    xdg.mime = mkIf cfg.defaultFileManager {
      enable = true;
      defaultApplications = {
        "inode/directory" = "spacedrive.desktop";
        "application/x-directory" = "spacedrive.desktop";
      };
    };

    # Ensure the desktop file is available
    environment.pathsToLink = mkIf cfg.defaultFileManager [ "/share/applications" ];
  };
}