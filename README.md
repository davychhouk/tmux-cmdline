# tmux-cmdline

A themeable popup replacement for tmux's `prefix + :` command prompt.
Catppuccin-aware, configured entirely through tmux options — no rebuild to
retheme.

<img width="1512" height="947" alt="image" src="https://github.com/user-attachments/assets/74386dc3-8835-4c06-9746-0130e51bc6c1" />

## Requirements

- tmux 3.3 or newer
- `curl` or `wget` to download a prebuilt binary, or a Rust toolchain as a
  fallback
- A [Nerd Font](https://www.nerdfonts.com/) for the default prompt glyph
  (or set your own with `@tmux-cmdline-prompt`)

## Install

### TPM

Add the plugin to `~/.tmux.conf` and install with `prefix + I`:

```tmux
set -g @plugin 'davychhouk/tmux-cmdline'
```

The plugin downloads and verifies the `v0.1.0` prebuilt binary on first load.
If that fails, it falls back to `cargo build --release` when Rust is installed.
Set `@tmux-cmdline-version` to select another release.

### Manual

```sh
git clone https://github.com/davychhouk/tmux-cmdline
cd tmux-cmdline && cargo build --release
```

Load the plugin from `~/.tmux.conf` and reload tmux:

```tmux
run-shell "/absolute/path/to/tmux-cmdline/tmux-cmdline.tmux"
```

## Development

Build and load the current checkout from the repository root:

```sh
cargo build --release
tmux run-shell "$PWD/tmux-cmdline.tmux"
```

Press `prefix + :` to test it. Rebuild after Rust changes; rerun `run-shell`
after changing the tmux or shell scripts.

## Usage

Press `prefix + :` to open the popup.

| Key               | Action                  |
| ----------------- | ----------------------- |
| `Enter`           | Execute the command     |
| `Esc` / `Ctrl-c`  | Cancel                  |
| `Ctrl-a` / `Home` | Start of line           |
| `Ctrl-e` / `End`  | End of line             |
| `Ctrl-u`          | Delete to start of line |
| `Ctrl-k`          | Delete to end of line   |
| `Ctrl-w`          | Delete previous word    |

The command is parsed in the invoking client's command queue after the popup
closes, so quoting, command sequences, and interactive tmux commands retain
their native behavior.

## Theming

Colors default to catppuccin's `@thm_*` palette when present, otherwise to
plain terminal colors. Override any of these in `~/.tmux.conf` (values are
format-expanded, so `#{@thm_*}` references work):

```tmux
set -g @tmux-cmdline-accent "#{@thm_blue}"      # border, title, prompt
set -g @tmux-cmdline-bg     "#{@thm_base}"      # popup background
set -g @tmux-cmdline-fg     "#{@thm_fg}"        # input text
set -g @tmux-cmdline-prompt " "                # prompt glyph
set -g @tmux-cmdline-title  " Tmux Cmdline "    # popup title (centred)
set -g @tmux-cmdline-width  "40%"               # popup width
set -g @tmux-cmdline-position "bottom"          # top | center | bottom (or raw -y value)
set -g @tmux-cmdline-offset   "-6"              # rows to shift position: negative = up
set -g @tmux-cmdline-border "rounded"           # popup border lines
```

## Positioning

`position` picks a named anchor; `offset` shifts it by signed rows (negative =
up, positive = down):

```tmux
set -g @tmux-cmdline-position "bottom"
set -g @tmux-cmdline-offset   "-6"    # 6 rows above the usual bottom spot
```

Named anchors compile to tmux format arithmetic — e.g. `bottom` with offset
`-6` becomes `-y '#{e|+:#{popup_status_line_y},-6}'` — so tmux does the math
at display time and the popup stays put when the terminal resizes.

Anything other than `top`/`center`/`bottom` is passed to `display-popup -y`
verbatim, so any raw value or format expression works:

```tmux
set -g @tmux-cmdline-position "#{e|/:#{client_height},4}"  # quarter height
```

Raw values are expanded when the popup opens, before `display-popup` runs, so
`#{popup_*}` variables are not available in them — use `client_height` math
or the named anchors instead.

## License

MIT — see [LICENSE](LICENSE).
