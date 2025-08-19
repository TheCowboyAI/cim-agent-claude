# CIM Agent Claude - Nix Flake
# Copyright 2025 - Cowboy AI, LLC. All rights reserved.

{
  description = "CIM Agent Claude - Event-driven Claude AI integration for CIM ecosystems with GUI management interface";

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
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ]; # For WASM GUI build
        };

        # Common build inputs for all packages
        commonBuildInputs = with pkgs; [
          openssl
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.libiconv
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        commonNativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
        ];

        # CIM Claude Adapter (backend service)
        cim-claude-adapter = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cim-claude-adapter";
          version = "0.1.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          buildInputs = commonBuildInputs;
          nativeBuildInputs = commonNativeBuildInputs;
          
          buildAndTestSubdir = "cim-claude-adapter";
          
          meta = with pkgs.lib; {
            description = "Event-driven Claude AI adapter service for CIM ecosystems";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;
          };
        };

        # CIM Claude GUI (desktop application)
        cim-claude-gui = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cim-claude-gui";
          version = "0.1.0";
          
          src = pkgs.lib.cleanSource ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          buildInputs = commonBuildInputs ++ (with pkgs; [
            # Additional GUI dependencies
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            libxkbcommon
            wayland
          ]);
          nativeBuildInputs = commonNativeBuildInputs;
          
          buildAndTestSubdir = "cim-claude-gui";
          
          meta = with pkgs.lib; {
            description = "Desktop GUI for managing CIM Claude conversations";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;
          };
        };

        # CIM Claude GUI WASM build for static web site
        cim-claude-gui-wasm = pkgs.stdenv.mkDerivation {
          pname = "cim-claude-gui-wasm";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          
          nativeBuildInputs = with pkgs; [
            rustToolchain
            wasm-pack
            nodePackages.npm
            binaryen # For wasm-opt optimization
          ];

          buildPhase = ''
            export HOME=$PWD
            cd cim-claude-gui
            
            # Build WASM package
            wasm-pack build --target web --out-dir pkg --features wasm
            
            # Optimize WASM binary
            wasm-opt -Oz pkg/cim_claude_gui_bg.wasm -o pkg/cim_claude_gui_bg.wasm
          '';

          installPhase = ''
            mkdir -p $out/share/cim-claude-gui-web
            cd cim-claude-gui
            
            # Copy WASM files
            cp -r pkg/* $out/share/cim-claude-gui-web/
            
            # Create index.html
            cat > $out/share/cim-claude-gui-web/index.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>CIM Claude GUI</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #f5f5f5;
        }
        #loading {
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            text-align: center;
        }
        canvas {
            display: block;
            margin: 0 auto;
        }
    </style>
</head>
<body>
    <div id="loading">
        <h2>Loading CIM Claude GUI...</h2>
        <p>Initializing WebAssembly application</p>
    </div>
    
    <script type="module">
        import init from './cim_claude_gui.js';
        
        async function run() {
            await init();
            document.getElementById('loading').style.display = 'none';
        }
        
        run().catch(console.error);
    </script>
</body>
</html>
EOF
          '';

          meta = with pkgs.lib; {
            description = "WebAssembly build of CIM Claude GUI for static web deployment";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;
          };
        };

        # Combined package with both components
        cim-agent-claude = pkgs.symlinkJoin {
          name = "cim-agent-claude";
          version = "0.1.0";
          paths = [ cim-claude-adapter cim-claude-gui ];
          
          meta = with pkgs.lib; {
            description = "Complete CIM Claude Agent with adapter service and GUI management interface";
            homepage = "https://github.com/TheCowboyAI/cim-agent-claude";
            license = licenses.mit;
            maintainers = [ "Cowboy AI, LLC <info@thecowboy.ai>" ];
            platforms = platforms.unix;
          };
        };

      in {
        # Packages
        packages = {
          default = cim-agent-claude;
          cim-agent-claude = cim-agent-claude;
          cim-claude-adapter = cim-claude-adapter;
          cim-claude-gui = cim-claude-gui;
          cim-claude-gui-wasm = cim-claude-gui-wasm;
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = commonBuildInputs ++ (with pkgs; [
            # GUI development dependencies
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            libxkbcommon
            wayland
          ]);
          nativeBuildInputs = commonNativeBuildInputs ++ (with pkgs; [
            # Additional development tools
            rust-analyzer
            clippy
            rustfmt
            cargo-audit
            cargo-watch
            cargo-tarpaulin
            wasm-pack
            
            # NATS tools
            natscli
            
            # General development tools
            jq
            curl
            httpie
            
            # Documentation tools
            mdbook
            
            # Nix tools
            nixpkgs-fmt
            nil
          ]);

          shellHook = ''
            echo "🤖 CIM Agent Claude Development Environment"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "📦 Rust toolchain: $(rustc --version)"
            echo "🔧 Available packages:"
            echo "  cim-claude-adapter      - Backend service"
            echo "  cim-claude-gui          - Desktop GUI"
            echo "  cim-claude-gui-wasm     - Web GUI (WASM)"
            echo ""
            echo "🏗️  Build commands:"
            echo "  cargo build                     - Build all packages"
            echo "  cargo run -p cim-claude-adapter - Run adapter service"
            echo "  cargo run -p cim-claude-gui     - Run desktop GUI"
            echo "  nix build .#cim-claude-adapter  - Build adapter with Nix"
            echo "  nix build .#cim-claude-gui      - Build GUI with Nix"
            echo "  nix build .#cim-claude-gui-wasm - Build web GUI with Nix"
            echo ""
            echo "🔍 Set environment variables:"
            echo "  export CLAUDE_API_KEY=your-api-key"
            echo "  export NATS_URL=nats://localhost:4222"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
          '';
        };

        # Apps
        apps = {
          default = flake-utils.lib.mkApp {
            drv = cim-claude-adapter;
            name = "cim-claude-adapter";
          };
          cim-claude-adapter = flake-utils.lib.mkApp {
            drv = cim-claude-adapter;
            name = "cim-claude-adapter";
          };
          cim-claude-gui = flake-utils.lib.mkApp {
            drv = cim-claude-gui;
            name = "cim-claude-gui";
          };
        };

        # Formatter
        formatter = pkgs.nixpkgs-fmt;

        # Checks (for CI)
        checks = {
          # Build checks
          build-adapter = cim-claude-adapter;
          build-gui = cim-claude-gui;
          build-wasm = cim-claude-gui-wasm;
        };
      }
    ) // {
      # NixOS modules
      nixosModules.default = import ./nix/module.nix;
      nixosModules.cim-agent-claude = import ./nix/module.nix;
      
      # Overlay for adding these packages to nixpkgs
      overlays.default = final: prev: {
        cim-agent-claude = self.packages.${final.system}.cim-agent-claude;
        cim-claude-adapter = self.packages.${final.system}.cim-claude-adapter;
        cim-claude-gui = self.packages.${final.system}.cim-claude-gui;
        cim-claude-gui-wasm = self.packages.${final.system}.cim-claude-gui-wasm;
      };
    };
}