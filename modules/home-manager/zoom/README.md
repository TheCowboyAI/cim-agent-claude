# Zoom via Flatpak Home Manager Module

This module installs and manages Zoom using Flatpak.

## Requirements

- NixOS must have Flatpak enabled at the system level:
```nix
# In your NixOS configuration
services.flatpak.enable = true;
```

## Usage

Add this to your Home Manager configuration:

```nix
{ pkgs, ... }:

{
  # Enable the Zoom Flatpak module
  programs.zoom-flatpak.enable = true;
}
```

## Features

- Installs Zoom from Flathub
- Automatically updates Zoom weekly via a systemd user timer
- Sets up required XDG environment variables

## Notes

- Flatpak applications run in a sandbox and may need additional permissions
- First run may require accepting Flatpak permissions dialogs
- You may need to logout and login again after first installation

## Troubleshooting

If you encounter issues with the module:

1. Ensure Flatpak is enabled at the system level:
```nix
services.flatpak.enable = true;
```

2. Try running the Flatpak installation manually:
```bash
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user -y flathub us.zoom.Zoom
```

3. Check if Flatpak is properly installed and accessible:
```bash
which flatpak
flatpak --version
```

4. Look at the Home Manager service logs for errors:
```bash
journalctl -u home-manager-$USER.service
``` 