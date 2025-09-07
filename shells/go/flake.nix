{
  description = "A simple Go Shell";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ... }:
  let
    pkgs = import nixpkgs { system = "x86_64-linux"; };
  in
  {
    devShells."x86_64-linux".default = with pkgs; mkShell {
      packages = [
        go
        gopls
        gotools
        go-tools
        # Latest Node.js for Claude Code compatibility
        nodejs_22
      ];

      shellHook = ''
        echo "═════════════════════════════════════════════════"
        echo "🐹 Go Development Shell Activated"
        echo "═════════════════════════════════════════════════"
        echo "🐹 Go: $(go version)"
        echo "🟢 Node.js: $(node --version) (Claude Code compatible)"
        echo ""
        echo "💻 Claude Code Integration:"
        echo "   claude --resume    - Resume Claude session"
        echo "   claude --new       - Start new Claude session"
        echo "═════════════════════════════════════════════════"
      '';
    };
  };
}
