#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
repo=davychhouk/tmux-cmdline
requested=$(tmux show-option -gqv @tmux-cmdline-version)
version=${requested:-v0.1.1}
binary="$root/target/release/tmux-cmdline"
version_file="$root/target/release/.version"
failure_file="$root/target/release/.failed-version"
commit=$(git -C "$root" rev-parse HEAD 2>/dev/null || printf unknown)
source_version="source:$commit"
attempt="$version:$commit"

warn() {
    tmux display-message "tmux-cmdline: $*" 2>/dev/null ||
        printf 'tmux-cmdline: %s\n' "$*" >&2
}

remember_failure() {
    printf '%s\n' "$attempt" >"$failure_file" || true
}

target() {
    case "$(uname -s):$(uname -m)" in
    Darwin:arm64 | Darwin:aarch64) printf 'aarch64-apple-darwin' ;;
    Darwin:x86_64) printf 'x86_64-apple-darwin' ;;
    Linux:arm64 | Linux:aarch64) printf 'aarch64-unknown-linux-gnu' ;;
    Linux:x86_64 | Linux:amd64) printf 'x86_64-unknown-linux-gnu' ;;
    *) return 1 ;;
    esac
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --max-time 30 "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        # --tries caps GNU wget's 20-retry default; BusyBox wget lacks it and
        # already makes a single attempt.
        if wget --help 2>&1 | grep -q -- --tries; then
            wget -q -T 30 --tries=1 -O "$2" "$1"
        else
            wget -q -T 30 -O "$2" "$1"
        fi
    else
        return 1
    fi
}

checksum() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 -c "$1"
    elif command -v sha256sum >/dev/null 2>&1; then
        sha256sum -c "$1"
    else
        return 1
    fi
}

fetch() {
    target=$(target) || return 1
    asset="tmux-cmdline-$target.tar.gz"
    url="https://github.com/$repo/releases/download/$version/$asset"
    tmp=$(mktemp -d "${TMPDIR:-/tmp}/tmux-cmdline-install.XXXXXX") || return 1
    trap 'rm -rf "$tmp"' EXIT

    download "$url" "$tmp/$asset" || return 1
    download "$url.sha256" "$tmp/$asset.sha256" || return 1
    (cd "$tmp" && checksum "$asset.sha256") >/dev/null || return 1
    tar -xzf "$tmp/$asset" -C "$(dirname "$binary")"
}

# Best-effort: on failure warn but keep going so the key still binds and any
# existing binary keeps working; the popup script re-checks at press time.
install() {
    mkdir -p "$(dirname "$binary")" || { warn "could not create binary directory"; return 1; }
    if fetch; then
        installed=$version
        rm -f "$failure_file"
    elif command -v cargo >/dev/null 2>&1 && { [ -z "$requested" ] || [ ! -x "$binary" ]; }; then
        (cd "$root" && cargo build --release) || { warn "cargo build failed"; return 1; }
        installed=$source_version
        remember_failure
        warn "using a source build because the $version binary could not be installed"
    elif [ -x "$binary" ]; then
        remember_failure
        warn "could not install $version; keeping the existing binary"
        return 1
    elif target >/dev/null 2>&1; then
        warn "could not download or verify the $version binary; check your network or install Rust"
        return 1
    else
        warn "no prebuilt binary for $(uname -s)/$(uname -m); install Rust or select an available @tmux-cmdline-version"
        return 1
    fi
    printf '%s\n' "$installed" >"$version_file"
}

# Verified downloads use the release tag; Cargo fallbacks use their source
# commit. An empty marker belongs to a user-managed local build.
installed=$(cat "$version_file" 2>/dev/null || true)
failed=$(cat "$failure_file" 2>/dev/null || true)
if [ ! -x "$binary" ]; then
    install || true
elif [ -n "$requested" ] && [ "$installed" != "$version" ] && [ "$failed" != "$attempt" ]; then
    install || true
elif [ -z "$requested" ] && [ -n "$installed" ] &&
    [ "$installed" != "$version" ] && [ "$installed" != "$source_version" ] &&
    [ "$failed" != "$attempt" ]; then
    install || true
fi

command_dir=$(tmux show-option -gvq @tmux-cmdline-dir)
if [ ! -d "$command_dir" ]; then
    command_dir=$(mktemp -d "${TMPDIR:-/tmp}/tmux-cmdline.XXXXXX")
    chmod 700 "$command_dir"
    tmux set-option -gq @tmux-cmdline-dir "$command_dir"
fi
command_file="$command_dir/#{client_pid}"

tmux bind-key : \
    run-shell "$root/scripts/tmux-cmdline.sh '$command_file'" '\;' \
    run-shell -C "source-file '$command_file'" '\;' \
    run-shell "rm -f '$command_file'"
