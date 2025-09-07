{
  imports = [
    ./configuration.nix
    ./hardware-configuration.nix
    ./bootloader.nix
    #./sops.nix
    ./security.nix
    ./network.nix
    ./audio.nix
    ./fonts.nix
    ./packages.nix
    ./programs.nix
    ./services.nix
    ./virtualization.nix
    ./x.nix
    ./printing.nix
    ../../modules/nixos/code-cursor
    ../../modules/nixos/claude-code
    # ../../modules/nixos/cim-sage-local  # Disabled - module has issues
    ../../modules/nixos/utensils-mcp-nixos  # Utensils mcp-nixos replacement
    ../../modules/nixos/arxiv-mcp-cowboy  # TheCowboyAI fork
    ../../modules/nixos/playwright-mcp
    ../../modules/nixos/mcp-nats
    # ../../modules/nixos/research-mcp  # Disabled - HTTP connection issues
    # ../../modules/nixos/mcp-nixos-server  # Disabled - namespace issues
    # ../../modules/nixos/filesystem-mcp  # Temporarily disabled - path issue
    ../../modules/nixos/spacedrive
    ../../modules/nixos/warp-terminal
    ./cursor-fix.nix
    ./yubikey.nix
    ../../users/steele/user.nix
  ];
}
