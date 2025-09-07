{ config, pkgs, ... }:

{
  # Enable CUPS for printing
  services.printing = {
    enable = true;
    drivers = [ pkgs.hplip ];
  };

  # Enable SANE for scanning
  hardware.sane = {
    enable = true;
    extraBackends = [ 
      pkgs.hplipWithPlugin 
      pkgs.sane-backends
    ];
    netConf = ''
      # HP LaserJet Pro MFP M227fdn
      10.0.0.243
    '';
    # Enable network scanning
    openFirewall = true;
  };
  
  # Additional HP services
  services.printing.extraConf = ''
    # Allow network scanning
    BrowsePoll 10.0.0.243
  '';

  # Enable printer discovery
  services.avahi = {
    enable = true;
    nssmdns4 = true;
    openFirewall = true;
  };

  # Configure the HP LaserJet printer
  hardware.printers = {
    ensurePrinters = [
      {
        name = "HP_LaserJet";
        location = "Office";
        description = "HP LaserJet Pro MFP M227fdn";
        deviceUri = "ipp://10.0.0.243/ipp/print";
        model = "HP/hp-laserjet_pro_mfp_m227-m231-ps.ppd.gz";
        ppdOptions = {
          PageSize = "Letter";
        };
      }
    ];
    ensureDefaultPrinter = "HP_LaserJet";
  };

  # Open firewall for printing services
  networking.firewall = {
    allowedTCPPorts = [ 631 ];
    allowedUDPPorts = [ 631 ];
  };
}