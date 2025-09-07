{ pkgs, ... }:
{

  imports = [
    ../../modules/home-manager/firefox
  ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  services.qemuGuest.enable = true;

  networking.hostName = "browse";
  networking.firewall.allowedTCPPorts = [ 22 4222 443 80 ];

  security.sudo.wheelNeedsPassword = false;
  security.polkit.enable = true;

  users.mutableUsers = true;
  users.users.steele = {
    isNormalUser = true;
    hashedPassword = "$y$j9T$Ke4uxBFRgEh9dHYzq/pZR0$jndNVu/AZxuP8jsYP6xZCrBGyvk.BhjclQUJIzDr9i1";
    extraGroups = [ "wheel" "networkmanager" ]; # Enable ‘sudo’ for the user.
    useDefaultShell = false;
    shell = pkgs.zsh;
    openssh.authorizedKeys.keys = [ "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDgGW4Y7S8YO3Se/1AK1ZuIaAtxa+sakK4SBv/nixRyJ cim@thecowboy.ai" ];
  };

  users.users.cim = {
    isNormalUser = true;
    initialPassword = "cim";
    extraGroups = [ "wheel" "networkmanager" ]; # Enable ‘sudo’ for the user.
    useDefaultShell = false;
    shell = pkgs.zsh;
    openssh.authorizedKeys.keys = [ "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDgGW4Y7S8YO3Se/1AK1ZuIaAtxa+sakK4SBv/nixRyJ cim@thecowboy.ai" ];
  };

  virtualisation = {
    vmVariant = {
      virtualisation = {
        memorySize = 16384;
        cores = 4;
        graphics = true;
        diskSize = 32768;
      };
    };
  };

  environment.variables = {
    MOZ_ENABLE_WAYLAND = "1";
    WAYLAND_DISPLAY = "1";
    ANKI_WAYLAND = "1";
    WINIT_UNIX_BACKEND = "wayland";
  };

  firefox.enable = true;

  services.openssh = {
    enable = true;
    settings.PasswordAuthentication = true;
  };

  programs = {
    zsh.enable = true;
    direnv.enable = true;
    starship.enable = true;
    git.enable = true;
  };

  environment.systemPackages = with pkgs; [
    htop
    just
    cacert
    openssl
    openssl.dev
    gnupg
    pkg-config
    zlib.dev
    curl
    natscli
    waypipe
    socat
  ];

  system.stateVersion = "24.05";
}
