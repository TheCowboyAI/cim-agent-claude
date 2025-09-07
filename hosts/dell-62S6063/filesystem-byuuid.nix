{
  fileSystems."/" =
    {
      device = "/dev/disk/by-uuid/c90a8e8c-a15b-4c85-8296-eda0300b0d6f";
      fsType = "ext4";
    };

  fileSystems."/boot" =
    {
      device = "/dev/disk/by-uuid/94B7-A883";
      fsType = "vfat";
    };

  fileSystems."/git" =
    {
      device = "/dev/disk/by-uuid/aec99a44-2a99-48aa-b894-503d0411018c";
      fsType = "ext4";
    };
    

  swapDevices =
    [{ device = "/dev/disk/by-uuid/7459c524-7dec-4f96-9340-b5f2cacd5177"; }];
}