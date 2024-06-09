use std::fmt::Debug;
use serde::Serialize;

#[derive(Debug, Serialize)]

pub struct Amount(u64);
pub trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl Amount {
    pub fn from_sat(satoshi: u64) -> Amount {
        Amount(satoshi)
    }
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}