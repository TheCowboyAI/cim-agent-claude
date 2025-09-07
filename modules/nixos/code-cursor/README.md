# Cursor AI Code Editor for NixOS

This module provides the [Cursor](https://cursor.sh/) AI-powered code editor for NixOS systems.

## Features

- Automated installation and configuration of Cursor
- Version management with automatic update notifications
- Wayland compatibility 
- Preconfigured settings

## Usage

### Basic Setup

Add the module to your NixOS configuration:

```nix
# Import the module in your configuration.nix or flake.nix
imports = [
  # ... other imports
  ./modules/nixos/code-cursor
];

# Enable the module
code-cursor.enable = true;
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `code-cursor.enable` | boolean | `false` | Whether to enable the Cursor editor |
| `code-cursor.package` | package | `cursorPackage` | The Cursor package to use |
| `code-cursor.notifyUpdates` | boolean | `true` | Show notifications when updates are available |

### Updating Cursor

The module automatically checks for updates and notifies you when a new version is available:

1. During system rebuild, a message will appear if an update is available
2. Run `check-cursor-updates` to see update details and get the hash

To update to a newer version:

1. Edit `modules/nixos/code-cursor/default.nix`
2. Update these two variables:
   ```nix
   cursorVersion = "new.version.number";
   cursorHash = "sha256-new-hash-value=";
   ```
3. Rebuild your system with `sudo nixos-rebuild switch --flake .#your-host`

The URL is automatically retrieved from the version history JSON file.

## Advanced Configuration

### Custom Settings

Settings for Cursor are stored in `modules/nixos/code-cursor/settings.json`. Edit this file to customize your Cursor experience.

### Technical Details

This module:

1. Uses the [cursor-version-history](https://github.com/oslook/cursor-ai-downloads) repository to lookup version URLs
2. Packages Cursor using `appimageTools.wrapType2` for NixOS compatibility
3. Provides wayland compatibility through environment variables

## Troubleshooting

If you encounter issues:

1. Ensure `appimage-run` works properly on your system
2. Check that the hash is correct for your version
3. Verify the version exists in the version history JSON

## Additional Commands

- `cursor-with-settings`: Launches Cursor with the configured settings
- `check-cursor-updates`: Checks for available updates and provides update instructions

## Using in Your Flake

You can use this module in your own flake:

```nix
{
  inputs.your-flake.url = "github:your-username/your-repo";
  
  outputs = { self, your-flake, ... }: {
    nixosConfigurations.your-host = nixosSystem {
      # ...
      modules = [
        your-flake.nixosModules.code-cursor
        # ...
      ];
    };
  };
}
```

## System Installation

Enable the module in your NixOS configuration:

```nix
{ config, pkgs, ... }:
{
  imports = [ ./path/to/code-cursor ];
  code-cursor.enable = true;
}
``` 