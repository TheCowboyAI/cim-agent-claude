{ inputs, config, agenix, sops-nix, ... }:
{
  inputs = [
    agenix.nixosModules.default
    sops-nix.nixosModules.sops
  ];

  sops = {
    defaultSopsFile = ../../.secrets/secrets.yaml;
    validateSopsFiles = false;
  };

  age = {
    sshKeyPaths = [ "~/.ssh/id_thecowboy_ai" ];
    keyFile = "/var/lib/sops-nix/key.txt";
    generateKey = true;
  };
}