#!/bin/sh
# Removes podatki. Add --purge to also delete the config:
#   curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/uninstall.sh | sh
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}"

rm -f "$BIN_DIR/podatki" "$CONFIG_DIR/fish/completions/podatki.fish"
echo "Removed: $BIN_DIR/podatki"

if [ "${1:-}" = "--purge" ]; then
    rm -rf "$CONFIG_DIR/podatki"
    echo "Removed: $CONFIG_DIR/podatki"
fi
