# CIM Claude Adapter - Nix Flake
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{
  description = "CIM Claude Adapter - Event-driven Claude AI integration for CIM ecosystems";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # CIM Claude Adapter Configuration
        # Hard-locked Anthropic API version for consistency
        anthropicApiVersion = "2023-06-01";

        # Simple Rust package using rustPlatform
        cim-claude-adapter = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cim-claude-adapter";
          version = "0.1.0";
          
          # Copy the entire source directory without filtering
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          # Build-time environment variables
          CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
          
          buildInputs = with pkgs; [
            openssl
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchain
          ];

          # Metadata
          meta = with pkgs.lib; {
            description = "Event-driven Claude AI adapter for CIM ecosystems";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;

          };
        };

        # Skip NixOS systems for now due to complexity
        # TODO: Re-enable when module integration is stable

      in {
        # Packages
        packages = {
          default = cim-claude-adapter;
          cim-claude-adapter = cim-claude-adapter;
          # TODO: Add container and test-system when module integration is stable
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          # Build inputs (same as the package)
          buildInputs = cim-claude-adapter.buildInputs;
          nativeBuildInputs = cim-claude-adapter.nativeBuildInputs;

          # Additional packages for development
          packages = with pkgs; [
            # Rust development tools
            rustToolchain
            rust-analyzer
            clippy
            rustfmt
            cargo-audit
            cargo-watch
            cargo-tarpaulin
            
            # NATS tools
            natscli
            
            # General development tools
            jq
            curl
            
            # Documentation tools
            mdbook
            
            # Nix tools
            nixpkgs-fmt
            nil
          ];

          # Environment variables for development
          CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
          
          shellHook = ''
            echo "🤖 CIM Claude Adapter Development Environment"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "📦 Rust toolchain: $(rustc --version)"
            echo "🔒 Anthropic API Version: ${anthropicApiVersion} (hard-locked)"
            echo "🔧 Available commands:"
            echo "  cargo build                 - Build the project"
            echo "  cargo test                  - Run tests"
            echo "  cargo run                   - Run the adapter"
            echo "  cargo watch -x run          - Auto-reload on changes"
            echo "  nix build                   - Build with Nix"
            echo "  nix build .#container       - Build NixOS container"
            echo ""
            echo "📚 Documentation:"
            echo "  docs/API.md                 - API reference"
            echo "  docs/USER_GUIDE.md          - User guide"
            echo "  docs/DESIGN.md              - Architecture docs"
            echo ""
            echo "🔍 Set environment variables:"
            echo "  export CLAUDE_API_KEY=your-api-key"
            echo "  export NATS_URL=nats://localhost:4222"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
          '';
        };

        # Apps
        apps.default = flake-utils.lib.mkApp {
          drv = cim-claude-adapter;
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Checks (for CI)
        checks = {
          # Build check
          build = cim-claude-adapter;
          
          # Clippy check
          clippy = pkgs.rustPlatform.buildRustPackage {
            pname = "cim-claude-adapter-clippy";
            version = "0.1.0";
            src = ./.;
            cargoLock = { lockFile = ./Cargo.lock; };
            CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
            buildInputs = cim-claude-adapter.buildInputs;
            nativeBuildInputs = cim-claude-adapter.nativeBuildInputs;
            buildPhase = ''
              cargo clippy --all-targets -- --deny warnings
            '';
            installPhase = ''
              mkdir -p $out
              echo "Clippy check passed" > $out/clippy-success
            '';
          };
          
          # Formatting check
          fmt = pkgs.stdenv.mkDerivation {
            pname = "cim-claude-adapter-fmt";
            version = "0.1.0";
            src = ./.;
            nativeBuildInputs = [ rustToolchain ];
            buildPhase = ''
              cargo fmt --check
            '';
            installPhase = ''
              mkdir -p $out
              echo "Format check passed" > $out/fmt-success
            '';
          };
          
          # Test check (basic - can be expanded)
          test = pkgs.stdenv.mkDerivation {
            pname = "cim-claude-adapter-test";
            version = "0.1.0";
            src = ./.;
            CIM_ANTHROPIC_API_VERSION = anthropicApiVersion;
            buildInputs = cim-claude-adapter.buildInputs;
            nativeBuildInputs = cim-claude-adapter.nativeBuildInputs;
            buildPhase = ''
              cargo test --lib
            '';
            installPhase = ''
              mkdir -p $out
              echo "Tests passed" > $out/test-success
            '';
          };
        };
      }
    ) // {
      # NixOS modules
      nixosModules.default = import ./nix/module.nix;
      nixosModules.cim-claude-adapter = import ./nix/module.nix;
      nixosModules.nats-infrastructure = import ./nix/nats-infrastructure.nix;
      nixosModules.test-integration = import ./nix/test-integration.nix;
      
      # Overlay for adding this package to nixpkgs
      overlays.default = final: prev: {
        cim-claude-adapter = self.packages.${final.system}.cim-claude-adapter;
      };
    };
}