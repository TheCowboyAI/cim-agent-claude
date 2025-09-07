{ lib, config, pkgs, ... }:

with lib;
let cfg = config.zsh;

in {
  options.zsh.enable = lib.mkEnableOption "Enable zsh";
  config = mkIf cfg.enable {
  programs.zsh = {
      enable = true;

      # directory to put config files in
      dotDir = "${config.xdg.configHome}/zsh";

      enableCompletion = true;
      autosuggestion.enable = true;
      syntaxHighlighting.enable = true;

      # .zshrc
      initContent = ''
        export DIRENV_LOG_FORMAT="";
        bindkey '^ ' autosuggest-accept
        eval "$(starship init zsh)"
      '';

      # basically aliases for directories: 
      # `cd ~dots` will cd into ~/.config/nixos
      dirHashes = {
      };

      # Tweak settings for history
      history = {
        save = 10000;
        size = 10000;
        path = "$HOME/.cache/zsh_history";
      };

      # Set some aliases
      shellAliases = {
        c = "clear";
        md = "mkdir -vp";
        rm = "rm -rifv";
        mv = "mv -iv";
        cp = "cp -riv";
        cat = "bat --paging=never --style=plain";
        ls = "eza -a --icons";
        tree = "eza --tree --icons";
      };

    };
  };
}
