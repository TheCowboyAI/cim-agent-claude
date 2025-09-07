{
  services.qemuGuest.enable = true;
  virtualisation.docker.enable = false;
  virtualisation.libvirtd.enable = true;
  virtualisation.waydroid.enable = false;
  virtualisation.containers.enable = true;

  containers = {
    # browse = {
    #   autoStart = true;
    #   privateNetwork = true;
    #   config = import ../../compute/firefox/configuration.nix;
    # };

    # steele-dev = {
    #   autoStart = false;
    #   privateNetwork = true;
    #   config = import ../../compute/steele-dev/configuration.nix;
    # };

    # blender = {
    #   autoStart = false;
    #   privateNetwork = true;
    #   config = import ../../compute/blender/configuration.nix;
    # };

    neo4j = {
      autoStart = true;
      privateNetwork = false;
      config = import ../../compute/neo4j/configuration.nix;
      bindMounts = {
        "/var/lib/neo4j/data" = {
          hostPath = "/home/steele/neo4j/data";
          isReadOnly = false;
        };
        "/var/lib/neo4j/conf" = {
          hostPath = "/home/steele/neo4j/conf";
          isReadOnly = false;
        };
        "/var/lib/neo4j/import" = {
          hostPath = "/home/steele/neo4j/import";
          isReadOnly = false;
        };
        "/var/lib/neo4j/plugins" = {
          hostPath = "/home/steele/neo4j/plugins";
          isReadOnly = false;
        };
      };
    };
  };


}
