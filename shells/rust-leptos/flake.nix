{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
            inherit system overlays;
            config.allowUnfree = true;
          };
      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShells."x86_64-linux".default = with pkgs; mkShell {
        packages = [
              rustToolchain
              cargo-edit
              cargo-expand
              cargo-udeps
              cargo-whatfeatures
              cargo-leptos
              cargo-generate
              cargo-make
              cacert
              trunk
              direnv
              lld
              clang
              gcc
              zsh
              git
              just
              starship
              openssl
              openssl.dev
              pkg-config
              zlib.dev
              alsa-lib
              xorg.libX11
              xorg.libXi
              xorg.libXcursor
              libpulseaudio
              libGL
              libglvnd
              libiconv
              tailwindcss
              sass
              glibc
              # Latest Node.js for Claude Code compatibility
              nodejs_22
        ];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            # see: https://discourse.nixos.org/t/running-a-rust-application-that-needs-egl-with-shell-nix/33245/3
            LD_LIBRARY_PATH = "${pkgs.libglvnd}/lib";

            shellHook = ''
              echo "═════════════════════════════════════════════════"
              echo "🦀 Rust Leptos Development Shell Activated"
              echo "═════════════════════════════════════════════════"
              echo "🦀 Rust: $(rustc --version)"
              echo "🟢 Node.js: $(node --version) (Claude Code compatible)"
              echo "🌊 Leptos: Ready for full-stack Rust development"
              echo ""
              echo "💻 Claude Code Integration:"
              echo "   claude --resume    - Resume Claude session"
              echo "   claude --new       - Start new Claude session"
              echo ""
              echo "📦 Leptos Commands:"
              echo "   cargo leptos watch    - Start development server"
              echo "   cargo leptos build    - Production build"
              echo "   trunk serve           - Alternative development server"
              echo "═════════════════════════════════════════════════"
              
              if [ -f .env ]; then
                export $(grep -v '^#' .env | xargs)
              fi
              export GIT_CONFIG_NOSYSTEM=1
              ZSH_CUSTOM=$HOME/.config/zsh
              export PATH="$HOME/.cargo/bin:$PATH"
              export LD_LIBRARY_PATH="${pkgs.libglvnd}/lib";
            '';
      };
    };
}
