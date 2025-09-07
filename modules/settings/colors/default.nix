{ pkgs, ... }:
{
  environment.etc."/nix-colors/dracula.yaml".source = ./dracula/dracula.yaml;
  environment.etc."/nix-colors/mountain.yaml".source = ./mountain/mountain.yaml;
  environment.etc."/nix-colors/tokyodark.yaml".source = ./tokyodark/tokyodark.yaml;

}