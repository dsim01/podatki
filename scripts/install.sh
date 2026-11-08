#!/bin/sh
# Installs the latest podatki release to ~/.local/bin:
#   curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/install.sh | sh
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
FISH_COMPLETIONS="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/podatki.fish"

case "$(uname -m)" in
    x86_64 | amd64) target=x86_64-unknown-linux-gnu ;;
    aarch64 | arm64) target=aarch64-unknown-linux-gnu ;;
    *) echo "Непідтримувана архітектура: $(uname -m)" >&2; exit 1 ;;
esac

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

curl -fsSL "https://github.com/dsim01/podatki/releases/latest/download/podatki-$target.tar.gz" \
    | tar -xz -C "$tmp" --strip-components=1
mkdir -p "$BIN_DIR"
cp "$tmp/podatki" "$BIN_DIR/podatki"
chmod 755 "$BIN_DIR/podatki"
echo "Встановлено: $BIN_DIR/podatki"

if command -v fish >/dev/null 2>&1; then
    mkdir -p "$(dirname "$FISH_COMPLETIONS")"
    echo 'COMPLETE=fish podatki | source' >"$FISH_COMPLETIONS"
    echo "Автодоповнення fish: $FISH_COMPLETIONS"
fi

echo
echo "Автодоповнення для zsh/bash (за потреби, один раз):"
echo "  zsh:  echo 'source <(COMPLETE=zsh podatki)' >> ~/.zshrc"
echo "  bash: echo 'source <(COMPLETE=bash podatki)' >> ~/.bashrc"
