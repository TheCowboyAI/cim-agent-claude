{ pkgs, ... }:

{
  # AUDIO
  services.pulseaudio.enable = false;

  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    audio.enable = true;
    wireplumber.enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
    jack.enable = true;
  };

  environment.systemPackages = with pkgs; [
    playerctl
    coppwr
    alsa-lib
    alsa-utils
    udev
    qjackctl
    helvum
    qastools
  ];
}
