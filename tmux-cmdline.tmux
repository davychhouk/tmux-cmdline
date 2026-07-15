#!/bin/sh
set -eu

root=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
repo=davychhouk/tmux-cmdline
version=$(tmux show-option -gqv @tmux-cmdline-version)
version=${version:-v0.1.0}
binary="$root/target/release/tmux-cmdline"
version_file="$root/target/release/.version"

fail() {
    tmux display-message "tmux-cmdline: $*" 2>/dev/null ||
        printf 'tmux-cmdline: %s\n' "$*" >&2
    exit 1
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
        wget -q -T 30 -O "$2" "$1"
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

installed=$(cat "$version_file" 2>/dev/null || true)
if [ ! -x "$binary" ] || { [ -n "$installed" ] && [ "$installed" != "$version" ]; }; then
    mkdir -p "$(dirname "$binary")" || fail "could not create binary directory"
    if ! fetch; then
        command -v cargo >/dev/null 2>&1 ||
            fail "no prebuilt binary for $version; install Rust or select an available @tmux-cmdline-version"
        (cd "$root" && cargo build --release) || fail "cargo build failed"
    fi
    printf '%s\n' "$version" >"$version_file"
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
