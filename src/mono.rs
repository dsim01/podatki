use std::{str::FromStr, time::Duration};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local};
use rust_decimal::Decimal;
use serde::Deserialize;

const MONO_URL: &str = "https://api.monobank.ua/bank/currency";
const UAH: u16 = 980;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MonoRow {
    currency_code_a: u16,
    currency_code_b: u16,
    date: i64,
    rate_buy: Option<serde_json::Number>,
    rate_cross: Option<serde_json::Number>,
}

#[derive(Debug, Clone)]
pub enum Kind {
    Buy,
    Cross,
}

#[derive(Debug, Clone)]
pub struct MonoRate {
    pub rate: Decimal,
    pub kind: Kind,
    pub updated: DateTime<Local>,
}

fn dec(n: &serde_json::Number) -> Result<Decimal> {
    let s = n.to_string();
    Decimal::from_str(&s)
        .or_else(|_| Decimal::from_scientific(&s))
        .context("некоректний курс моно")
}

pub fn fetch_rate(iso_code: u16) -> Result<MonoRate> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .build()
        .new_agent();
    let mut resp = match agent.get(MONO_URL).call() {
        Ok(r) => r,
        Err(ureq::Error::StatusCode(429)) => {
            bail!("monobank обмежує частоту запитів, спробуйте за хвилину")
        }
        Err(e) => return Err(e).context("запит до monobank не вдався"),
    };
    let rows: Vec<MonoRow> = resp
        .body_mut()
        .read_json()
        .context("не вдалося розібрати відповідь monobank")?;

    let Some(row) = rows
        .into_iter()
        .find(|r| r.currency_code_a == iso_code && r.currency_code_b == UAH)
    else {
        bail!("monobank не публікує курс для цієї валюти");
    };
    let (rate, kind) = match (&row.rate_buy, &row.rate_cross) {
        (Some(b), _) => (dec(b)?, Kind::Buy),
        (None, Some(c)) => (dec(c)?, Kind::Cross),
        (None, None) => bail!("monobank не повернув курс"),
    };
    let updated = DateTime::from_timestamp(row.date, 0)
        .context("некоректна дата курсу моно")?
        .with_timezone(&Local);
    Ok(MonoRate {
        rate,
        kind,
        updated,
    })
}
