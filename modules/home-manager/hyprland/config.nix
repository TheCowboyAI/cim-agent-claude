{ lib, ... }:

{
  # see: https://github.com/natpen/awesome-wayland

  wayland.windowManager.hyprland = {
    enable = true;
    systemd.enable = true;
    systemd.variables = ["--all"];
    xwayland.enable = true;

    settings = {
      "$mod" = "SUPER";

      monitor = [
        "eDP-1, 3840x2160, 0x0, 1"
        # "DP-3, preferred, auto, 1, mirror, eDP-1"
      ];

      exec-once = [
        "/usr/lib/xdg-desktop-portal-hyprland &"
        "eww daemon"
        "eww open bar"
        "wl-paste --watch cliphist store"
        "wl-paste --type text --watch cliphist store #Stores only text data"
        "wl-paste --type image --watch cliphist store #Stores only image data"
        "dunst"
        "dbus-update-activation-environment --systemd --all"
        "dbus-update-activation-environment --systemd WAYLAND_DISPLAY XDG_CURRENT_DESKTOP"
      ];

      bindm = [
        # Drag windows with SUPER + left click anywhere on window
        "$mod, mouse:272, movewindow"
        # Resize windows with SUPER + right click  
        "$mod, mouse:273, resizewindow"
      ];

      bind =
        [
          "$mod, V, exec, cliphist list | rofi -dmenu -p 'Clipboard' | cliphist decode | wl-copy"

          # Screenshot a window
          "$mod, PRINT, exec, hyprshot -m window"
          # Screenshot a monitor
          ", PRINT, exec, hyprshot -m output"
          # Screenshot a region
          "$mod SHIFT, PRINT, exec, hyprshot -m region"
          # Save screenshot directly to Pictures/Screenshots
          "$mod ALT, PRINT, exec, grim -g \"$(slurp)\" $HOME/Pictures/Screenshots/Screenshot-$(date +%F_%T).png"

          # ", Print, exec, grim - | wl-copy && wl-paste > ~/Pictures/Screenshots/Screenshot-$(date +%F_%T).png | dunstify 'Screenshot of whole screen taken' -t 1000" # screenshot of the whole screen
          "$mod, W, exec, google-chrome-stable"
          "$mod, R, exec, rofi -show drun"  # Rofi application launcher - much better than anyrun
          "$mod SHIFT, R, exec, rofi -show run"  # Rofi run command mode
          "$mod, B, exec, blender"
          "$mod, E, exec, cursor"
          "$mod, RETURN, exec, terminator"
          "$mod SHIFT, RETURN, exec, kitty"

          "$mod, M, exec, spotify"
          "$mod, C, exec, wasistlos"
          "$mod, Z, exec, zoom"

          "$mod, L, exec, loginctl lock-session"
          "$mod ALT, Q, killactive,"
          "$mod CONTROL, X, exit,"
          "$mod, F, exec, dolphin"
          "$mod, T, togglefloating,"
          "$mod ALT, F, fullscreen,"
          "$mod SHIFT, T, exec, ~/.config/hypr/toggle-layout.sh"

          # Move ws with mainMod + ALT + SHIFT + arrow keys
          "$mod ALT SHIFT, left, workspace, -1"
          "$mod ALT SHIFT, right, workspace, +1"

          # Move focus with mainMod + arrow keys
          "$mod, left, movefocus, l"
          "$mod, right, movefocus, r"
          "$mod, up, movefocus, u"
          "$mod, down, movefocus, d"

          # Resize with mainMod + ALT + arrow keys
          "$mod ALT, left, resizeactive, -30 0"
          "$mod ALT, right, resizeactive, 30 0"
          "$mod ALT, up, resizeactive, 0 30"
          "$mod ALT, down, resizeactive, 0 -30"

          # Move Window with mainMod + CTRL + arrow keys
          "$mod CONTROL, left, moveactive, -30 0"
          "$mod CONTROL, right, moveactive, 30 0"
          "$mod CONTROL, up, moveactive, 0 30"
          "$mod CONTROL, down, moveactive, 0 -30"

          # Position Window with mainMod + SHIFT + arrow keys
          "$mod SHIFT, left, movewindow, l"
          "$mod SHIFT, right, movewindow, r"
          "$mod SHIFT, up, movewindow, u"
          "$mod SHIFT, down, movewindow, d"

          #volume
          ", XF86AudioRaiseVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+"
          ", XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"
          ", XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"

          #media keys
          ", XF86AudioPlay, exec, playerctl play-pause"
          ", XF86AudioPause, exec, playerctl pause"
          ", XF86AudioNext, exec, playerctl next"
          ", XF86AudioPrev, exec, playerctl previous"

          #track (alternative keybinds)
          "$mod CONTROL, left, exec, playerctl prev"
          "$mod CONTROL, right, exec, playerctl next"
        ]
        ++ (
          # workspaces
          # binds $mod + [shift +] {1..10} to [move to] workspace {1..10}
          builtins.concatLists (builtins.genList
            (
              x: let
                ws = let
                  c = (x + 1) / 10;
                in
                  builtins.toString (x + 1 - (c * 10));
              in [
                "$mod, ${ws}, workspace, ${toString (x + 1)}"
                "$mod SHIFT, ${ws}, movetoworkspace, ${toString (x + 1)}"
              ]
            )
            10)
        );

      input = {
        scroll_method = "2fg";
      };

      # https://wiki.hyprland.org/Configuring/Variables/#decoration
      decoration = {
        rounding = 10;
        # Change transparency of focused and unfocused windows
        active_opacity = 1.0;
        inactive_opacity = 1.0;
        fullscreen_opacity = 1.0;

        shadow = {
          enabled = true;
          range = 10;
          render_power = 3;
        };

        # https://wiki.hyprland.org/Configuring/Variables/#blur
        blur = {
          enabled = true;
          size = 3;
          passes = 1;
          vibrancy = 0.1696;
        };
      };
      
      # General settings
      general = {
        border_size = 3;  # Larger border for easier dragging
        gaps_in = 5;
        gaps_out = 10;
        resize_on_border = true;
        extend_border_grab_area = 15;  # Larger grab area around borders
        hover_icon_on_border = true;   # Show resize cursor on borders
        "col.active_border" = lib.mkForce "0xff7c6f64";
        "col.inactive_border" = lib.mkForce "0xff3c3836";
      };
      
      # Window rules (tiling by default, floating for specific apps)
      windowrulev2 = [
        # Window rules for xsane
        "float, class:^(xsane)$"
        "center, class:^(xsane)$"
        "size 800 600, class:^(xsane)$, title:^(xsane)$"
        "move 100 100, class:^(xsane)$, title:^(Preview).*"
        "size 600 800, class:^(xsane)$, title:^(Preview).*"
        "pin, class:^(xsane)$, title:^(Progress).*"
        "move 750 100, class:^(xsane)$, title:^(Batch scan).*"
        
        # Rofi window rules with zoom animation (0% to 100%)
        "stayfocused,class:^(Rofi)$"
        "animation popin,class:^(Rofi)$"
      ];
      
      # Layer rules for rofi
      layerrule = [
        "animation popin,rofi"
      ];
      
      # Animations configuration
      animations = {
        enabled = true;
        
        # Bezier curves for smooth animations
        bezier = [
          "overshot, 0.05, 0.9, 0.1, 1.05"
          "smoothOut, 0.36, 0, 0.66, -0.56"
          "smoothIn, 0.25, 1, 0.5, 1"
          "easeOutBack, 0.175, 0.885, 0.32, 1.275"
        ];
        
        # Animation rules
        animation = [
          "windows, 1, 5, easeOutBack, popin"
          "windowsOut, 1, 4, smoothIn, popin"
          "windowsMove, 1, 4, default"
          "border, 1, 10, default"
          "fade, 1, 10, smoothIn"
          "fadeDim, 1, 10, smoothIn"
          "workspaces, 1, 6, default"
          "specialWorkspace, 1, 5, easeOutBack, slidevert"
        ];
      };
    };
  };
}
