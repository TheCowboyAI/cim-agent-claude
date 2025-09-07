{
  description = "NixOS configuration for TheCowboy.AI dev system";
  
  nixConfig = {
    sandbox = false;  # Disable sandbox completely to fix cargo vendor permission issues
    extra-sandbox-paths = [ ];
    allow-import-from-derivation = true;
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
    extra-substituters = [
      "https://nix-community.cachix.org"
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
    nix-inspect.url = "github:bluskript/nix-inspect";
    home-manager.url = "github:nix-community/home-manager";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
    agenix.url = "github:ryantm/agenix";
    agenix.inputs.nixpkgs.follows = "nixpkgs";
    alejandra = {
      url = "github:kamadorueda/alejandra";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # don't forget the submodules, or hyprland.homeManagerModules won't build
    hyprland.url = "git+https://github.com/hyprwm/Hyprland?submodules=1";
    hyprland-plugins = {
      url = "github:hyprwm/hyprland-plugins";
      inputs.hyprland.follows = "hyprland";
    };
    hyprpanel.url = "github:Jas-SinghFSU/HyprPanel";

    nur.url = "github:nix-community/NUR";

    # anyrun = {
    #   url = "github:anyrun-org/anyrun";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
    # anyrun-nixos-options.url = "github:n3oney/anyrun-nixos-options";  # Disabled due to unicode compilation issues

    microvm.url = "github:astro/microvm.nix";
    microvm.inputs.nixpkgs.follows = "nixpkgs";

    nix-colors.url = "github:thecowboyai/nix-colors";
    stylix.url = "github:danth/stylix";

    shell-rust = {
      url = "path:./shells/rust";  
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
        flake-utils.follows = "flake-utils";
      };
    };
    shell-rust-leptos.url = "path:./shells/rust-leptos";
    shell-simple.url = "path:./shells/simple";
    shell-go.url = "path:./shells/go";

    cursor-version-history = {
      url = "github:oslook/cursor-ai-downloads/main";
      flake = false;
    };
    
      cim-sage = {
      url = "git+ssh://git@github.com/thecowboyai/cim-sage.git";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # claudia = {
    #   url = "github:getAsterisk/claudia";
    #   flake = false;
    # };
  };

  outputs = {
    self,
    nixpkgs,
    home-manager,
    rust-overlay,
    flake-utils,
    # anyrun,  # Disabled due to unicode compilation issues
    # anyrun-nixos-options,
    nix-colors,
    agenix,
    alejandra,
    hyprland,
    hyprland-plugins,
    hyprpanel,
    nur,
    microvm,
    shell-rust,
    shell-rust-leptos,
    shell-simple,
    shell-go,
    cursor-version-history,
    cim-sage,
    naersk,
    # claudia,
    ...
  } @ inputs: let
    system = "x86_64-linux";
    
    pkgs = import nixpkgs {
      inherit system;
      overlays = [];
    };
    
    # Get lib from nixpkgs
    inherit (nixpkgs) lib;
  in {
    nixosConfigurations = {
      "dell-62S6063" = nixpkgs.lib.nixosSystem {
        inherit system;
        specialArgs = inputs;
        modules = [
          inputs.stylix.nixosModules.stylix
          microvm.nixosModules.host
          ./hosts
          ./modules/nixos/stylix
          ./modules/settings/colors
          ./users/steele/user.nix
          home-manager.nixosModules.home-manager
          {
            home-manager.useGlobalPkgs = true;
            home-manager.useUserPackages = true;
            home-manager.extraSpecialArgs = {inherit inputs;};
            home-manager.users.steele = import ./users/steele/home.nix;
            home-manager.backupFileExtension = "backup";
          }
          # CIM SAGE packages overlay
          {
            nixpkgs.overlays = [
              (final: prev: {
                inherit (cim-sage.packages.${system}) sage sage-service;
              })
            ];
          }
        ];
      };
    };

        # Export CIM SAGE packages from the imported flake
    packages.${system} = {
      inherit (cim-sage.packages.${system})
        sage-service
        sage
        default;
    };
    
    # Export the code-cursor module
    nixosModules.code-cursor = { config, ... }: import ./modules/nixos/code-cursor {
      inherit (inputs) cursor-version-history;
      inherit (inputs.nixpkgs) lib;
      inherit config;
      inherit pkgs;
    };

    devShells."x86_64-linux" = {
      default = shell-simple.devShells."x86_64-linux".default;
      rust = shell-rust.devShells."x86_64-linux".default;
      rust-leptos = shell-rust-leptos.devShells."x86_64-linux".default;
      go = shell-go.devShells."x86_64-linux".default;
      simple = shell-simple.devShells."x86_64-linux".default;
    };
  };
}
