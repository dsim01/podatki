#!/bin/sh
# Installs the latest podatki release to ~/.local/bin:
#   curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/install.sh | sh
set -eu

BIN_DIR="${BIN_DIR:-$HOME/.local/bin}"
FISH_COMPLETIONS="${XDG_CONFIG_HOME:-$HOME/.config}/fish/completions/podatki.fish"

case "$(uname -m)" in
    x86_64 | amd64) asset=podatki-linux-x86_64 ;;
    aarch64 | arm64) asset=podatki-linux-arm64 ;;
    *) echo "Unsupported architecture: $(uname -m)" >&2; exit 1 ;;
esac

mkdir -p "$BIN_DIR"
tmp="$BIN_DIR/.podatki.download"
trap 'rm -f "$tmp"' EXIT
curl -fsSL "https://github.com/dsim01/podatki/releases/latest/download/$asset" -o "$tmp"
chmod 755 "$tmp"
mv -f "$tmp" "$BIN_DIR/podatki"
echo "Installed: $BIN_DIR/podatki"

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *)
        echo
        echo "Warning: $BIN_DIR is not in your PATH, so the podatki command will not be found."
        echo "Add this line to ~/.bashrc or ~/.zshrc and open a new terminal:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo "For fish: fish_add_path ~/.local/bin"
        ;;
esac

if command -v fish >/dev/null 2>&1; then
    mkdir -p "$(dirname "$FISH_COMPLETIONS")"
    echo 'COMPLETE=fish podatki | source' >"$FISH_COMPLETIONS"
    echo "fish completion: $FISH_COMPLETIONS"
fi

echo
echo "Completion for zsh/bash (optional, run once):"
echo "  zsh:  echo 'source <(COMPLETE=zsh podatki)' >> ~/.zshrc"
echo "  bash: echo 'source <(COMPLETE=bash podatki)' >> ~/.bashrc"
