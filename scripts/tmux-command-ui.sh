#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
binary="$root/target/release/tmux-command-ui"

if [ ! -x "$binary" ]; then
    tmux display-message "tmux-command-ui: run cargo build --release"
    exit 1
fi

command_file=$(mktemp "${TMPDIR:-/tmp}/tmux-command-ui.XXXXXX")
trap 'rm -f "$command_file"' EXIT HUP INT TERM

tmux display-popup -E -w 60% -h 3 -x C -y S -T Cmdline \
    "$binary" "$command_file"

if [ -s "$command_file" ] && ! error=$(tmux source-file "$command_file" 2>&1); then
    tmux display-message "tmux-command-ui: $error"
fi
