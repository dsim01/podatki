mod config;
mod mono;
mod nbu;
mod output;
mod tax;

use std::{io::Write, path::PathBuf, str::FromStr};

use anyhow::{Context, Result, bail};
use chrono::{Days, Local, NaiveDate};
use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::{
    CompleteEnv,
    engine::{ArgValueCandidates, CompletionCandidate},
};
use owo_colors::{OwoColorize, Stream::Stdout};
use rust_decimal::Decimal;

use output::{Row, money, percent, print_rows, rule};

fn date_candidates() -> Vec<CompletionCandidate> {
    let today = Local::now().date_naive();
    let mut out = vec![
        CompletionCandidate::new("today").help(Some("сьогодні".into())),
        CompletionCandidate::new("yesterday").help(Some("вчора".into())),
    ];
    for i in 0..7 {
        let d = today - Days::new(i);
        out.push(CompletionCandidate::new(d.format("%d.%m.%Y").to_string()));
    }
    out
}

fn currency_candidates() -> Vec<CompletionCandidate> {
    CURRENCY_SUGGESTIONS
        .iter()
        .map(|(code, name)| CompletionCandidate::new(*code).help(Some((*name).into())))
        .collect()
}

const CURRENCY_SUGGESTIONS: &[(&str, &str)] = &[
    ("USD", "Долар США"),
    ("EUR", "Євро"),
    ("GBP", "Фунт стерлінгів"),
    ("UAH", "Гривня (без конвертації)"),
    ("PLN", "Злотий"),
    ("CHF", "Швейцарський франк"),
    ("CZK", "Чеська крона"),
    ("CAD", "Канадський долар"),
    ("JPY", "Єна"),
];

#[derive(Parser, Debug)]
#[command(
    name = "podatki",
    version,
    about,
    after_help = "Приклади:\n  podatki 12.06.2026 USD 1000\n  podatki yesterday EUR 2500,50 --no-esv\n\nАвтодоповнення:\n  fish:  echo 'COMPLETE=fish podatki | source' > ~/.config/fish/completions/podatki.fish\n  zsh:   echo 'source <(COMPLETE=zsh podatki)' >> ~/.zshrc\n  bash:  echo 'source <(COMPLETE=bash podatki)' >> ~/.bashrc"
)]
struct Cli {
    #[arg(add = ArgValueCandidates::new(date_candidates))]
    date: String,
    #[arg(add = ArgValueCandidates::new(currency_candidates))]
    currency: String,
    amount: Option<String>,
    #[arg(long)]
    no_esv: bool,
    #[arg(long)]
    no_mil: bool,
    #[arg(long)]
    no_mono: bool,
    #[arg(short, long, env = "PODATKI_CONFIG", value_hint = ValueHint::FilePath)]
    config: Option<PathBuf>,
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    let today = Local::now().date_naive();
    match s.to_lowercase().as_str() {
        "today" => return Ok(today),
        "yesterday" => return Ok(today - Days::new(1)),
        _ => {}
    }
    NaiveDate::parse_from_str(s, "%d.%m.%Y")
        .or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .map_err(|_| anyhow::anyhow!("некоректна дата '{s}', очікується ДД.ММ.РРРР"))
}

fn parse_amount(s: &str) -> Result<Decimal> {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '_')
        .collect();
    let v = Decimal::from_str(&cleaned.replace(',', "."))
        .with_context(|| format!("некоректна сума '{s}'"))?;
    if v <= Decimal::ZERO {
        bail!("сума має бути більшою за 0");
    }
    if v.normalize().scale() > 2 {
        bail!("сума може мати не більше 2 знаків після коми");
    }
    Ok(v)
}

fn prompt_amount(currency: &str) -> Result<Decimal> {
    print!("Сума доходу в {currency}: ");
    std::io::stdout().flush()?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    parse_amount(line.trim())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let date = parse_date(&cli.date)?;
    let currency = cli.currency.to_uppercase();

    let cfg_path = match cli.config {
        Some(p) => p,
        None => config::default_path()?,
    };
    let (cfg, created) = config::load_or_init(&cfg_path)?;
    if created {
        eprintln!(
            "Створено конфігурацію за замовчуванням: {}",
            cfg_path.display()
        );
    }

    let amount = match cli.amount.as_deref() {
        Some(a) => parse_amount(a)?,
        None => prompt_amount(&currency)?,
    };

    let (rate, rate_info) = if currency == "UAH" {
        (Decimal::ONE, None)
    } else {
        let r = nbu::fetch_rate(&currency, date)?;
        (r.rate, Some(r))
    };

    let opts = tax::Options {
        esv: !cli.no_esv,
        mil: !cli.no_mil,
    };
    let t = tax::calculate(amount, rate, &cfg, &opts);
    let dp = cfg.decimal_places;
    let width = 44;

    println!();
    println!(
        "  {}",
        "Розрахунок податків ФОП 3 група"
            .if_supports_color(Stdout, |s| s.bold().cyan().to_string())
    );
    println!("  {}", rule(width));

    let mut info = vec![
        Row {
            label: "Дата".into(),
            value: date.format("%d.%m.%Y").to_string(),
            highlight: false,
        },
        Row {
            label: "Дохід".into(),
            value: format!("{} {currency}", money(amount, 2)),
            highlight: false,
        },
    ];
    if let Some(r) = &rate_info {
        info.push(Row {
            label: "Курс НБУ".into(),
            value: format!(
                "{} UAH/{}",
                r.rate.normalize().to_string().replace('.', ","),
                r.code
            ),
            highlight: false,
        });
        info.push(Row {
            label: "Валюта".into(),
            value: r.name.clone(),
            highlight: false,
        });
        if r.date != date.format("%d.%m.%Y").to_string() {
            info.push(Row {
                label: "Курс встановлено на".into(),
                value: r.date.clone(),
                highlight: false,
            });
        }
    }
    info.push(Row {
        label: "Дохід у гривні".into(),
        value: format!("{} ₴", money(t.income_uah, dp)),
        highlight: false,
    });
    print_rows(&info, width);

    println!("  {}", rule(width));
    let fmt_tax = |on: bool, v: Decimal| {
        if on {
            format!("{} ₴", money(v, dp))
        } else {
            "—".into()
        }
    };
    let taxes = vec![
        Row {
            label: format!("Єдиний податок ({})", percent(cfg.ep)),
            value: fmt_tax(true, t.ep),
            highlight: false,
        },
        Row {
            label: format!("Військовий збір ({})", percent(cfg.mil)),
            value: fmt_tax(opts.mil, t.mil),
            highlight: false,
        },
        Row {
            label: "ЄСВ (фіксований)".into(),
            value: fmt_tax(opts.esv, t.esv),
            highlight: false,
        },
    ];
    print_rows(&taxes, width);
    println!("  {}", rule(width));
    print_rows(
        &[Row {
            label: "До сплати".into(),
            value: format!("{} ₴", money(t.total, dp)),
            highlight: true,
        }],
        width,
    );

    let net = t.income_uah - t.total;
    print_rows(
        &[Row {
            label: "Залишок після податків".into(),
            value: format!("{} ₴", money(net, dp)),
            highlight: false,
        }],
        width,
    );

    if let (Some(r), false) = (&rate_info, cli.no_mono) {
        print_mono(r, amount, t.total, dp, width);
    }
    println!();
    println!(
        "  {}",
        format!("Конфіг: {}", cfg_path.display())
            .if_supports_color(Stdout, |s| s.dimmed().to_string())
    );
    println!();
    Ok(())
}

fn print_mono(nbu: &nbu::Rate, amount: Decimal, total_tax: Decimal, dp: u32, width: usize) {
    println!();
    println!(
        "  {}",
        "Монобанк (поточний курс)".if_supports_color(Stdout, |s| s.bold().cyan().to_string())
    );
    println!("  {}", rule(width));
    let m = match mono::fetch_rate(nbu.iso_code) {
        Ok(m) => m,
        Err(e) => {
            println!(
                "  {}",
                format!("Курс недоступний: {e:#}")
                    .if_supports_color(Stdout, |s| s.yellow().to_string())
            );
            return;
        }
    };
    let label = match m.kind {
        mono::Kind::Buy => "Курс купівлі моно",
        mono::Kind::Cross => "Крос-курс моно",
    };
    let sale = amount * m.rate;
    let diff = sale - amount * nbu.rate;
    let sign = if diff.is_sign_negative() { "−" } else { "+" };
    print_rows(
        &[
            Row {
                label: label.into(),
                value: format!(
                    "{} UAH/{}",
                    m.rate.normalize().to_string().replace('.', ","),
                    nbu.code
                ),
                highlight: false,
            },
            Row {
                label: "Оновлено".into(),
                value: m.updated.format("%d.%m.%Y %H:%M").to_string(),
                highlight: false,
            },
            Row {
                label: "Продаж валюти в моно".into(),
                value: format!("{} ₴", money(sale, dp)),
                highlight: false,
            },
            Row {
                label: "Різниця з курсом НБУ".into(),
                value: format!("{sign}{} ₴", money(diff.abs(), dp)),
                highlight: false,
            },
            Row {
                label: "На руки після податків".into(),
                value: format!("{} ₴", money(sale - total_tax, dp)),
                highlight: true,
            },
        ],
        width,
    );
}

fn main() {
    CompleteEnv::with_factory(Cli::command)
        .bin("podatki")
        .completer("podatki")
        .complete();

    if let Err(e) = run() {
        eprintln!(
            "{} {e:#}",
            "Помилка:"
                .if_supports_color(owo_colors::Stream::Stderr, |s| s.red().bold().to_string())
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn parses_amounts() {
        assert_eq!(parse_amount("1000").unwrap(), dec!(1000));
        assert_eq!(parse_amount("2 500,50").unwrap(), dec!(2500.50));
        assert!(parse_amount("-1").is_err());
        assert!(parse_amount("abc").is_err());
        assert!(parse_amount("1000.555").is_err());
        assert_eq!(parse_amount("1000.500").unwrap(), dec!(1000.5));
    }

    #[test]
    fn parses_dates() {
        assert_eq!(
            parse_date("12.06.2026").unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 12).unwrap()
        );
        assert!(parse_date("2026/06/12").is_err());
        let today = Local::now().date_naive();
        assert_eq!(parse_date("today").unwrap(), today);
        assert_eq!(parse_date("Yesterday").unwrap(), today - Days::new(1));
    }

    #[test]
    fn cli_definition_is_valid() {
        Cli::command().debug_assert();
    }
}
