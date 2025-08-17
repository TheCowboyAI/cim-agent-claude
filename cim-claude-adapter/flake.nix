# CIM Claude Adapter - Nix Flake
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{
  description = "CIM Claude Adapter - Event-driven Claude AI integration for CIM ecosystems";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane }:
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

        # Crane library for building Rust projects
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Common arguments for crane
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        commonArgs = {
          inherit src;
          strictDeps = true;
          
          buildInputs = with pkgs; [
            # Runtime dependencies
            openssl
            pkg-config
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # macOS-specific dependencies
            pkgs.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        # Build just the cargo dependencies (this is cached)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate
        cim-claude-adapter = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          
          # Additional build-time environment variables
          CARGO_BUILD_INCREMENTAL = "false";
          CARGO_BUILD_JOBS = "$(nproc)";
          
          # Metadata
          meta = with pkgs.lib; {
            description = "Event-driven Claude AI adapter for CIM ecosystems";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <hello@cowboy-ai.com>" ];
            platforms = platforms.unix;
          };
        });

        # NixOS container
        container = pkgs.nixosSystem {
          system = system;
          modules = [
            ({ config, pkgs, ... }: {
              boot.isContainer = true;
              networking.useDHCP = false;
              
              # Include our NixOS module
              imports = [ ./nix/module.nix ];
              
              # Enable the service with minimal config
              services.cim-claude-adapter = {
                enable = true;
                package = cim-claude-adapter;
                claude.apiKey = "/run/secrets/claude-api-key"; # Placeholder
                openFirewall = true;
              };
              
              # Container networking
              networking.firewall.allowedTCPPorts = [ 8080 9090 ];
              
              # Minimal system configuration
              systemd.services.systemd-logind.enable = false;
              systemd.services.getty@.enable = false;
              systemd.services."serial-getty@".enable = false;
              
              system.stateVersion = "24.05";
            })
          ];
        }.config.system.build.toplevel;

      in {
        # Packages
        packages = {
          default = cim-claude-adapter;
          cim-claude-adapter = cim-claude-adapter;
          container = container;
        };

        # Development shell
        devShells.default = craneLib.devShell {
          # Inherit inputs from the package
          inputsFrom = [ cim-claude-adapter ];

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
          shellHook = ''
            echo "🤖 CIM Claude Adapter Development Environment"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "📦 Rust toolchain: $(rustc --version)"
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
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          
          # Formatting check
          fmt = craneLib.cargoFmt {
            inherit src;
          };
          
          # Audit check (commented out for now due to advisory-db dependency)
          # audit = craneLib.cargoAudit {
          #   inherit src;
          #   advisory-db = pkgs.fetchFromGitHub {
          #     owner = "RustSec";
          #     repo = "advisory-db";
          #     rev = "main";
          #     sha256 = ""; # Would need actual hash
          #   };
          # };
          
          # Documentation check
          doc = craneLib.cargoDoc (commonArgs // {
            inherit cargoArtifacts;
          });
          
          # Test check
          test = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };
      }
    ) // {
      # NixOS module
      nixosModules.default = import ./nix/module.nix;
      nixosModules.cim-claude-adapter = import ./nix/module.nix;
      
      # Overlay for adding this package to nixpkgs
      overlays.default = final: prev: {
        cim-claude-adapter = self.packages.${final.system}.cim-claude-adapter;
      };
    };
}