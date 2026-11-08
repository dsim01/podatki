# podatki: developer notes

User documentation is in [README.md](README.md). This file covers
how the CLI works, how to build it and how releases are made.

## Command-line reference

```
podatki [OPTIONS] <DATE> <CURRENCY> [AMOUNT]
```

| Argument   | Description |
|------------|-------------|
| `DATE`     | `DD.MM.YYYY`, `YYYY-MM-DD`, `today` or `yesterday`. |
| `CURRENCY` | Currency code, case-insensitive; any currency the NBU publishes. `UAH` skips conversion. |
| `AMOUNT`   | Optional (prompted if omitted). Accepts `1000`, `2500.50`, `2500,50`, `2 500,50`. At most 2 decimal places, must be positive. |

| Option              | Description |
|---------------------|-------------|
| `--no-esv`          | Exclude ESV from the total. |
| `--no-mil`          | Exclude the military levy from the total. |
| `--mono`            | Show the monobank section (compare with the current monobank rate). |
| `-c, --config PATH` | Use a different config file (also `PODATKI_CONFIG` env var). |
| `-h, --help`        | Show help. |
| `-V, --version`     | Show version. |

Config path resolution order: `--config`, then `PODATKI_CONFIG`, then
`dirs::config_dir()/podatki/config.toml`. Config values may be numbers
(`1902.34`) or strings (`"1902,34"`); negative values are rejected.

## How the calculation works

```
income_uah = amount × NBU rate on DATE
single_tax = round(income_uah × ep)
military   = round(income_uah × mil)        (0 with --no-mil)
esv        = round(esv)                     (0 with --no-esv)
total      = single_tax + military + esv
```

- Each tax is calculated from the **unrounded** UAH income and rounded
  separately; the total is the sum of the rounded taxes.
- Rounding is half away from zero (0.005 → 0.01).
- All arithmetic uses exact decimals (`rust_decimal`), not floating point. The
  NBU rate is parsed from the JSON text as-is.

The amount must be positive and have at most 2 decimal places.

## Data sources

### NBU (tax base)

```
GET https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange?valcode=USD&date=20260612&json
```

Queried on every run; nothing is cached. The rate is per 1 unit of currency. The
NBU also sets rates for weekends and holidays. If no rate exists for the date (for
example, a future date), the CLI exits with an error.

### monobank (informational)

```
GET https://api.monobank.ua/bank/currency
```

Off by default; enabled with `--mono`. Shows what you would actually receive if
you sold the currency to monobank **now**:

- **monobank buy rate**: monobank's buy rate (published for USD and EUR).
- **monobank cross rate**: the cross rate, used for currencies with no buy rate (PLN,
  GBP, CHF, …).
- **Difference vs NBU**: sale proceeds minus income at the NBU rate.
- **Net after taxes**: sale proceeds minus total tax.

Notes:

- monobank only publishes current rates, so this section always uses today's rate,
  even for past dates.
- It has no effect on the tax calculation: taxable income is always valued at the
  NBU rate on the date of receipt.
- monobank rate-limits this endpoint heavily. Running the CLI several times in a
  row often hits a 429 error. The section then shows a warning and the rest of the
  output is unaffected.

## Shell completion

Completion is dynamic: it knows which argument the cursor is on. It suggests
`today`, `yesterday` and the last 7 dates for `DATE`, currency codes with names
for `CURRENCY`, file paths for `--config`, and all flags.

```sh
# fish (installed automatically by install.sh)
echo 'COMPLETE=fish podatki | source' > ~/.config/fish/completions/podatki.fish

# zsh
echo 'source <(COMPLETE=zsh podatki)' >> ~/.zshrc

# bash
echo 'source <(COMPLETE=bash podatki)' >> ~/.bashrc
```

Completion uses `clap_complete`'s `unstable-dynamic` feature, so its API may change
when the dependency is upgraded.

## Development

```sh
cargo build
cargo test
cargo clippy --all-targets
cargo run -- 12.06.2026 USD 1000
```

| File            | Purpose |
|-----------------|---------|
| `src/main.rs`   | Argument parsing, date and amount parsing, completion, output layout. |
| `src/config.rs` | Config loading, default creation, decimal parsing. |
| `src/nbu.rs`    | NBU exchange rate client. |
| `src/mono.rs`   | monobank exchange rate client. |
| `src/tax.rs`    | Tax calculation and rounding. |
| `src/output.rs` | Number formatting (`1 234,56`) and row rendering. |
| `scripts/install.sh`, `scripts/uninstall.sh` | Install the latest release to, or remove it from, `~/.local/bin` (used via `curl \| sh`). |
| `.github/workflows/ci.yml` | CI: tests, release builds, GitHub Releases. |

## CI and releases

`.github/workflows/ci.yml` runs on GitHub Actions:

| Job       | When | What |
|-----------|------|------|
| `test`    | every push to `main`, pull requests, tags | `cargo fmt --check` (Linux only), `cargo clippy -D warnings` and `cargo test` on Linux x86_64, Linux ARM64 and Windows. |
| `build`   | after `test` passes | Release build for each target, uploaded as a workflow artifact (a plain executable, no archive). |
| `release` | tags `v*` only | Checks that the tag matches the `Cargo.toml` version, then creates a GitHub Release with all binaries. Release notes list the commits since the previous tag (up to 10, plus a compare link), generated by `scripts/release-notes.sh`. |

| Target | Runner | Release asset |
|--------|--------|---------------|
| `x86_64-unknown-linux-musl`  | `ubuntu-latest`    | `podatki-linux-x86_64` |
| `aarch64-unknown-linux-musl` | `ubuntu-24.04-arm` | `podatki-linux-arm64` |
| `x86_64-pc-windows-msvc`     | `windows-latest`   | `podatki-windows-x86_64.exe` |

All targets build natively on their own runner, with no cross-compilation.
Linux builds use musl and are fully static, so they run on any distribution
regardless of its glibc version. `ring` (pulled in by `rustls`) compiles C code,
so CI installs `musl-tools` and sets `CC_<target>=musl-gcc`. To build locally:

```sh
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
CC_x86_64_unknown_linux_musl=musl-gcc cargo build --release --target x86_64-unknown-linux-musl
```

Release binaries are stripped (`[profile.release] strip = true`).

To publish a release:

```sh
# 1. bump `version` in Cargo.toml, then:
cargo build            # updates Cargo.lock
git commit -am "Release v0.2.0"
git tag v0.2.0
git push origin main v0.2.0
```

Asset names contain no version, so
`releases/latest/download/<asset>` always points to the latest release. The
install script relies on this.

## Known limitations

- A single config for all dates: past periods with different ESV or rates are
  calculated with the current values.
- No caching or calculation history; every run makes network requests.
- Piping output into a command that closes early (e.g. `podatki … | head`) panics
  with a broken pipe error.
