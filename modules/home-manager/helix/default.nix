{ lib, config, pkgs, ... }:

with lib;
let cfg = config.helix;

in {
  options.helix.enable = lib.mkEnableOption "Enable helix";
  config = mkIf cfg.enable {
    programs.helix = {
      enable = true;
      defaultEditor = true;
        settings = {
          theme = lib.mkForce "tokyonight_storm";
          editor = {
            lsp.display-messages = true;
            cursor-shape = {
              insert = "bar";
              normal = "block";
              select = "underline";
            };
            file-picker = {
              hidden = false;
            };
            auto-save = false;
            completion-trigger-len = 1;
            idle-timeout = 0;
            mouse = true;
          };
          
          keys.normal = {
            # VS Code-like keybindings for normal mode
            "C-s" = ":w"; # Save
            "C-S-s" = ":wa"; # Save all
            "C-z" = "u"; # Undo
            "C-S-z" = "U"; # Redo
            "C-y" = "U"; # Redo (alternative)
            "C-x" = "d"; # Cut
            "C-c" = "y"; # Copy
            "C-v" = "p"; # Paste
            "C-a" = "%"; # Select all
            "C-f" = "/"; # Find
            "C-h" = ":s"; # Find and replace
            "C-g" = "g"; # Go to line
            "C-d" = ["C-d" "c"]; # Duplicate line
            "C-/" = "gcc"; # Toggle comment
            "C-S-k" = "dd"; # Delete line
            "C-p" = "file_picker"; # Command palette / file picker
            "C-S-p" = "command_palette"; # Command palette
            "C-o" = "file_picker"; # Open file
            "C-n" = ":n"; # New file
            "C-w" = ":bc"; # Close buffer
            "C-S-t" = ":buffer-previous"; # Reopen closed tab
            "C-PageDown" = ":bn"; # Next buffer
            "C-PageUp" = ":bp"; # Previous buffer
            "C-b" = "b"; # Toggle sidebar (file tree)
            "F2" = "r"; # Rename
            "F3" = "n"; # Find next
            "S-F3" = "N"; # Find previous
            "F12" = "gd"; # Go to definition
            "C-F12" = "gi"; # Go to implementation
            "C-space" = "completion"; # Trigger completion
            "C-S-up" = ["v" "k" "d" "k" "P"]; # Move line up (Ctrl+Shift+Up)
            "C-S-down" = ["v" "j" "d" "p"]; # Move line down (Ctrl+Shift+Down)
            "A-up" = ["v" "k" "d" "k" "P"]; # Alt + up
            "A-down" = ["v" "j" "d" "p"]; # Alt + down
            "tab" = "indent"; # Indent
            "S-tab" = "unindent"; # Unindent
            "escape" = "normal_mode"; # Escape to normal mode
            "C-backspace" = "delete_word_backward"; # Delete word backward
            "C-delete" = "delete_word_forward"; # Delete word forward
            "home" = "goto_line_start"; # Home key
            "end" = "goto_line_end"; # End key
            "C-home" = "goto_file_start"; # Ctrl+Home
            "C-end" = "goto_file_end"; # Ctrl+End
            "C-left" = "move_prev_word_start"; # Ctrl+Left
            "C-right" = "move_next_word_end"; # Ctrl+Right
            "C-l" = "goto_line"; # Go to line
            "C-e" = ":e"; # Open recent files
            "F1" = ":help"; # Help
            "C-k" = "hover"; # Show hover documentation
            "C-S-o" = "symbol_picker"; # Go to symbol
            "C-t" = "goto_type_definition"; # Go to type definition
            "C-S-f" = "search"; # Search in files
            "C-S-h" = "replace"; # Replace in files
            "C-`" = ":sh"; # Open terminal
            "C-S-`" = ":new"; # New terminal
            "C-," = ":config-open"; # Open settings
            "C-k-C-s" = ":config-open"; # Open keyboard shortcuts
            "C-S-e" = "file_picker"; # Show explorer
            "C-S-g" = ":sh git status"; # Git status
            "C-S-m" = "expand_selection"; # Expand selection
            "A-S-f" = ":format"; # Format document
            "F8" = "goto_next_diag"; # Next problem
            "S-F8" = "goto_prev_diag"; # Previous problem
            "C-." = "code_action"; # Quick fix
            "F5" = ":sh"; # Run/Debug (terminal)
            "C-F5" = ":sh"; # Run without debugging
            "S-F5" = ":reload"; # Restart
            "C-S-d" = ":debug"; # Debug console
            "C-k-C-i" = "hover"; # Show hover
            "A-F12" = "goto_reference"; # Peek definition
            "S-F12" = "goto_reference"; # Go to references
            "C-=" = ":zoom-in"; # Zoom in
            "C--" = ":zoom-out"; # Zoom out
            "C-0" = ":zoom-reset"; # Reset zoom
            "C-S-[" = ":fold"; # Fold
            "C-S-]" = ":unfold"; # Unfold
            "C-k-C-0" = ":fold-all"; # Fold all
            "C-k-C-j" = ":unfold-all"; # Unfold all
            "C-k-C-/" = ":fold-comment"; # Fold all block comments
            "A-z" = ":toggle-word-wrap"; # Toggle word wrap
          };
          
          keys.insert = {
            # VS Code-like keybindings for insert mode
            "C-s" = ":w"; # Save
            "C-z" = "undo"; # Undo
            "C-y" = "redo"; # Redo
            "C-space" = "completion"; # Trigger completion
            "C-backspace" = "delete_word_backward"; # Delete word backward
            "C-delete" = "delete_word_forward"; # Delete word forward
            "tab" = "indent"; # Tab
            "S-tab" = "unindent"; # Shift+Tab
            "C-v" = "paste_after"; # Paste
            "C-x" = "delete_selection"; # Cut
            "C-c" = "yank"; # Copy
            "C-a" = "select_all"; # Select all
            "escape" = "normal_mode"; # Escape to normal mode
            "C-n" = "completion"; # Auto-complete
            "C-p" = "completion"; # Auto-complete (previous)
            "C-enter" = ["normal_mode" "o"]; # Insert line below
            "C-S-enter" = ["normal_mode" "O"]; # Insert line above
            "A-up" = ["normal_mode" "v" "k" "d" "k" "P" "i"]; # Move line up
            "A-down" = ["normal_mode" "v" "j" "d" "p" "i"]; # Move line down
            "home" = "goto_line_start"; # Home
            "end" = "goto_line_end"; # End
            "C-home" = "goto_file_start"; # Ctrl+Home
            "C-end" = "goto_file_end"; # Ctrl+End
            "C-left" = "move_prev_word_start"; # Ctrl+Left
            "C-right" = "move_next_word_end"; # Ctrl+Right
            "C-d" = ["normal_mode" "yy" "p" "i"]; # Duplicate line
            "C-/" = ["normal_mode" "gcc" "i"]; # Toggle comment
            "C-S-space" = ["normal_mode" "hover" "i"]; # Show hover (Ctrl+Shift+Space)
            "C-S-k" = ["normal_mode" "dd" "i"]; # Delete line
            "F1" = ["normal_mode" ":help" "i"]; # Help
          };
          
          keys.select = {
            # VS Code-like keybindings for visual/select mode
            "C-c" = "y"; # Copy
            "C-x" = "d"; # Cut
            "C-v" = "p"; # Paste
            "tab" = "indent"; # Indent selection
            "S-tab" = "unindent"; # Unindent selection
            "C-/" = "toggle_comments"; # Toggle comment
            "C-d" = ["y" "P"]; # Duplicate selection
            "escape" = "normal_mode"; # Escape to normal mode
            "C-a" = "select_all"; # Select all
            "C-f" = "/"; # Find in selection
            "C-h" = ":s"; # Replace in selection
            "A-up" = ["d" "k" "P"]; # Move selection up
            "A-down" = ["d" "p"]; # Move selection down
            "C-S-k" = "d"; # Delete selection
            "C-g" = "goto_line"; # Go to line
            "home" = "goto_line_start"; # Home
            "end" = "goto_line_end"; # End
            "C-home" = "goto_file_start"; # Ctrl+Home
            "C-end" = "goto_file_end"; # Ctrl+End
          };
        };

        languages = {
          rust = {
            enable = true;
            settings = {
              auto-format = true;
              rust-analyzer = {
                enable = true;
              };
            };
          };
          bash.enable = true;
          css.enable = true;
          docker-compose.enable = true;
          dockerfile.enable = true;
          env.enable = true;
          go.enable = true;
          graphql.enable = true;
          helm.enable = true;
          html.enable = true;
          javascript.enable = true;
          jq.enable = true;
          json.enable = true;
          jsonc.enable = true;
          just.enable = true;
          latex.enable = true;
          llvm.enable = true;
          log.enable = true;
          lua.enable = true;
          markdown.enable = true;
          mermaid.enable = true;
          nginx.enable = true;
          nix = {
            enable = true;
            auto-format = true;
          };
          python.enable = true;
          regex.enable = true;
          scss.enable = true;
          sql.enable = true;
          toml.enable = true;
          typescript.enable = true;
          wgsl.enable = true;
          wit.enable = true;
          xml.enable = true;
          yaml.enable = true;
          yuck.enable = true;
          zig.enable = true;
        };
    };
  };
}