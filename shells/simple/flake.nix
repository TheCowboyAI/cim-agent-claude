{
  description = "A simple Nix development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          # Basic development tools
          git
          curl
          wget
          
          # Latest Node.js for Claude Code compatibility
          nodejs_22
          
          # NATS tools
          natscli
          nsc
          nats-top
          
          # Browser
          google-chrome
          
          # Add any other tools you need here
        ];
        
        shellHook = ''
          echo "═════════════════════════════════════════════════"
          echo "🚀 Simple Development Shell Activated"
          echo "═════════════════════════════════════════════════"
          echo "🟢 Node.js: $(node --version) (Claude Code compatible)"
          echo "🌐 NATS tools available: natscli, nsc, nats-top"
          echo ""
          echo "💻 Claude Code Integration:"
          echo "   claude --resume    - Resume Claude session"
          echo "   claude --new       - Start new Claude session" 
          echo "═════════════════════════════════════════════════"
        '';
      };
    };
}
