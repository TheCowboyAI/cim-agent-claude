{ pkgs
, ...
}: {
  # https://github.com/luong-komorebi/Awesome-Linux-Software
  home.packages = with pkgs; [
    # stuff
    killall
    act
    udisks
    udiskie
    testdisk-qt
    gnome-keyring
    lssecret
    compose2nix
    direnv
    natscli
    nsc
    nats-top
    gh

    # graphic apps
    webcord
    discord-ptb
    mpv
    
    spotify
    inkscape
    telegram-desktop
    pinta
    tuba
    vym
    google-chrome
    element-desktop
    fractal
    beeper
    whatsapp-for-linux
    lxqt.qps
    freetube
    sabnzbd

    # audio
    coppwr
    alsa-lib
    alsa-utils
    udev
    qjackctl
    helvum

    # language servers
    #awk-language-server
    bash-language-server
    #cmake-language-server
    #dockerfile-language-server-nodejs
    #docker-compose-language-service
    #helm-ls
    #lua-language-server
    #nginx-language-server
    #tailwindcss-language-server
    #typescript-language-server
    vscode-langservers-extracted
    yaml-language-server
    #omnisharp-roslyn
    nil
    alejandra
    nix-inspect
    nixfmt-rfc-style
    nixpkgs-fmt
    nixd
    rust-analyzer
    mold
    lldb
    marksman
    gopls

    zstd
    bat
    eza
    # wofi  # Replaced with rofi-wayland
    # wofi-emoji  # Using rofi instead
    caprine-bin
    hexchat
    krusader
    kdePackages.polkit-kde-agent-1
    cliphist
    wl-clipboard
    wayland-utils
    paleta
    slurp
    grim
    xdg-utils
    zathura

    #hyprland
    hypridle
    hyprpaper
    hyprpicker
    hyprshot

    glfw
    freetype
    vulkan-headers
    vulkan-loader
    vulkan-validation-layers
    vulkan-tools # vulkaninfo
    vulkan-tools-lunarg # vkconfig
    shaderc # GLSL to SPIRV compiler - glslc
    renderdoc # Graphics debugger
    tracy

    lld
    clang
    # gcc  # Commented out to avoid collision with clang
    just
    starship
    openssl
    openssl.dev
    xorg.libX11
    xorg.libXi
    xorg.libXcursor

  ];
}
