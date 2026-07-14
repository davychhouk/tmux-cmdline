# tmux-command-ui

A popup replacement for tmux's `prefix + :` command prompt.

## Requirements

- tmux 3.3 or newer
- A current stable Rust toolchain

## Install

Build the binary:

```sh
cargo build --release
```

Load the plugin from `~/.tmux.conf`:

```tmux
run-shell "/absolute/path/to/tmux-command-ui/tmux-command-ui.tmux"
```

Reload tmux, then press `prefix + :`. Press Enter to execute, or Escape to
cancel.

The command is parsed in the invoking client's command queue after the popup
closes, so quoting, command sequences, and interactive tmux commands retain
their native behavior.

## Theming

Colors default to catppuccin's `@thm_*` palette when present, otherwise to
plain terminal colors. Override any of these in `~/.tmux.conf` (values are
format-expanded, so `#{@thm_*}` references work):

```tmux
set -g @tmux-command-ui-accent "#{@thm_blue}"      # border, title, prompt
set -g @tmux-command-ui-bg     "#{@thm_base}"      # popup background
set -g @tmux-command-ui-fg     "#{@thm_fg}"        # input text
set -g @tmux-command-ui-prompt " "                # prompt glyph
set -g @tmux-command-ui-title  " Tmux Cmdline "    # popup title (centred)
set -g @tmux-command-ui-width  "40%"               # popup width
set -g @tmux-command-ui-position "bottom"          # top | center | bottom (or raw -y value)
set -g @tmux-command-ui-offset   "-6"              # rows to shift position: negative = up
set -g @tmux-command-ui-border "rounded"           # popup border lines
```

## Positioning

`position` picks a named anchor; `offset` shifts it by signed rows (negative =
up, positive = down):

```tmux
set -g @tmux-command-ui-position "bottom"
set -g @tmux-command-ui-offset   "-6"    # 6 rows above the usual bottom spot
```

Named anchors compile to tmux format arithmetic — e.g. `bottom` with offset
`-6` becomes `-y '#{e|+:#{popup_status_line_y},-6}'` — so tmux does the math
at display time and the popup stays put when the terminal resizes.

Anything other than `top`/`center`/`bottom` is passed to `display-popup -y`
verbatim, so any raw value or format expression works:

```tmux
set -g @tmux-command-ui-position "#{e|/:#{client_height},4}"  # quarter height
```

Raw values are expanded when the popup opens, before `display-popup` runs, so
`#{popup_*}` variables are not available in them — use `client_height` math
or the named anchors instead.
