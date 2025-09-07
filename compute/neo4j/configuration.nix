{ pkgs, ... }:
let
  apoc-ext = pkgs.fetchurl {
    url = "https://github.com/neo4j-contrib/neo4j-apoc-procedures/releases/download/5.21.0/apoc-5.21.0-extended.jar";
    sha256 = "sha256-Zir/V868F7t4B4QmN/p+ivZaKXP6P5hnT3rwpHI1w+o=";
  };
in
{
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  services.qemuGuest.enable = true;

  networking.hostName = "neodev";
  networking.firewall.allowedTCPPorts = [ 22 7474 7473 7687 2004 5005 ];

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
        memorySize = 8192;
        cores = 4;
        graphics = true;
        diskSize = 16384;
      };
    };
  };

  services = {
    openssh = {
      enable = true;
      settings.PasswordAuthentication = true;
    };

    neo4j = {
      enable = true;
      bolt.enable = true;
      http.enable = true;
      https.enable = false;
      shell.enable = true;
      bolt.tlsLevel = "DISABLED";
      extraServerConfig = ''
        dbms.security.procedures.unrestricted=apoc.*
        dbms.security.procedures.allowlist=apoc.*
      '';
    };
  };

  system.activationScripts = {
    intialize-dbms = {
      # link the apoc library into usable space
      text = ''
        ln -s ${apoc-ext} /var/lib/neo4j/plugins/apoc-5.21.0-extended.jar
      '';
      deps = [ ];
    };
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
    pkg-config
    zlib.dev
    curl
    waypipe
    socat
  ];

  environment.variables.NEO4J_CONF = "/var/lib/neo4j/conf/";

  system.stateVersion = "24.11";
}
