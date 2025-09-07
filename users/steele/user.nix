{ pkgs, ... }:
{
  # SYSTEM LEVEL USER SETTINGS THAT HOME-MANAGER CAN'T DO
  programs.zsh.enable = true;
  programs.starship.enable = true;
  users.groups.plugdev = {}; #for yubikey
  users.users.steele = {
    isNormalUser = true;
    description = "Steele Price";
    shell = pkgs.zsh;
    openssh.authorizedKeys.keys = [];
    extraGroups = [ "wheel" "networkmanager" "wireshark" "audio" "video" "docker" "qemu-libvirt" "kvm" "libvirt" "plugdev" "scanner" "lp"];
  };
}
