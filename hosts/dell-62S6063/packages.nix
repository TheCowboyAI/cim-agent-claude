{pkgs, lib, ...}: {
  nixpkgs.config.packageOverrides = pkgs: {
    # we want unstable/security patches, this is used for firefox plugins
    nur =
      import
      (builtins.fetchTarball {
        url = "https://github.com/nix-community/NUR/archive/master.tar.gz";
        sha256 = "0nrqw58qcpn808mc3ai2i5pg9cf4spgsdna9wy88c8459zaw849v";
      })
      {
        inherit pkgs;
      };
  };
  environment.variables.HYPRLAND_TRACE = 1;
  environment.variables.AQ_TRACE = 1;
  environment.variables.BROWSER = "${pkgs.google-chrome}/bin/google-chrome-stable";

  environment.systemPackages = with pkgs; [
    usbutils
    bind
    waylandpp
    wayland-scanner
    wayland-utils
    wayland-protocols
    pass-wayland
    waypipe
    virtiofsd
    ripgrep
    
    # Scanning utilities
    simple-scan
    xsane
    gscan2pdf
    sane-airscan  # For network scanning support
    kdePackages.skanlite      # KDE scanning app
    paperwork     # Document management and scanning
    
    # File manager
    (pkgs.writeShellScriptBin "dolphin" ''
      export QT_SCALE_FACTOR=2
      export QT_AUTO_SCREEN_SCALE_FACTOR=0
      exec ${pkgs.kdePackages.dolphin}/bin/dolphin "$@"
    '')
    kdePackages.dolphin-plugins # Additional Dolphin functionality

    opencode

    # Remote desktop
    rustdesk  # RustDesk remote desktop client

    hyprland-workspaces
    hyprland-protocols
    xdg-desktop-portal-hyprland
    kdePackages.qtwayland

    hypridle
    hyprpaper
    hyprpicker
    hyprshot
    hyprlang
    mesa
    bluez
    upower
    gvfs
    dart-sass
    libgtop
    btop
    matugen
    swww
    grimblast
    gpu-screen-recorder
    plexamp

    wl-mirror
    cytoscape
    libdrm

    libossp_uuid
    ssh-to-age
    curl
    micro # beats nano...
    wget
    zip
    unzip
    rclone
    dmidecode #to inspect hardware

    jc # to format it in json
    jq
    nmap
    dig
    alejandra
    nixos-anywhere
    virt-manager
    inetutils
    wireshark
    fastfetch
    starship #should be a module
    natscli
    nsc
    nats-top
    openssl

    libnotify
    wmctrl
    socat
    remmina
    gpu-screen-recorder-gtk

    spice
    spice-gtk
    spice-protocol
    virt-manager
    virt-viewer
    libinput
    libinput-gestures

    # browsers
    google-chrome
  ];
}
