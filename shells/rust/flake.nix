# SAGE-Orchestrated CIM Rust Development Shell
# Mathematical functional composition for CIM Agent Claude  
# Genesis: Orchestrated by SAGE for complete CIM development journey
# Enhanced: OpenSSL version consistency for Node.js/Claude Code compatibility
{
  description = "SAGE-orchestrated CIM Agent Claude development environment with OpenSSL consistency";
  
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
          overlays = [ rust-overlay.overlays.default ];
          config.allowUnfree = true;
        };
        
        # OpenSSL version consistency - ensure we use the latest stable version
        # that supports both Rust compilation and Node.js 20.x requirements
        opensslVersion = pkgs.openssl; # Use current OpenSSL (3.5.x) for compatibility
        
        # SAGE-orchestrated Rust toolchain with complete CIM support
        rustToolchain = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
          targets = [ "wasm32-unknown-unknown" ]; # WASM support for GUI
        };
        
        # Latest Node.js (22.x) with OpenSSL compatibility for Claude Code
        # Using nodejs_22 for better performance, security, and latest features
        compatibleNodejs = pkgs.nodejs_22.override {
          openssl = opensslVersion;
        };
        
        # CIM-specific development dependencies
        cimDevTools = with pkgs; [
          # NATS ecosystem (CIM messaging backbone)
          natscli
          nsc
          nats-server
          
          # Development and debugging tools
          gdb
          lldb
          valgrind
          strace
          
          # Network and protocol tools
          wireshark
          tcpdump
          netcat
          
          # Documentation and visualization
          graphviz
          plantuml
        ];
        
        # GUI and multimedia libraries for Iced applications
        guiLibraries = with pkgs; [
          # Core GUI support
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libXext
          xorg.libXfixes
          
          # Wayland support
          wayland
          wayland-protocols
          libxkbcommon
          
          # Graphics and rendering
          vulkan-loader
          vulkan-tools
          mesa
          libGL
          
          # Font and text rendering
          fontconfig
          freetype
          harfbuzz
          
          # Audio support (for potential multimedia features)
          alsa-lib
          pipewire
          
          # Image and multimedia
          libjpeg
          libpng
          libwebp
          
          # Clipboard support
          wl-clipboard
          xclip
        ];
        
        # System libraries and build dependencies
        systemLibraries = with pkgs; [
          # SSL/TLS support - use consistent OpenSSL version
          opensslVersion.dev
          opensslVersion.out
          
          # System interface libraries
          systemd.dev
          dbus
          
          # Build tools and compilers
          pkg-config
          cmake
          ninja
          gcc
          clang
          llvm
          
          # Standard C libraries
          glibc.dev
          libiconv
          zlib
          
          # Thread and concurrency support
          libunwind
        ];
      in
      {
        devShells.default = with pkgs; mkShell {
          packages = [
            # SAGE-orchestrated Rust toolchain
            rustToolchain
            
            # Claude Code compatible Node.js
            compatibleNodejs
            
            # Essential development tools
            git
            curl
            jq
            wget
            
            # Browser for testing web components
            firefox
            chromium
          ] ++ cimDevTools ++ guiLibraries ++ systemLibraries;
        
          # SAGE-orchestrated environment variables for CIM development
          
          # Rust compilation environment
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          RUST_BACKTRACE = "full";
          RUST_LOG = "debug";
          
          # OpenSSL configuration for NATS TLS connections - consistent version
          OPENSSL_DIR = "${opensslVersion.dev}";
          OPENSSL_LIB_DIR = "${opensslVersion.out}/lib";
          OPENSSL_INCLUDE_DIR = "${opensslVersion.dev}/include";
          OPENSSL_NO_VENDOR = "1";
          
          # PKG_CONFIG paths for all dependencies
          PKG_CONFIG_PATH = lib.concatStringsSep ":" [
            "${opensslVersion.dev}/lib/pkgconfig"
            "${fontconfig.dev}/lib/pkgconfig"
            "${libxkbcommon.dev}/lib/pkgconfig"
            "${wayland.dev}/lib/pkgconfig"
            "${wayland-protocols}/share/pkgconfig"
            "${vulkan-loader.dev}/lib/pkgconfig"
          ];
          
          # Complete library path for GUI applications
          LD_LIBRARY_PATH = lib.makeLibraryPath (guiLibraries ++ systemLibraries);
          
          # Display and window system configuration
          WINIT_UNIX_BACKEND = "x11,wayland"; # Allow both backends
          WAYLAND_DISPLAY = "wayland-0";
          XDG_SESSION_TYPE = "wayland";
          
          # CIM development specific
          CIM_ENV = "development";
          CIM_LOG_LEVEL = "debug";
          NATS_URL = "nats://localhost:4222";
          
          # Vulkan and graphics
          VK_LAYER_PATH = "${vulkan-tools}/share/vulkan/explicit_layer.d";
          
          # Font configuration
          FONTCONFIG_FILE = "${fontconfig.out}/etc/fonts/fonts.conf";
          
          # Disable network sandbox for development
          CARGO_NET_OFFLINE = "false";
        
          shellHook = ''
            echo "═══════════════════════════════════════════════════════════════════════"
            echo "🧠 SAGE-Orchestrated CIM Agent Claude Development Environment"
            echo "═══════════════════════════════════════════════════════════════════════"
            echo "🦀 Rust: $(rustc --version)"
            echo "🟢 Node.js: $(node --version) (Claude Code compatible)"
            echo "🔒 OpenSSL: $(openssl version) at $OPENSSL_DIR"
            echo "🖥️  Display: $WINIT_UNIX_BACKEND (Wayland + X11 support)"
            echo "🌐 NATS: $NATS_URL"
            echo "🎯 Environment: $CIM_ENV"
            echo ""
            echo "📦 Available Binaries:"
            echo "   cargo run --bin cim-agent-claude    - Main CIM orchestrator"
            echo "   cargo run --bin sage                - SAGE CLI interface"
            echo "   cargo run --bin sage-service        - SAGE service daemon"
            echo "   cargo run --bin sage-test           - SAGE testing harness"
            echo ""
            echo "🎨 GUI Development:"
            echo "   cd cim-claude-gui && cargo run      - Native GUI (Iced + TEA)"
            echo "   cd cim-claude-gui && cargo build --target wasm32-unknown-unknown - WASM build"
            echo ""
            echo "💻 Claude Code Integration:"
            echo "   claude --resume                      - Resume Claude Code session"
            echo "   claude --new                         - Start new Claude Code session"
            echo "   node --version                       - Check Node.js compatibility"
            echo ""
            echo "🔧 CIM Development Commands:"
            echo "   cargo build --release               - Production build"
            echo "   cargo test                           - Run test suite"
            echo "   cargo clippy                         - Lint analysis"
            echo "   cargo doc --open                     - Generate documentation"
            echo ""
            echo "🌊 NATS Tools:"
            echo "   nats-server                          - Start local NATS server"
            echo "   nats sub \"sage.>\"                    - Monitor SAGE events"
            echo "   nsc                                  - NATS security management"
            echo ""
            echo "🔍 Debugging:"
            echo "   gdb ./target/debug/cim-agent-claude - Debug with GDB"
            echo "   RUST_LOG=trace cargo run            - Detailed logging"
            echo "   valgrind ./target/debug/...         - Memory analysis"
            echo "═══════════════════════════════════════════════════════════════════════"
            echo "🧙 SAGE Orchestration: Mathematical CIM development journey activated"
            echo "═══════════════════════════════════════════════════════════════════════"
          '';
        };
      });
}
