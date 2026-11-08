use owo_colors::{OwoColorize, Stream::Stdout};
use rust_decimal::Decimal;

pub fn money(v: Decimal, dp: u32) -> String {
    let r = v
        .abs()
        .round_dp_with_strategy(dp, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
    let s = format!("{:.*}", dp as usize, r);
    let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
    let mut grouped = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(' ');
        }
        grouped.push(c);
    }
    let sign = if v.is_sign_negative() && !v.is_zero() {
        "-"
    } else {
        ""
    };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped},{frac}")
    }
}

pub fn percent(v: Decimal) -> String {
    format!("{}%", (v * Decimal::ONE_HUNDRED).normalize()).replace('.', ",")
}

pub struct Row {
    pub label: String,
    pub value: String,
    pub highlight: bool,
}

pub fn rule(width: usize) -> String {
    "─".repeat(width)
}

pub fn print_rows(rows: &[Row], width: usize) {
    for r in rows {
        let pad = width.saturating_sub(r.label.chars().count() + r.value.chars().count());
        if r.highlight {
            println!(
                "  {}{}{}",
                r.label.if_supports_color(Stdout, |t| t.bold()),
                " ".repeat(pad),
                r.value
                    .if_supports_color(Stdout, |t| t.bright_green().bold().to_string()),
            );
        } else {
            println!(
                "  {}{}{}",
                r.label.if_supports_color(Stdout, |t| t.dimmed()),
                " ".repeat(pad),
                r.value,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn formats_money() {
        assert_eq!(money(dec!(1234567.8), 2), "1 234 567,80");
        assert_eq!(money(dec!(12.345), 2), "12,35");
        assert_eq!(money(dec!(100), 2), "100,00");
    }

    #[test]
    fn formats_percent() {
        assert_eq!(percent(dec!(0.05)), "5%");
        assert_eq!(percent(dec!(0.015)), "1,5%");
    }
}
