{ lib, config, ... }:

with lib;
let cfg = config.starship;

in {
  options.starship.enable = lib.mkEnableOption "Enable starship";
  config = mkIf cfg.enable {
    programs.starship = {
      enable = true;
      enableZshIntegration = true;
      enableBashIntegration = true;
      settings = {
        character = {
          success_symbol = "[›](bold green)";
          error_symbol = "[›](bold red)";
        };
        aws = {
          symbol = "  ";
        };
        buf = {
          symbol = " ";
        };
        c = {
          symbol = " ";
        };
        conda = {
          symbol = " ";
        };
        dart = {
          symbol = " ";
        };
        directory = {
          read_only = "󰌾";
        };
        docker_context = {
          symbol = " ";
        };
        elixir = {
          symbol = " ";
        };
        elm = {
          symbol = " ";
        };
        fossil_branch = {
          symbol = " ";
        };
        git_branch = {
          format = "[$symbol$branch(:$remote_branch)]($style) ";
          symbol = " ";
          style = "bold purple bg:0xFCA17D";
          truncation_length = 9223372036854775807;
          truncation_symbol = "…";
          only_attached = false;
          always_show_remote = false;
          ignore_branches = [ ];
          disabled = false;
        };
        git_commit = {
          commit_hash_length = 7;
          format = "[($hash$tag)]($style) ";
          style = "green bold";
          only_detached = true;
          disabled = false;
          tag_symbol = " 🏷  ";
          tag_disabled = true;
        };
        git_metrics = {
          added_style = "bold green";
          deleted_style = "bold red";
          only_nonzero_diffs = true;
          format = "([+$added]($added_style) )([-$deleted]($deleted_style) )";
          disabled = false;
        };
        git_state = {
          am = "AM";
          am_or_rebase = "AM/REBASE";
          bisect = "BISECTING";
          cherry_pick = "🍒PICKING(bold red)";
          disabled = false;
          format = "([$state( $progress_current/$progress_total)]($style)) ";
          merge = "MERGING";
          rebase = "REBASING";
          revert = "REVERTING";
          style = "bold yellow";
        };
        git_status = {
          ahead = "🏎💨$count";
          behind = "😰$count";
          conflicted = "🏳";
          deleted = "🗑";
          disabled = false;
          diverged = "😵";
          format = "([$all_status$ahead_behind]($style) )";
          ignore_submodules = false;
          modified = "📝";
          renamed = "👅";
          staged = "[++($count)](green)";
          stashed = "📦";
          style = "red bold bg:0xFCA17D";
          untracked = "🤷";
          up_to_date = "✓";
        };
        golang = {
          symbol = " ";
        };
        guix_shell = {
          symbol = " ";
        };
        haskell = {
          symbol = " ";
        };
        haxe = {
          symbol = " ";
        };
        hg_branch = {
          symbol = " ";
        };
        hostname = {
          ssh_symbol = " ";
        };
        java = {
          symbol = " ";
        };
        julia = {
          symbol = " ";
        };
        lua = {
          symbol = " ";
        };
        memory_usage = {
          symbol = "󰍛 ";
        };
        meson = {
          symbol = "󰔷 ";
        };
        nim = {
          symbol = "󰆥 ";
        };
        nix_shell = {
          symbol = " ";
          heuristic = true;
        };
        nodejs = {
          symbol = " ";
        };
        os = {
          symbols = {
            Alpaquita = " ";
            Alpine = " ";
            Amazon = " ";
            Android = " ";
            Arch = " ";
            Artix = " ";
            CentOS = " ";
            Debian = " ";
            DragonFly = " ";
            Emscripten = " ";
            EndeavourOS = " ";
            Fedora = " ";
            FreeBSD = " ";
            Garuda = "󰛓 ";
            Gentoo = " ";
            HardenedBSD = "󰞌 ";
            Illumos = "󰈸 ";
            Linux = " ";
            Mabox = " ";
            Macos = " ";
            Manjaro = " ";
            Mariner = " ";
            MidnightBSD = " ";
            Mint = " ";
            NetBSD = " ";
            NixOS = " ";
            OpenBSD = "󰈺 ";
            openSUSE = " ";
            OracleLinux = "󰌷 ";
            Pop = " ";
            Raspbian = " ";
            Redhat = " ";
            RedHatEnterprise = " ";
            Redox = "󰀘 ";
            Solus = "󰠳 ";
            SUSE = " ";
            Ubuntu = " ";
            Unknown = " ";
            Windows = "󰍲 ";
          };
        };
        package = {
          symbol = "󰏗 ";
        };
        pijul_channel = {
          symbol = " ";
        };
        python = {
          symbol = " ";
        };
        rlang = {
          symbol = "󰟔 ";
        };
        ruby = {
          symbol = " ";
        };
        rust = {
          symbol = " ";
        };
        scala = {
          symbol = " ";
        };
      };
    };
  };
}
#home.sessionVariables.STARSHIP_CACHE = "${config.xdg.cacheHome}/starship";
