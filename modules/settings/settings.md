# Settings

Settings are plain nix files that return attribute sets.

These are NOT modules.

They are used for text files and attribute sets of definitions that can be imported.

These should have more descriptive names than just the attribute they set, such as a key definition. raw test should have a .txt extension so we know it's not a nix file.

These are used to set configurations dynamically on rebuild. change the import value pointing to the configuration and the resulting sets change.

This is primarily for things like "dot files" or specific configurations we aren't entrusting to nix to convert for us.

Most things should be attribute sets, text is only a fallback when nix conversion does not work.