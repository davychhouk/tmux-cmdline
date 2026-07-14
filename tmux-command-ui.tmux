#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
command_dir=$(tmux show-option -gvq @tmux-command-ui-dir)
if [ ! -d "$command_dir" ]; then
    command_dir=$(mktemp -d "${TMPDIR:-/tmp}/tmux-command-ui.XXXXXX")
    chmod 700 "$command_dir"
    tmux set-option -gq @tmux-command-ui-dir "$command_dir"
fi
command_file="$command_dir/#{client_pid}"

tmux bind-key : \
    run-shell "$root/scripts/tmux-command-ui.sh '$command_file'" '\;' \
    run-shell -C "source-file '$command_file'" '\;' \
    run-shell "rm -f '$command_file'"
