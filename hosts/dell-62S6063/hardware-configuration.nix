# Specific Hardware settings for a Dell Precision 7750 with ServiceTag 62S6063
# https://www.dell.com/support/home/en-us/product-support/servicetag/0-QngrYk01UVVKbWtRNEUzRU9FVVVUQT090/overview
{ config, lib, pkgs, modulesPath, ... }:

{
  imports =
    [
      (modulesPath + "/installer/scan/not-detected.nix")
      ./filesystem-byuuid.nix
    ];

  # Drivers
  nixpkgs.config.allowUnfree = true;
  hardware.enableAllFirmware = true;

  # Bluetooth
  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = true;

  # Game Pads
  #hardware.xone.enable = true;

  # Wireless (donggle) KB / Mice
  hardware.logitech.wireless.enable = true;
  hardware.logitech.wireless.enableGraphical = true;

  hardware.graphics = {
    enable = true;

    extraPackages = with pkgs; [

      # intel WebGPU graphics support
      # mesa
      # libdrm
      # intel-media-driver
      # intel-media-sdk
      # intel-compute-runtime
      # vulkan-headers
      # vulkan-loader
      # vulkan-validation-layers
      vulkan-tools # vulkaninfo
      # vulkan-tools-lunarg # vkconfig
      # vaapiIntel
      # vaapiVdpau
      # libvdpau-va-gl

      glfw
      freetype
      shaderc
      logiops
    ];
  };

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  powerManagement.cpuFreqGovernor = lib.mkDefault "performance";
  hardware.cpu.intel.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
}
