# podatki

A command-line tax calculator for Ukrainian sole proprietors (ФОП, 3rd group).
Given the date income was received, the currency and the amount, it fetches the
official National Bank of Ukraine (NBU) exchange rate for that date and calculates
the single tax (ЄП), military levy (ВЗ) and social contribution (ЄСВ).

It is the console counterpart of the podatki.ua web app (`../frontend`,
`../backend`) and uses the same calculation and rounding rules.

```
$ podatki 12.06.2026 USD 1000

  Розрахунок податків ФОП 3 група
  ────────────────────────────────────────────
  Дата                              12.06.2026
  Дохід                           1 000,00 USD
  Курс НБУ                     44,9256 UAH/USD
  Валюта                             Долар США
  Дохід у гривні                   44 925,60 ₴
  ────────────────────────────────────────────
  Єдиний податок (5%)               2 246,28 ₴
  Військовий збір (1%)                449,26 ₴
  ЄСВ (фіксований)                  1 902,34 ₴
  ────────────────────────────────────────────
  До сплати                         4 597,88 ₴
  Залишок після податків           40 327,72 ₴

  Монобанк (поточний курс)
  ────────────────────────────────────────────
  Курс купівлі моно              45,55 UAH/USD
  Оновлено                    08.11.2026 00:01
  Продаж валюти в моно             45 550,00 ₴
  Різниця з курсом НБУ              +624,40 ₴
  На руки після податків           40 952,12 ₴
```

The program's output is in Ukrainian; this document is in English.

## Installation

Linux (x86_64 or ARM64):

```sh
curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/install.sh | sh
```

This downloads the latest release binary to `~/.local/bin/podatki` and, if `fish`
is installed, sets up shell completion. Make sure `~/.local/bin` is on your `PATH`.
Run the same command again to update.

Windows: download `podatki-x86_64-pc-windows-msvc.zip` from the
[latest release](https://github.com/dsim01/podatki/releases/latest).

From source (requires Rust):

```sh
cargo install --path .
```

### Uninstallation

```sh
curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/uninstall.sh | sh
# also remove the config:
curl -fsSL https://raw.githubusercontent.com/dsim01/podatki/main/scripts/uninstall.sh | sh -s -- --purge
```

If you added completion lines to `~/.zshrc` or `~/.bashrc`, remove them by hand.

## Usage

```
podatki [OPTIONS] <DATE> <CURRENCY> [AMOUNT]
```

| Argument   | Description |
|------------|-------------|
| `DATE`     | Date the income was received: `DD.MM.YYYY`, `YYYY-MM-DD`, `today` (`сьогодні`) or `yesterday` (`вчора`). |
| `CURRENCY` | Currency code, case-insensitive: `USD`, `EUR`, `PLN`, … — any currency the NBU publishes. `UAH` skips conversion. |
| `AMOUNT`   | Income in that currency. Optional: if omitted, you are prompted for it. Accepts `1000`, `2500.50`, `2500,50`, `2 500,50`. At most 2 decimal places. |

| Option              | Description |
|---------------------|-------------|
| `--no-esv`          | Exclude ЄСВ from the total. |
| `--no-mil`          | Exclude the military levy from the total. |
| `--no-mono`         | Hide the monobank section. |
| `-c, --config PATH` | Use a different config file (also `PODATKI_CONFIG` env var). |
| `-h, --help`        | Show help. |
| `-V, --version`     | Show version. |

Examples:

```sh
podatki 12.06.2026 USD 1000
podatki yesterday eur 2500,50 --no-esv
podatki today USD            # asks for the amount
podatki 01.03.2026 UAH 50000 # income already in hryvnias
```

## Configuration

On first run, a default config is created at `~/.config/podatki/config.toml`
(or `$XDG_CONFIG_HOME/podatki/config.toml`):

```toml
# ЄСВ — fixed monthly amount in UAH
esv = 1902.34

# Single tax — fraction of income (0.05 = 5%)
ep = 0.05

# Military levy — fraction of income (0.01 = 1%)
mil = 0.01

# Decimal places for rounding
decimal_places = 2
```

| Key              | Meaning | Default |
|------------------|---------|---------|
| `esv`            | ЄСВ, a fixed amount in UAH (22% of the minimum wage). | `1902.34` |
| `ep`             | Single tax rate as a fraction of income. | `0.05` |
| `mil`            | Military levy rate as a fraction of income. | `0.01` |
| `decimal_places` | Rounding precision. | `2` |

Values may be written as numbers (`1902.34`) or strings (`"1902,34"`). Negative
values are rejected.

The config holds a single set of values for all dates. When the minimum wage or
tax rates change, update the file. Calculations for dates before that change will
then use the new values, so they will be wrong for those older dates.

Config path resolution order: `--config`, then `PODATKI_CONFIG`, then the default
path.

## How the calculation works

```
income_uah = amount × NBU rate on DATE
ЄП         = round(income_uah × ep)
ВЗ         = round(income_uah × mil)        (0 with --no-mil)
ЄСВ        = round(esv)                     (0 with --no-esv)
total      = ЄП + ВЗ + ЄСВ
```

- Each tax is calculated from the **unrounded** UAH income and rounded
  separately; the total is the sum of the rounded taxes.
- Rounding is half away from zero (0.005 → 0.01), the same as `Big.roundHalfUp` in
  the frontend.
- All arithmetic uses exact decimals (`rust_decimal`), not floating point. The
  NBU rate is parsed from the JSON text as-is.

This mirrors `frontend/src/utils/tax-calculations.ts`. Results were compared with
the frontend's `big.js` logic on 20,012 cases (random plus half-kopeck edge cases)
with no differences.

The CLI also enforces the frontend's input rule: the amount must be positive and
have at most 2 decimal places.

## Data sources

### NBU (tax base)

```
GET https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange?valcode=USD&date=20260612&json
```

Queried on every run; nothing is cached. The rate is per 1 unit of currency. The
NBU also sets rates for weekends and holidays. If no rate exists for the date (for
example, a future date), the CLI exits with an error.

The web backend uses a different NBU endpoint
(`https://bank.gov.ua/NBU_Exchange/exchange_site`) and caches it. For USD and EUR
both endpoints return identical rates.

### monobank (informational)

```
GET https://api.monobank.ua/bank/currency
```

Shows what you would actually receive if you sold the currency to monobank
**now**:

- **Курс купівлі моно**: monobank's buy rate (published for USD and EUR).
- **Крос-курс моно**: the cross rate, used for currencies with no buy rate (PLN,
  GBP, CHF, …).
- **Різниця з курсом НБУ**: sale proceeds minus income at the NBU rate.
- **На руки після податків**: sale proceeds minus total tax.

Notes:

- monobank only publishes current rates, so this section always uses today's rate,
  even for past dates.
- It has no effect on the tax calculation: taxable income is always valued at the
  NBU rate on the date of receipt.
- monobank rate-limits this endpoint heavily. Running the CLI several times in a
  row often hits a 429 error. The section then shows a warning and the rest of the
  output is unaffected. Use `--no-mono` to skip the request.

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
| `build`   | after `test` passes | Release build for each target, packaged with `README.md` and uploaded as a workflow artifact. |
| `release` | tags `v*` only | Checks that the tag matches the `Cargo.toml` version, then creates a GitHub Release with all archives. |

| Target | Runner | Archive |
|--------|--------|---------|
| `x86_64-unknown-linux-gnu`  | `ubuntu-latest`    | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | `.tar.gz` |
| `x86_64-pc-windows-msvc`    | `windows-latest`   | `.zip` |

All targets build natively on their own runner, with no cross-compilation.

To publish a release:

```sh
# 1. bump `version` in Cargo.toml, then:
cargo build            # updates Cargo.lock
git commit -am "Release v0.2.0"
git tag v0.2.0
git push origin main v0.2.0
```

Archives are named `podatki-<target>` without a version, so
`releases/latest/download/podatki-<target>.tar.gz` always points to the latest
release. The install script relies on this.

## Known limitations

- A single config for all dates: past periods with different ЄСВ or rates are
  calculated with the current values.
- No caching or calculation history; every run makes network requests.
- Piping output into a command that closes early (e.g. `podatki … | head`) panics
  with a broken pipe error.
