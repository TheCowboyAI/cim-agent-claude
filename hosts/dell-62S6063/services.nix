{ pkgs, lib, ... }:
{
  services = {
    blueman.enable = true;
    gnome.gnome-keyring.enable = true;
    dbus.packages = [ pkgs.gcr ];

    flatpak.enable = true;
    
    # SMB share for scanner
    samba = {
      enable = true;
      openFirewall = true;
      settings = {
        global = {
          security = "user";
        };
        scans = {
          path = "/home/steele/scans";
          browseable = "yes";
          "read only" = "no";
          "guest ok" = "yes";
          "create mask" = "0644";
          "directory mask" = "0755";
          "force user" = "steele";
          "force group" = "users";
        };
      };
    };
    yubikey.enable = true;

    warp-terminal = {
      enable = true;
      defaultTerminal = false; # Set to true if you want it as default
      autoStart = false; # Set to true to auto-start on login
      settings = {
        # Add any Warp-specific settings here
      };
    };

    # spacedrive = {
    #   enable = true;
    #   defaultFileManager = true;
    # };

    devmon.enable = true;
    gvfs.enable = true;
    udisks2.enable = true;
    
    nix-serve = {
      enable = true;
      port = 5000;
      bindAddress = "127.0.0.1";  # Only accessible locally by default
      secretKeyFile = "/var/sign/secret-key";
      openFirewall = false;
    };
    
    # special ham radio connector
    udev.extraRules = lib.mkForce ''
      SUBSYSTEM=="tty", ATTRS{idVendor}=="0403", ATTRS{idProduct}=="6015", GROUP="dialout", MODE="0660"
    '';

    udev.packages = with pkgs; [
      libinput
    ];

    # greetd with tuigreet for Hyprland
    greetd = {
      enable = lib.mkDefault true;
      settings = {
        default_session = {
          command = "${pkgs.tuigreet}/bin/tuigreet --time --remember --remember-session --cmd ${pkgs.hyprland}/bin/Hyprland";
          user = "greeter";
        };
      };
    };

    ollama = {
      enable = true;
      acceleration = false;
      openFirewall = true;
      loadModels = [
        "vicuna"
      ];
      host = "0.0.0.0";
      port = 11434;
      home = "/var/lib/ollama";
      environmentVariables = {
        OLLAMA_ORIGINS = "http://0.0.0.0:11434";
        OLLAMA_HOST = "0.0.0.0:11434";
        OLLAMA_MODELS = "/var/lib/ollama/models";
        OLLAMA_URL = "http://0.0.0.0:11434";
      };
    };

    nats = {
      enable = lib.mkDefault true;
      jetstream = true;
      settings = {
        jetstream = {
          store_dir = lib.mkForce "/var/lib/nats/jetstream";
        };
      };
    };

  };
  
}
