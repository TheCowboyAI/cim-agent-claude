{ config, pkgs, lib, ... }:

{
  #boot.loader.systemd-boot.enable = true;
  boot.supportedFilesystems = [ "ntfs" ]; # to detect windows better

  boot.kernelPackages = lib.mkDefault pkgs.linuxPackages_latest;

  boot.initrd = {
    kernelModules = lib.mkForce [
      "kvm-intel"
      "i915"
      "xhci_pci"
      "nvme"
      "usb_storage"
      "sd_mod"
      "rtsx_pci_sdmmc"
      "vfio_pci"
      "vfio"
      "vfio_iommu_type1"
      "mdev"
    ];
  };

  boot.extraModprobeConfig = lib.mkForce ''
    options i915 enable_guc=0 enable_gvt=1
  '';


  boot.kernelParams = lib.mkForce [
    "intel_iommu=on"
    "preempt=voluntary"
    "pcie_aspm=off"
    "iommu=pt"
    "pcie_acs_override=downstream,multifunction"
    "kvm.ignore_msrs=1"
  ];

  boot.blacklistedKernelModules = lib.mkForce [ "nouveau" ];

  # allow build arm images
  #  boot.binfmt.emulatedSystems = [ "aarch64-linux" ];

  virtualisation.libvirtd.enable = true;

  boot.loader = {
    efi.canTouchEfiVariables = true;
    grub.enable = lib.mkForce true; # We use this over systemd to dual boot.
    grub.devices = [ "nodev" ];
    grub.efiSupport = lib.mkForce true;
    grub.useOSProber = lib.mkDefault true;
  };

}
