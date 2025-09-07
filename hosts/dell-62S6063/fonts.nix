{ config, lib, pkgs, ... }:
{
  fonts = {
    fontDir.enable = true;
    enableDefaultPackages = true;
    
    fontconfig = {
      enable = true;
      antialias = true;
      hinting.enable = true;
      hinting.autohint = true;
      defaultFonts = {
        serif = [
          "NotoSerif Nerd Font"
        ];
        sansSerif = [
          "NotoSans Nerd Font"
        ];
        monospace = [
          "BitstreamVeraSansMono"
          "Noto Sans"
        ];
      };
    };

    packages = with pkgs; [
      material-symbols
      material-design-icons
      material-icons
      noto-fonts-emoji
      nerd-fonts.bitstream-vera-sans-mono
      nerd-fonts.fira-code
      nerd-fonts.fira-mono
      nerd-fonts.noto
      nerd-fonts.jetbrains-mono
      nerd-fonts.go-mono
    ];
  };
}
