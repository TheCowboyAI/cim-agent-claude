#!/usr/bin/env bash

# Simply launch cursor without overriding settings
# This preserves user's home directory settings
appimage-run --wrap-type=2 -- cursor "$@" 
