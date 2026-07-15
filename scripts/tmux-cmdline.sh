#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
binary="$root/target/release/tmux-cmdline"
command_file=$1

umask 077
: >"$command_file"

if [ ! -x "$binary" ]; then
  tmux display-message "tmux-cmdline: run cargo build --release"
  exit 0
fi

# Option lookup: @tmux-cmdline-* first, then a catppuccin @thm_* color,
# then a hard fallback. Values are format-expanded so "#{@thm_sapphire}" works.
opt() {
  value=$(tmux display-message -p "#{E:@tmux-cmdline-$1}")
  [ -n "$value" ] || value=$(tmux display-message -p "$2")
  printf '%s' "${value:-$3}"
}

accent=$(opt accent '#{@thm_blue}' cyan)
bg=$(opt bg '#{@thm_base}' terminal)
fg=$(opt fg '#{@thm_fg}' terminal)
prompt=$(opt prompt '' ' ')
title=$(opt title '' ' Tmux Cmdline ')
width=$(opt width '' '40%')
border=$(opt border '' rounded)
commands=$(
  tmux list-commands -F '#{command_list_name} #{command_list_alias}'
)
# || true: command-alias may be unset, and set -e would abort the popup.
aliases=$(tmux show-options -sv command-alias 2>/dev/null | sed 's/=.*//' || true)

# Named positions become format arithmetic so a signed row offset can shift
# them (positive = down, negative = up). -y is the popup's bottom edge.
position=$(opt position '' bottom)
offset=$(opt offset '' -6)
case $position in
top) y=$((3 + offset)) ;;
center) y="#{e|+:#{popup_centre_y},$offset}" ;;
bottom) y="#{e|+:#{popup_status_line_y},$offset}" ;;
*) y=$position ;; # raw tmux -y value passthrough
esac

tmux display-popup -E -w "$width" -h 3 -x C -y "$y" \
  -e "TCU_ACCENT=$accent" -e "TCU_PROMPT=$prompt" \
  -e "TCU_COMMANDS=$commands" \
  -e "TCU_ALIASES=$aliases" \
  -b "$border" \
  -s "bg=$bg,fg=$fg" \
  -S "fg=$accent,bg=$bg" \
  -T "#[align=centre]#[fg=$accent,bold]$title" \
  "$binary" "$command_file"
