use std::str::FromStr;

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

const NBU_URL: &str = "https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange";

#[derive(Debug, Deserialize)]
struct NbuRow {
    r030: u16,
    rate: serde_json::Number,
    cc: String,
    txt: String,
    exchangedate: String,
}

#[derive(Debug, Clone)]
pub struct Rate {
    pub rate: Decimal,
    pub iso_code: u16,
    pub code: String,
    pub name: String,
    pub date: String,
}

pub fn fetch_rate(currency: &str, date: NaiveDate) -> Result<Rate> {
    let rows: Vec<NbuRow> = ureq::get(NBU_URL)
        .query("valcode", currency)
        .query("date", date.format("%Y%m%d").to_string())
        .query("json", "")
        .call()
        .context("не вдалося виконати запит до НБУ")?
        .body_mut()
        .read_json()
        .context("не вдалося розібрати відповідь НБУ")?;

    let Some(row) = rows.into_iter().next() else {
        bail!(
            "НБУ не повернув курс {currency} на {} (невідома валюта або курс ще не встановлено)",
            date.format("%d.%m.%Y")
        );
    };

    let rate = Decimal::from_str(&row.rate.to_string())
        .or_else(|_| Decimal::from_scientific(&row.rate.to_string()))
        .context("некоректний курс у відповіді НБУ")?;

    Ok(Rate {
        rate,
        iso_code: row.r030,
        code: row.cc,
        name: row.txt,
        date: row.exchangedate,
    })
}
