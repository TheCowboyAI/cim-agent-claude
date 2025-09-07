default:
  @just --list

# Auto-format the source tree
fmt:
  treefmt

# we need sudo to update the lock file and write to /nix/store, assuming this is running on the live system

# Run nixos-rebuild switch
switch:
    sudo nixos-rebuild switch --flake .#dell-62S6063 --impure --show-trace

check: 
    sudo nix flake check --impure

update: 
    sudo nix flake update --impure 

upgrade:
  just update && just switch
  

