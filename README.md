# tmux-cmdline

Popup replacement for tmux's `prefix + :` command prompt, with syntax
highlighting, tab completion, and history. Themed through tmux options;
Catppuccin-aware.

<img width="1511" height="949" alt="image" src="https://github.com/user-attachments/assets/8c5df83f-446a-4bed-9104-fec108aa53e7" />

## Features

- **Popup prompt** — replaces `prefix + :` with a floating input; commands
  run with tmux's native parsing, so quoting and sequences behave as usual.
- **Syntax highlighting** — commands, flags, strings, variables, formats, and
  comments are colored as you type.
- **Command validation** — command names that are unknown or an ambiguous
  prefix are shown in red before you run them; aliases count as known.
- **Readline-style editing** — `Ctrl-a/e/u/k/w`, arrows, `Home`/`End`, with
  Unicode-aware cursor movement and scrolling.
- **Tab completion** — `Tab` completes command names and aliases, extending
  to the common prefix, then cycling through matches.
- **Command history** — `Up`/`Down` (or `Ctrl-p`/`Ctrl-n`) cycle past
  commands, filtered by what you've typed, persisted across popups and tmux
  restarts.
- **Alerts** — command output and errors appear in a notification alert.
- **Theming** — Catppuccin-aware colors, prompt, title, border, and width via
  tmux options.
- **Positioning** — top/center/bottom anchors with a row offset, or any raw
  `display-popup -y` value.
- **Prebuilt binaries** — for macOS and Linux (x86_64, arm64), downloaded and
  checksum-verified on first load, with a Cargo fallback.

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

The plugin downloads and verifies the `v0.3.0` prebuilt binary on first load.
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
after changing the tmux or shell scripts. Run the unit tests with `cargo test`.

## Usage

Press `prefix + :` to open the popup.

| Key               | Action                  |
| ----------------- | ----------------------- |
| `Enter`           | Execute the command     |
| `Esc` / `Ctrl-c`  | Cancel                  |
| `Ctrl-a` / `Home` | Start of line           |
| `Ctrl-e` / `End`  | End of line             |
| `Left` / `Right`  | Move cursor             |
| `Backspace`       | Delete previous char    |
| `Delete`          | Delete next char        |
| `Ctrl-u`          | Delete to start of line |
| `Ctrl-k`          | Delete to end of line   |
| `Ctrl-w`          | Delete previous word    |
| `Tab`             | Complete command name   |
| `Shift-Tab`       | Previous completion     |
| `Up` / `Ctrl-p`   | Previous history entry  |
| `Down` / `Ctrl-n` | Next history entry      |

The command is parsed in the invoking client's command queue after the popup
closes, so quoting, command sequences, and interactive tmux commands retain
their native behavior.

`Tab` completes the command name at the cursor: at the start of the line or
after a `;` or `{` separator. A unique match is completed with a trailing
space; several matches extend to their common prefix, then `Tab` / `Shift-Tab`
cycle through them. Arguments are not completed.

History navigation only shows entries starting with the text typed before the
first `Up`; with nothing typed, it walks all history.

Submitted commands are saved (last 100, empty and consecutive duplicates
skipped) to `$XDG_STATE_HOME/tmux-cmdline/history`, falling back to
`~/.local/state/tmux-cmdline/history`. Override the path with:

```tmux
set -g @tmux-cmdline-history-file "$HOME/.tmux-cmdline-history"
```

## Theming

Colors default to catppuccin's `@thm_*` palette when present, otherwise to
plain terminal colors. Override any of these in `~/.tmux.conf` (values are
format-expanded, so `#{@thm_*}` references work):

```tmux
set -g @tmux-cmdline-accent "#{@thm_blue}"      # border, title, prompt, command names
set -g @tmux-cmdline-bg     "#{@thm_base}"      # popup background
set -g @tmux-cmdline-fg     "#{@thm_fg}"        # input text
set -g @tmux-cmdline-prompt " "                # prompt glyph
set -g @tmux-cmdline-title  " Tmux Cmdline "    # popup title (centred)
set -g @tmux-cmdline-width  "40%"               # popup width
set -g @tmux-cmdline-position "bottom"          # top | center | bottom (or raw -y value)
set -g @tmux-cmdline-offset   "-6"              # rows to shift position: negative = up
set -g @tmux-cmdline-border "rounded"           # popup border lines
```

## Alerts

Messages and errors from a command run through the popup (e.g.
`can't find window`, `display-message` output, parse errors) are shown in a
notification alert in the top-right corner. It closes after a timeout or on any
key.

```tmux
set -g @tmux-cmdline-alert         "on"            # off keeps tmux's status-line message only
set -g @tmux-cmdline-alert-timeout "3000"          # ms, max 25500
set -g @tmux-cmdline-alert-error   "#{@thm_red}"   # border/title color on error
set -g @tmux-cmdline-alert-info    "#{@thm_blue}"  # border/title color otherwise
set -g @tmux-cmdline-alert-min-width  "40"         # cols or % of client width
set -g @tmux-cmdline-alert-max-width  "80"         # cols or %; longer lines wrap
set -g @tmux-cmdline-alert-max-height "10"         # rows or %; extra lines are cut
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
