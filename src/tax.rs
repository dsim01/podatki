use rust_decimal::{Decimal, RoundingStrategy};

use crate::config::Config;

#[derive(Debug, Clone, PartialEq)]
pub struct Taxes {
    pub income_uah: Decimal,
    pub ep: Decimal,
    pub mil: Decimal,
    pub esv: Decimal,
    pub total: Decimal,
}

pub struct Options {
    pub esv: bool,
    pub mil: bool,
}

fn round(v: Decimal, dp: u32) -> Decimal {
    v.round_dp_with_strategy(dp, RoundingStrategy::MidpointAwayFromZero)
}

pub fn calculate(amount: Decimal, rate: Decimal, cfg: &Config, opts: &Options) -> Taxes {
    let dp = cfg.decimal_places;
    let income = amount * rate;
    let ep = round(income * cfg.ep, dp);
    let mil = if opts.mil {
        round(income * cfg.mil, dp)
    } else {
        Decimal::ZERO
    };
    let esv = if opts.esv {
        round(cfg.esv, dp)
    } else {
        Decimal::ZERO
    };
    Taxes {
        income_uah: round(income, dp),
        ep,
        mil,
        esv,
        total: round(ep + mil + esv, dp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn cfg() -> Config {
        Config {
            esv: dec!(1902.34),
            ep: dec!(0.05),
            mil: dec!(0.01),
            decimal_places: 2,
        }
    }

    #[test]
    fn full() {
        let t = calculate(
            dec!(1000),
            dec!(44.9256),
            &cfg(),
            &Options {
                esv: true,
                mil: true,
            },
        );
        assert_eq!(t.income_uah, dec!(44925.60));
        assert_eq!(t.ep, dec!(2246.28));
        assert_eq!(t.mil, dec!(449.26));
        assert_eq!(t.esv, dec!(1902.34));
        assert_eq!(t.total, dec!(4597.88));
    }

    #[test]
    fn without_esv_and_mil() {
        let t = calculate(
            dec!(1000),
            dec!(44.9256),
            &cfg(),
            &Options {
                esv: false,
                mil: false,
            },
        );
        assert_eq!(t.total, dec!(2246.28));
    }
}
