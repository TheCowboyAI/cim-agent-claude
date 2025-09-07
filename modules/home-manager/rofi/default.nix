{ lib, config, osConfig, pkgs, inputs, ... }:

with lib;
let cfg = config.rofi;

in {
  options.rofi.enable = lib.mkEnableOption "Enable rofi application launcher";
  
  config = mkIf cfg.enable {
    # Create the complete rofi configuration file
    xdg.configFile."rofi/config.rasi".text = ''
      configuration {
          modi: "drun,run,window,ssh";
          show-icons: true;
          icon-theme: "Adwaita";
          font: "JetBrainsMono 18";
          display-drun: " Apps";
          display-run: " Run";
          display-window: " Windows";
          display-ssh: " SSH";
          drun-display-format: "{name}";
          window-format: "{w} {c} {t}";
          sidebar-mode: true;
          sorting-method: "fzf";
          case-sensitive: false;
          cycle: true;
      }

      * {
          /* Cowboy AI Dark Theme Colors */
          background:     rgba(17, 18, 29, 0.95);
          background-alt: rgba(43, 44, 56, 0.6);
          foreground:     #fbf1c7;
          selected:       rgba(149, 197, 97, 0.8);
          active:         rgba(215, 166, 95, 0.8);
          urgent:         rgba(204, 36, 29, 0.8);
          
          border-color:   rgba(215, 166, 95, 0.8);
          separatorcolor: rgba(124, 111, 100, 0.6);
          
          normal-background:     @background;
          normal-foreground:     @foreground;
          alternate-normal-background: @background-alt;
          alternate-normal-foreground: @foreground;
          selected-normal-background:  @selected;
          selected-normal-foreground:  #11121d;
          
          active-background:     @active;
          active-foreground:     @background;
          alternate-active-background: @background-alt;
          alternate-active-foreground: @active;
          selected-active-background:  @active;
          selected-active-foreground:  @background;
          
          urgent-background:     @urgent;
          urgent-foreground:     @background;
          alternate-urgent-background: @background-alt;
          alternate-urgent-foreground: @urgent;
          selected-urgent-background:  @urgent;
          selected-urgent-foreground:  @background;
      }

      window {
          transparency:     "real";
          background-color: @background;
          border:           3px;
          border-color:     @border-color;
          border-radius:    16px;
          padding:          30px;
          width:            75%;
          height:           90%;
          location:         northwest;
          anchor:           northwest;
          x-offset:         20px;
          y-offset:         80px;
      }

      mainbox {
          border:  0;
          padding: 0;
          background-color: transparent;
      }

      message {
          border:       2px 0px 0px;
          border-color: @separatorcolor;
          padding:      10px;
      }

      textbox {
          text-color: @foreground;
      }

      listview {
          fixed-height: false;
          border:       0px;
          border-color: @separatorcolor;
          spacing:      8px;
          scrollbar:    true;
          padding:      15px 0px 0px;
          background-color: transparent;
      }

      element {
          border:  0;
          padding: 12px 18px;
          border-radius: 12px;
          background-color: transparent;
          text-color: @normal-foreground;
      }

      element-text {
          background-color: transparent;
          text-color:       inherit;
      }

      element-icon {
          background-color: transparent;
          text-color:       inherit;
          size:             36px;
          padding:          0px 15px 0px 0px;
      }

      element normal.normal {
          background-color: @normal-background;
          text-color:       @normal-foreground;
      }

      element normal.urgent {
          background-color: @urgent-background;
          text-color:       @urgent-foreground;
      }

      element normal.active {
          background-color: @active-background;
          text-color:       @active-foreground;
      }

      element selected.normal {
          background-color: @selected-normal-background;
          text-color:       @selected-normal-foreground;
          border:           2px;
          border-color:     @border-color;
      }

      element selected.urgent {
          background-color: @selected-urgent-background;
          text-color:       @selected-urgent-foreground;
      }

      element selected.active {
          background-color: @selected-active-background;
          text-color:       @selected-active-foreground;
      }

      element alternate.normal {
          background-color: @alternate-normal-background;
          text-color:       @alternate-normal-foreground;
      }

      element alternate.urgent {
          background-color: @alternate-urgent-background;
          text-color:       @alternate-urgent-foreground;
      }

      element alternate.active {
          background-color: @alternate-active-background;
          text-color:       @alternate-active-foreground;
      }

      scrollbar {
          width:        4px;
          border:       0;
          handle-color: @active;
          handle-width: 8px;
          padding:      0;
          background-color: @background-alt;
      }

      mode-switcher {
          border:       2px 0px 0px;
          border-color: @separatorcolor;
          background-color: @background-alt;
          padding: 10px;
      }

      button {
          text-color: @normal-foreground;
          background-color: @background;
          padding: 12px 24px;
          border-radius: 12px;
      }

      button selected {
          background-color: @selected-normal-background;
          text-color:       @selected-normal-foreground;
      }

      inputbar {
          spacing:    12px;
          text-color: @normal-foreground;
          padding:    18px;
          background-color: @background-alt;
          border-radius: 18px;
          border: 3px;
          border-color: rgba(85, 86, 97, 0.7);
          margin: 0px 0px 15px 0px;
          children:   [ prompt,textbox-prompt-colon,entry,case-indicator ];
      }

      case-indicator {
          spacing:    0;
          text-color: @normal-foreground;
      }

      entry {
          spacing:    0;
          text-color: @normal-foreground;
          placeholder: "Search applications...";
          placeholder-color: rgba(171, 178, 191, 0.6);
      }

      prompt {
          spacing:    0;
          text-color: @active;
          font:       "JetBrainsMono Bold 18";
      }

      textbox-prompt-colon {
          expand:     false;
          str:        ":";
          margin:     0px 0.3em 0em 0em;
          text-color: @active;
      }
    '';
    
    programs.rofi = {
      enable = true;
      package = pkgs.rofi-wayland;
      terminal = "${pkgs.wezterm}/bin/wezterm";
      
      # Don't set theme or extraConfig - everything is in config.rasi
      theme = lib.mkForce null;
      extraConfig = lib.mkForce {};
    };
  };
}