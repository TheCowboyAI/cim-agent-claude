{
  config,
  pkgs,
  lib,
  ...
}:

with lib;

let
  cfg = config.services.yubikey;
in
{
  options.services.yubikey = {
    enable = mkEnableOption "Yubikey integration and development tools";

    pam = {
      enable = mkEnableOption "Yubikey PAM authentication support";
      
      u2f = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable U2F PAM authentication";
        };
        cue = mkOption {
          type = types.bool;
          default = true;
          description = "Whether to require the user to touch the device when using pam-u2f";
        };
      };
      
      challenge-response = {
        enable = mkOption {
          type = types.bool;
          default = true;
          description = "Enable challenge-response PAM authentication";
        };
      };
    };

    gpg = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable GPG/PGP support for Yubikey";
      };
      
      autostart = mkOption {
        type = types.bool;
        default = true;
        description = "Auto-start GPG agent for Yubikey";
      };
    };

    ssh = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable SSH with Yubikey";
      };
    };

    u2f = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable U2F/FIDO2 support for Yubikey";
      };
    };

    oath = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable OATH/TOTP support for Yubikey";
      };
    };

    pcsc = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable PC/SC smart card interface for Yubikey";
      };
    };

    development = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable development tools for Yubikey";
      };
    };
  };

  config = mkIf cfg.enable {
    # Core USB packages and tools for Yubikey
    services.udev.packages = with pkgs; [
      yubikey-personalization
      libu2f-host
    ];

    # PAM authentication configuration
    security.pam.u2f = mkIf (cfg.pam.enable && cfg.pam.u2f.enable) {
      enable = true;
        settings = {
          cue = cfg.pam.u2f.cue;
          # This allows all users on the system to use U2F
          authFile = "/etc/u2f_mappings";
        };
    };

    security.pam.yubico = mkIf (cfg.pam.enable && cfg.pam.challenge-response.enable) {
      enable = true;
      mode = "challenge-response";
      # Default mode is client which matches Yubikeys against an online service
      # challenge-response works offline
    };

    # GPG/PGP support
    programs.gnupg.agent = mkIf (cfg.gpg.enable) {
      enable = true;
      enableSSHSupport = cfg.ssh.enable;
      enableExtraSocket = true;
      pinentryPackage = pkgs.pinentry-curses;
    };

    # PC/SC Smart Card support
    services.pcscd = mkIf cfg.pcsc.enable {
      enable = true;
      plugins = [ pkgs.ccid ];
    };

    # System packages
    environment.systemPackages = with pkgs; [
      # Base Yubikey tools
      yubikey-manager
      yubikey-personalization
      
      # GPG tools if enabled
      (mkIf cfg.gpg.enable gnupg)
      (mkIf cfg.gpg.enable pinentry)
      
      # OATH/TOTP support
      (mkIf cfg.oath.enable oath-toolkit)
      (mkIf cfg.oath.enable yubioath-flutter)
      
      # U2F/FIDO2 support
      (mkIf cfg.u2f.enable pam_u2f)
      (mkIf cfg.u2f.enable libu2f-host)
      
      # SSH tools
      (mkIf cfg.ssh.enable openssh)
      
      # Development tools if enabled
      (mkIf cfg.development.enable yubico-piv-tool)
      (mkIf cfg.development.enable opensc)
      (mkIf cfg.development.enable pcsctools)
      (mkIf cfg.development.enable libfido2)
      (mkIf cfg.development.enable usbutils)
    ];

    services.udev.extraRules = ''
      # Yubikey udev rules
      ACTION!="add|change", GOTO="yubico_end"

      # Yubico YubiKey
      KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1050", ATTRS{idProduct}=="0113|0114|0115|0116|0120|0121|0200|0402|0403|0406|0407|0410", TAG+="uaccess"

      # FIDO2 / U2F
      KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="1050", ATTRS{idProduct}=="0113|0114|0115|0116|0120|0200|0402|0403|0406|0407|0410", TAG+="uaccess"
      KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2581", ATTRS{idProduct}=="f1d0", TAG+="uaccess"
      KERNEL=="hidraw*", SUBSYSTEM=="hidraw", ATTRS{idVendor}=="2c97", ATTRS{idProduct}=="0000|0001", TAG+="uaccess"

      LABEL="yubico_end"
    '';
    
    # Instructions for the user
    environment.variables = {
      YUBIKEY_ENABLED = mkIf cfg.enable "1";
    };
    
    # Optional: autostart GPG agent and SSH agent
    systemd.user.services.gpg-agent = mkIf (cfg.gpg.enable && cfg.gpg.autostart) {
      description = "GnuPG cryptographic agent and passphrase cache";
      wantedBy = [ "default.target" ];
      serviceConfig = {
        ExecStart = lib.mkDefault "${pkgs.gnupg}/bin/gpg-agent --daemon";
        Type = "forking";
        Restart = "on-abort";
      };
    };
  };
}
