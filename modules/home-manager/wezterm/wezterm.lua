-- Pull in the wezterm API
local wezterm = require 'wezterm'

-- This will hold the configuration.
local config = wezterm.config_builder()

-- This is where you actually apply your config choices
config.enable_wayland = true
config.font = wezterm.font 'BitstreamVeraSansMono'
config.color_scheme = 'dracula'

-- and finally, return the configuration to wezterm
return config