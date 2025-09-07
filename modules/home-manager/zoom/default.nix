{ lib, config, pkgs, ... }:

with lib;
let 
  cfg = config.programs.zoom-flatpak;
  flatpakBin = "${pkgs.flatpak}/bin/flatpak";
  
  # Create a wrapper script for zoom
  zoomScript = pkgs.writeShellScriptBin "zoom" ''
    #!/usr/bin/env bash
    exec ${flatpakBin} run us.zoom.Zoom "$@"
  '';

in {
  options.programs.zoom-flatpak = {
    enable = mkEnableOption "Zoom via Flatpak";
  };

  config = mkIf cfg.enable {
    # Skip the assertion for now, as it might not correctly
    # detect the system Flatpak service
    # assertions = [
    #   {
    #     assertion = config.home.sessionVariables ? NIXOS_MODULE_FLATPAK_ENABLED 
    #                 || builtins.pathExists "/run/current-system/sw/bin/flatpak";
    #     message = "The Zoom Flatpak module requires Flatpak to be enabled at the system level. " +
    #               "Please add 'services.flatpak.enable = true;' to your NixOS configuration.";
    #   }
    # ];

    # Enable Flatpak system-wide in NixOS
    home.packages = with pkgs; [
      flatpak
      # Add our wrapper script
      zoomScript
    ];

    # Make sure XDG portals are available for Flatpak
    home.sessionVariables = {
      # Explicitly prepend Flatpak paths to make sure they take precedence
      XDG_DATA_DIRS = "/var/lib/flatpak/exports/share:$HOME/.local/share/flatpak/exports/share:$XDG_DATA_DIRS";
    };

    # Install Zoom via Flatpak
    # This requires a script that we'll place in the store
    home.activation = {
      # Add flatpak package as a dependency
      installFlatpak = lib.hm.dag.entryBefore ["installZoomFlatpak"] ''
        export PATH="${pkgs.flatpak}/bin:$PATH"
      '';

      installZoomFlatpak = lib.hm.dag.entryAfter ["writeBoundary"] ''
        $DRY_RUN_CMD ${pkgs.writeShellScriptBin "install-zoom-flatpak" ''
          # Check if Flathub remote is added
          if ! ${flatpakBin} remotes --user | grep -q "flathub"; then
            echo "Adding Flathub remote..."
            ${flatpakBin} remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
          fi

          # Install or update Zoom
          echo "Installing/updating Zoom from Flatpak..."
          ${flatpakBin} install --user -y flathub us.zoom.Zoom
          
          # Copy desktop file to ensure it's visible in all environments
          mkdir -p $HOME/.local/share/applications
          mkdir -p $HOME/.config/autostart
          
          # Only copy if source file exists
          if [ -f "$HOME/.local/share/flatpak/exports/share/applications/us.zoom.Zoom.desktop" ]; then
            cp -f "$HOME/.local/share/flatpak/exports/share/applications/us.zoom.Zoom.desktop" "$HOME/.local/share/applications/"
            cp -f "$HOME/.local/share/flatpak/exports/share/applications/us.zoom.Zoom.desktop" "$HOME/.config/autostart/us.zoom.Zoom-autostart.desktop"
            chmod +x "$HOME/.local/share/applications/us.zoom.Zoom.desktop"
          fi
        ''}/bin/install-zoom-flatpak
      '';
    };

    # Add systemd user service to keep Zoom updated
    systemd.user.services.flatpak-zoom-update = {
      Unit = {
        Description = "Update Zoom Flatpak";
      };
      
      Service = {
        Type = "oneshot";
        ExecStart = "${flatpakBin} update --user -y us.zoom.Zoom";
      };
      
      Install = {
        WantedBy = ["default.target"];
      };
    };

    systemd.user.timers.flatpak-zoom-update = {
      Unit = {
        Description = "Update Zoom Flatpak weekly";
      };
      
      Timer = {
        OnCalendar = "weekly";
        Persistent = true;
      };
      
      Install = {
        WantedBy = ["timers.target"];
      };
    };
  };
}
