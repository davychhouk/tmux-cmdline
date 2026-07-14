#!/bin/sh

root=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
tmux bind-key : run-shell "$root/scripts/tmux-command-ui.sh"
