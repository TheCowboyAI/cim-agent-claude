{ pkgs, ... }: {
  imports = [
    # Add fixes first
    ./home-manager/gurk-rs-fix.nix
    # Add the mako override first to ensure it takes precedence
    ./home-manager/mako-override.nix
    
    ./home-manager/rofi
    ./home-manager/brave
    ./home-manager/direnv
    ./home-manager/dunst
    ./home-manager/eww
    ./home-manager/ferdium
    ./home-manager/firefox
    ./home-manager/git
    ./home-manager/ham
    ./home-manager/helix
    ./home-manager/htop
    ./home-manager/hyprland
    # ./home-manager/mako  # Removed since we use dunst instead
    ./home-manager/obs-studio
    ./home-manager/starship
    ./home-manager/terminator
    ./home-manager/thunar
    ./home-manager/vscode
    ./home-manager/wezterm
    # ./home-manager/wofi  # Replaced with rofi-wayland
    ./home-manager/wpaperd
    ./home-manager/zsh
    ./home-manager/zoom
  ];
}
