# Window Manager - hyprland
hyprland is our choice for window managers.
It is a wayland compositor which allow flexible use of both tiling and stacking window approaches. It is a stable and advanced addition which has wide support for accompliching our goal of customizable local window management.

# More About Hyperland
Hyprland is a Wayland compositor that is gaining popularity in the Linux community due to its unique features and performance benefits. Here are some key aspects that make it a compelling choice for setting up a Linux desktop:

1. **Wayland Support**: Hyprland is built on Wayland, the modern replacement for the X11 display server protocol. Wayland aims to provide smoother graphics and better handling of high-resolution displays, which can lead to an overall improved user experience.

2. **Customizability**: Hyprland offers extensive customization options. Users can tailor many aspects of their desktop environment, including window behaviors, keyboard shortcuts, and appearance settings. This level of customization is particularly appealing to users who want to personalize their workspace to fit their needs.

3. **Performance**: Hyprland is known for its efficiency and low resource usage. This makes it a good option for users who want a lightweight yet powerful desktop environment, especially on systems with limited resources.

4. **Tiling Window Management**: It includes features of tiling window managers, which automatically organize windows to maximize screen real estate. This is useful for productivity and multi-tasking.

5. **Active Development**: Hyprland is under active development, with a community that is continuously working on adding new features and improvements. This ensures that the compositor stays up-to-date with the latest technologies and trends in desktop environments.

6. **Security and Stability**: Since Hyprland is based on Wayland, it inherently benefits from Wayland's security model, which is more robust compared to X11. This can lead to a more secure and stable desktop experience.

7. **Community and Support**: Hyprland has a growing community of users and developers. The availability of support and resources, such as documentation, forums, and chat channels, can be valuable for both new and experienced users.

In summary, Hyprland is an attractive option for Linux users due to its modern architecture, customization capabilities, performance efficiency, and the active community supporting it. It is especially suitable for users who prefer a tiling window manager approach and those who are interested in a Wayland-based environment.

## Hyprland Base
see: 
- https://wiki.hyprland.org/Useful-Utilities/Must-have/
- https://wiki.hyprland.org/Nix/Hyprland-on-NixOS/

Our base for a configured Hyprland system:

  - hyprland
    - default.nix 
      - settings and activation
    - hyprland-config.nix
      - the actually config to be written for hyprland
  - sway
    - window manager 
  - swaybg
    - desktop background
  - swayidle
    - timer/keystrokes for locking the screen
  - dconf
    - settings
  -XDG
    - desktop portal
  - Dunst
    - notifications
  - Pipewire _ wireplumber
    - screen sharing
  - polkit
    - polkit-kde-agent
  - QT
    - add wayland support for QT apps
    - qt5-wayland and qt6-wayland
  - Themes
  - Media