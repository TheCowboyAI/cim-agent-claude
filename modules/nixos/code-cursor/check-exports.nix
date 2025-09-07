# check-exports.nix
# A simple script to check our module exports
{ pkgs ? import <nixpkgs> { } }:

let
  # Import our module to check the exports
  lib = pkgs.lib;
  
  # Mock config
  config = {
    code-cursor.enable = false;
  };
  
  # Import the module
  cursorModule = import ./default.nix {
    inherit pkgs lib config;
  };
  
  # Function to check if an attribute exists and has a package property
  checkPackage = name: attr:
    if attr ? package && builtins.isAttrs attr
    then pkgs.writeScriptBin "check-${name}" ''
      echo "${name} export check passed"
      exit 0
    ''
    else pkgs.writeScriptBin "check-${name}" ''
      echo "${name} export check FAILED - no package attribute found"
      exit 1
    '';
    
  # Create checkers for each export
  getUrlChecker = checkPackage "getUrl" cursorModule.getUrl;
  cursorUpdaterChecker = checkPackage "cursorUpdater" cursorModule.cursorUpdater;
  
  # Check if cursor is correctly exported
  cursorChecker = 
    if cursorModule ? cursor && builtins.isAttrs cursorModule.cursor
    then pkgs.writeScriptBin "check-cursor" ''
      echo "cursor export check passed"
      exit 0
    ''
    else pkgs.writeScriptBin "check-cursor" ''
      echo "cursor export check FAILED - cursor not found or not an attribute"
      exit 1
    '';
    
  # Create a combined checker
  combinedChecker = pkgs.writeScriptBin "check-all" ''
    ${getUrlChecker}/bin/check-getUrl
    ${cursorUpdaterChecker}/bin/check-cursorUpdater
    ${cursorChecker}/bin/check-cursor
    echo "All checks passed!"
  '';
    
in {
  inherit getUrlChecker cursorUpdaterChecker cursorChecker combinedChecker;
  
  default = combinedChecker;
} 