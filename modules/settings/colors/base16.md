# Base 16 Color Scheme and Theming

Colors follow Base16 for a consistent way to generate our models

The Base 16 scheme is intended to assign specific identities to common functions.

There is an extension called Tinting that increases this to 32 colors, but the extra 16 are derived from the initial 16 and are used for light/dark mode preferences.

These colors can be extracted or generated easily and form a common pallete to use when assigning theme colors for consistency. All named palettes should include both dark/light versions, because your users will expect it.

The CIM32 schema is adapted from aggregating the following: 
  - [https://github.com/chriskempson/base16/blob/main/styling.md]
  - [https://stylix.danth.me/styling.html]
  - [https://github.com/tajmone/Base16-Sass]
  - [https://github.com/tinted-theming/home]

This is a map from base16 to BehaviorColor
and defaults to "light mode"
"dark mode" is applied after light mode as an overlay

# Original Intended Behavior:
    base00 - Default Background
    base01 - Lighter Background (Used for status bars, line number and folding marks)
    base02 - Selection Background
    base03 - Comments, Invisibles, Line Highlighting
    base04 - Dark Foreground (Used for status bars)
    base05 - Default Foreground, Caret, Delimiters, Operators
    base06 - Light Foreground (Not often used)
    base07 - Brightest Foreground (Not often used)
    base08 - Variables, XML Tags, Markup Link Text, Markup Lists, Diff Deleted
    base09 - Integers, Boolean, Constants, XML Attributes, Markup Link Url
    base0A - Classes, Markup Bold, Search Text Background
    base0B - Strings, Inherited Class, Markup Code, Diff Inserted
    base0C - Support, Regular Expressions, Escape Characters, Markup Quotes
    base0D - Functions, Methods, Attribute IDs, Headings
    base0E - Keywords, Storage, Selector, Markup Italic, Diff Changed
    base0F - Deprecated, Opening/Closing Embedded Language Tags, e.g. <?php ?>


# We interpret this as:
base00: "bg"
base01: "bg-light"
base02: "bg-selected"
base03: "comment"
base04: "color-dark"
base05: "color"
base06: "color-light"
base07: "color-bright"
base08: "text-var"
base09: "text-num"
base0A: "text-bold"
base0B: "text-bright"
base0C: "quote"
base0D: "action"
base0E: "const"
base0F: "text-dim"

# Base BehaviorColor
bg-default:   base00
bg-alt:       base01
bg-selected:  base02
text-default: base05
text-alt:     base04
bg-warning:   base0A
bg-urgent:    base09
bg-error:     base08

border-win-unfocused: base03
border-win-focused: base0D
border-win-unfocused: base03
border-win-urgent: base08
text-win-title: base05

### Notifications
border-notify: base0D
bg-low-priority: base06
text-low-priority: base0A
bg-high-priority: base0F
text-high-priority: base08
bg-progress: base01
color-progress: base02

## DARK Mode
Dark mode has some different setting to ensure contrast and readability

text-default: base00
text-alt: base01
color-onbg: base0E
color-offbg: base0D
color-onbg-alt: base09
color-offbg-alt: base02
bg-unselected: base0D
bg-selected: base03


