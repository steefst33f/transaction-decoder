use std::fmt::Debug;
use serde::{Serialize, Serializer};

use crate::amount::{Amount, BitcoinValue};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: String,
    pub output_index: u32,
    pub script: String,
    pub sequence: u32,
}
#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub output_script: String,
}

fn as_btc<T: BitcoinValue, S: Serializer>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}

#[derive(Debug, Serialize)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
}
