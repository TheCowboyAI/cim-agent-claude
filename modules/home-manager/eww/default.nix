{ lib, config, pkgs, ... }:

with lib;
let cfg = config.eww;

in {
  options.eww.enable = lib.mkEnableOption "Enable eww";
  config = mkIf cfg.enable {
    # theres no programs.eww.enable here because eww looks for files in .config
    # thats why we have all the home.files

    # eww package
    home.packages = with pkgs; [
      eww
      just
      pamixer
      brightnessctl
      nerd-fonts.jetbrains-mono
    ];

    # configuration
    home.file.".config/eww/eww.scss".source = ./eww.scss;

    home.file.".config/eww/styles".source = ./styles;

    home.file.".config/eww/eww.yuck".source = ./eww.yuck;
    home.file.".config/eww/justfile" = {
      source = ./justfile;
      executable = false;
    };

    # scripts
    # home.file.".config/eww/scripts/battery.sh" = {
    #   source = ./scripts/battery.sh;
    #   executable = true;
    # };

    # systemd user service for eww
    systemd.user.services.eww = {
      Unit = {
        Description = "ElKowar's Wacky Widgets";
        PartOf = [ "graphical-session.target" ];
        After = [ "graphical-session.target" ];
        Requisite = [ "graphical-session.target" ];
      };
      Service = {
        ExecStart = "${pkgs.eww}/bin/eww daemon --no-daemonize";
        ExecStartPost = [
          "${pkgs.coreutils}/bin/sleep 1"  
          "${pkgs.eww}/bin/eww open bar"
        ];
        ExecStop = "${pkgs.eww}/bin/eww kill";
        Restart = "on-failure";
        RestartSec = "1";
        WorkingDirectory = "%h/.config/eww";
      };
      Install.WantedBy = [ "graphical-session.target" ];
    };

  };
}
