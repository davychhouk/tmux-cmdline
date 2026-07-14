# tmux-command-ui

A popup replacement for tmux's `prefix + :` command prompt.

## Requirements

- tmux 3.2 or newer
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

The command is parsed by tmux after the popup closes, so quoting, command
sequences, and interactive tmux commands retain their native behavior.
