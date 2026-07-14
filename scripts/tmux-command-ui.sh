#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
binary="$root/target/release/tmux-command-ui"
command_file=$1

umask 077
: >"$command_file"

if [ ! -x "$binary" ]; then
    tmux display-message "tmux-command-ui: run cargo build --release"
    exit 0
fi

tmux display-popup -E -w 60% -h 3 -x C -y S -T Cmdline \
    "$binary" "$command_file"
