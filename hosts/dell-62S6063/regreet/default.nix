{ config, lib, pkgs, ... }:

{
  # Regreet configuration
  environment.etc."greetd/regreet.toml".text = ''
    [terminal]
    # VT to use
    vt = 1

    [default_session]
    # The command to run when starting the default session
    command = "${pkgs.hyprland}/bin/Hyprland"

    [background]
    # Use a solid color instead of an image
    color = "#1E1E1E"

    [GTK]
    # Name of the cursor theme
    cursor_theme_name = "Adwaita"
    # Name of the icon theme
    icon_theme_name = "Adwaita"
    # GTK application theme name
    theme_name = "Adwaita-dark"

    [notifications]
    # Border width
    border_width = 0
    # Border radius
    border_radius = 5
    # Padding
    padding = 24
    # Background color (RGBA)
    background_color = "#1E1E1E99"
    # Border color (RGBA)
    border_color = "#FFFFFFFF"
    # Progress bar background color (RGBA)
    progress_background_color = "#1E1E1EFF"
    # Progress bar foreground color (RGBA)
    progress_foreground_color = "#FFFFFFFF"
    # Margin above first notification
    margin_top = 8
    # Spacing between notifications
    spacing = 8
  '';

  # Add comprehensive GTK dependencies to ensure proper functioning
  environment.systemPackages = with pkgs; [
    # GTK and theme dependencies
    gtk3
    gtk4
    adwaita-icon-theme
    hicolor-icon-theme
    gsettings-desktop-schemas
    regreet
  ];

  # Set necessary environment variables for GTK to function properly
  environment.sessionVariables = {
    # Essential for GTK to find themes and icons
    XDG_DATA_DIRS = [ 
      "${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}"
      "${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}"
      "${pkgs.adwaita-icon-theme}/share"
    ];
    # Make sure GTK can find its modules
    GTK_PATH = "${lib.makeSearchPath "lib/gtk-3.0" [ pkgs.gtk3 ]}";
  };

  # Create required symbolic links for GTK themes
  system.activationScripts.setupGtkThemes = ''
    mkdir -p /run/current-system/sw/share/icons
    mkdir -p /run/current-system/sw/share/themes
  '';
} 