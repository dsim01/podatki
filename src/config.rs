use std::{fs, path::PathBuf, str::FromStr};

use anyhow::{Context, Result, bail};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, de::Error as _};

pub const DEFAULT_CONFIG: &str = r#"# Конфігурація podatki (ФОП 3 група)

# ЄСВ — фіксована сума в гривнях за місяць
esv = 1902.34

# Єдиний податок — частка від доходу (0.05 = 5%)
ep = 0.05

# Військовий збір — частка від доходу (0.01 = 1%)
mil = 0.01

# Кількість знаків після коми при округленні
decimal_places = 2
"#;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "de_decimal")]
    pub esv: Decimal,
    #[serde(deserialize_with = "de_decimal")]
    pub ep: Decimal,
    #[serde(deserialize_with = "de_decimal")]
    pub mil: Decimal,
    #[serde(default = "default_decimal_places")]
    pub decimal_places: u32,
}

fn de_decimal<'de, D: Deserializer<'de>>(d: D) -> Result<Decimal, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Str(String),
        Int(i64),
        Float(f64),
    }
    let s = match Raw::deserialize(d)? {
        Raw::Str(s) => s.trim().replace(',', "."),
        Raw::Int(i) => i.to_string(),
        Raw::Float(f) => f.to_string(),
    };
    Decimal::from_str(&s).map_err(|e| D::Error::custom(format!("некоректне число '{s}': {e}")))
}

fn default_decimal_places() -> u32 {
    2
}

pub fn default_path() -> Result<PathBuf> {
    if let Ok(p) = std::env::var("PODATKI_CONFIG") {
        return Ok(PathBuf::from(p));
    }
    let dir = dirs::config_dir().context("не вдалося визначити каталог конфігурації")?;
    Ok(dir.join("podatki").join("config.toml"))
}

pub fn load_or_init(path: &PathBuf) -> Result<(Config, bool)> {
    let mut created = false;
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("не вдалося створити {}", parent.display()))?;
        }
        fs::write(path, DEFAULT_CONFIG)
            .with_context(|| format!("не вдалося записати {}", path.display()))?;
        created = true;
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("не вдалося прочитати {}", path.display()))?;
    let cfg: Config = toml::from_str(&raw)
        .with_context(|| format!("помилка в конфігурації {}", path.display()))?;
    if cfg.esv.is_sign_negative() || cfg.ep.is_sign_negative() || cfg.mil.is_sign_negative() {
        bail!("значення esv/ep/mil не можуть бути від'ємними");
    }
    Ok((cfg, created))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn default_config_parses() {
        let cfg: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(cfg.esv, dec!(1902.34));
        assert_eq!(cfg.ep, dec!(0.05));
        assert_eq!(cfg.mil, dec!(0.01));
        assert_eq!(cfg.decimal_places, 2);
    }

    #[test]
    fn accepts_string_values() {
        let cfg: Config = toml::from_str("esv = \"1902,34\"\nep = \"0.05\"\nmil = 0").unwrap();
        assert_eq!(cfg.esv, dec!(1902.34));
        assert_eq!(cfg.mil, dec!(0));
    }
}
