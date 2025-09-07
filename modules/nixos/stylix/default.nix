# See: https://stylix.danth.me/styling.html
#      https://github.com/tinted-theming/schemes

{ lib, config, pkgs, ... }:


with lib;
{
  config = {
    # don't set font here, just colors and images.
    # we use xdg alone for fonts

    # add theme packages... for a list see: https://search.nixos.org/packages?channel=unstable&show=tokyonight-gtk-theme&from=0&size=200&sort=relevance&type=packages&query=-gtk-theme
    environment.systemPackages = with pkgs; [
        sassc
        gtk-engine-murrine
        gnome-themes-extra
        tokyonight-gtk-theme
        whitesur-gtk-theme
        dracula-theme
    ];

    stylix = {
      enable = true;
      autoEnable = true;
      image = ../../settings/colors/tokyodark/cowboyai-tokyodark-bg.png; 
      polarity = "dark";
      base16Scheme = ../../settings/colors/tokyodark/tokyodark.yaml;
    };
  };
}

