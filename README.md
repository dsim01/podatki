# podatki

Tax calculator for Ukrainian sole proprietors (FOP, 3rd group): single tax,
military levy and social contribution (ESV). Uses the official NBU exchange rate
for the date the income was received.

## Install

Linux:

```sh
curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/install.sh | sh
```

Windows: download `podatki-windows-x86_64.exe` from the
[latest release](https://github.com/dsim01/podatki/releases/latest).

Uninstall:

```sh
curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/uninstall.sh | sh
```

## Usage

```sh
podatki 12.06.2026 USD 1000
podatki yesterday EUR 2500,50
podatki today USD              # prompts for the amount
podatki 01.03.2026 UAH 50000
```

| Option      | Description |
|-------------|-------------|
| `--no-esv`  | Exclude ESV. |
| `--no-mil`  | Exclude military levy. |
| `--mono`    | Compare with the current monobank rate. |
| `--help`    | Show help. |

## Config

`~/.config/podatki/config.toml` (Windows: `%APPDATA%\podatki\config.toml`),
created on first run:

```toml
esv = 1902.34          # ESV, UAH per month
ep = 0.05              # single tax, 5%
mil = 0.01             # military levy, 1%
decimal_places = 2
```

Update it when the minimum wage or tax rates change.

## Shell completion

```sh
echo 'source <(COMPLETE=zsh podatki)' >> ~/.zshrc     # zsh
echo 'source <(COMPLETE=bash podatki)' >> ~/.bashrc   # bash
```

fish is set up automatically by the installer.

---

Development notes: [DEVELOPMENT.md](DEVELOPMENT.md).
