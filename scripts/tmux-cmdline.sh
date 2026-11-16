#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
binary="$root/target/release/tmux-cmdline"

# Option lookup: @tmux-cmdline-* first, then a catppuccin @thm_* color,
# then a hard fallback. Values are format-expanded so "#{@thm_sapphire}" works.
opt() {
  value=$(tmux display-message -p "#{E:@tmux-cmdline-$1}")
  [ -n "$value" ] || value=$(tmux display-message -p "$2")
  printf '%s' "${value:-$3}"
}

bg=$(opt bg '#{@thm_base}' terminal)
fg=$(opt fg '#{@thm_fg}' terminal)
border=$(opt border '' rounded)

if [ "$1" = alert ]; then
  command_file=$2
  client=$3
  # The success marker deletes the file, so it surviving means the command failed.
  level=info
  [ ! -e "$command_file" ] || level=error
  rm -f "$command_file"
  [ "$(opt alert '' on)" != off ] || exit 0

  # The log is newest first: keep this client's messages since the popup opened.
  messages=$(tmux show-messages | awk -v start="$binary $command_file" -v tag=" $client message: " '
    index($0, start) { exit }
    i = index($0, tag) { m[n++] = substr($0, i + length(tag)) }
    END { while (n) print m[--n] }')
  [ -n "$messages" ] || exit 0

  case $level in
  error) color=$(opt alert-error '#{@thm_red}' red) ;;
  *) color=$(opt alert-info '#{@thm_blue}' cyan) ;;
  esac
  cols=$(tmux display-message -p -c "$client" '#{client_width}')
  rows=$(tmux display-message -p -c "$client" '#{client_height}')
  # Sizes are cells or a percentage of $2, clamped to [$3, $2].
  size() {
    case $1 in *%) n=$(($2 * ${1%\%} / 100)) ;; *) n=$1 ;; esac
    [ "$n" -le "$2" ] || n=$2
    [ "$n" -ge "$3" ] || n=$3
    printf '%s' "$n"
  }
  max=$(size "$(opt alert-max-width '' 80)" "$cols" 8)
  min=$(size "$(opt alert-min-width '' 40)" "$cols" 1)
  [ "$min" -le "$max" ] || min=$max
  max_h=$(size "$(opt alert-max-height '' 10)" "$rows" 3)
  body=$(printf '%s\n' "$messages" | fold -s -w "$((max - 4))" | sed 's/^/ /' | head -n "$((max_h - 2))")
  w=$(printf '%s\n' "$body" | awk -v n="$min" '{ if (length + 3 > n) n = length + 3 } END { print n }')
  h=$(($(printf '%s\n' "$body" | wc -l) + 2))
  # ponytail: stty time is in deciseconds and caps at 25.5s.
  timeout=$(($(opt alert-timeout '' 3000) / 100))
  [ "$timeout" -le 255 ] || timeout=255
  [ "$timeout" -ge 1 ] || timeout=1

  # The alert replaces the status message; while one is shown tmux won't paint popup output.
  tmux display-message -c "$client" -d 1 ''
  # -y is the popup's bottom edge: h + 1 leaves one row above it.
  # Separate args run directly, not via default-shell (which may be fish).
  # shellcheck disable=SC2016 # expanded by the popup's sh
  tmux display-popup -c "$client" -E -w "$w" -h "$h" -x R -y "$((h + 1))" \
    -e "TCU_BODY=$body" -e "TCU_TIMEOUT=$timeout" \
    -b "$border" \
    -s "bg=$bg,fg=$fg" \
    -S "fg=$color,bg=$bg" \
    -T "#[align=centre]#[fg=$color,bold] $level " \
    sh -c 'printf "\033[?25l%s" "$TCU_BODY"; stty -icanon -echo min 0 time "$TCU_TIMEOUT"; dd bs=1 count=1 >/dev/null 2>&1'
  exit 0
fi

command_file=$1
client=${2-}

umask 077
: >"$command_file"

if [ ! -x "$binary" ]; then
  tmux display-message "tmux-cmdline: run cargo build --release"
  exit 0
fi

accent=$(opt accent '#{@thm_blue}' cyan)
prompt=$(opt prompt '' ' ')
title=$(opt title '' ' Tmux Cmdline ')
width=$(opt width '' '40%')
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

[ -s "$command_file" ] || exit 0
# Parse errors would open a view-mode pane; report them as a message instead.
if ! error=$(tmux source-file -n "$command_file" 2>&1); then
  tmux display-message -c "$client" "${error#"$command_file":[0-9]*: }"
  : >"$command_file"
  exit 0
fi
# An error skips the rest of its line, so this marker runs only on success.
# ponytail: a trailing "# comment" swallows the marker and misreports success as error.
line=$(cat "$command_file")
printf '%s ; run-shell "rm -f '\''%s'\''"\n' "$line" "$command_file" >"$command_file"
