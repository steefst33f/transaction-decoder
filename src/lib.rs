use std::io::Read;
use std::error::Error;
use std::result::Result;
use clap::{arg, value_parser, Command};

mod transaction;
use self::transaction::{Decodable, Transaction,};

mod amount;

pub fn get_arg() -> String {
    let matches = Command::new("Bitcoin Transaction Decoder")
        .version("1.0")
        .about("Decodes a raw transaction")
        .arg(
            arg!([RAW_TRANSACTION])
                .value_parser(value_parser!(String))
                .required(true)
        )
        .get_matches();

    matches
        .get_one::<String>("RAW_TRANSACTION")
        .cloned()
        .expect("raw transaction is required")
}

pub fn decode(raw_transaction_hex: String) -> Result<Transaction, Box<dyn Error>> {
    let transaction_bytes = hex::decode(raw_transaction_hex).map_err(|e| format!("Hex decodiong error: {}", e))?;
    let mut bytes_slice = transaction_bytes.as_slice();
    Ok(Transaction::consensus_decode(&mut bytes_slice)?)
}

pub fn run(raw_transaction_hex: String) -> Result<String, Box<dyn Error>> {
    let transaction = decode(raw_transaction_hex)?;
    Ok(serde_json::to_string_pretty(&transaction)?)
}

pub fn read_compact_size_integer(bytes_slice: &mut &[u8]) -> std::io::Result<u64> {
    let mut compact_size = [0; 1];
    bytes_slice.read(&mut compact_size)?;
    
    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            bytes_slice.read(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        },
        254 => {
            let mut buffer = [0; 4];
            bytes_slice.read(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        },
        255 => {
            let mut buffer = [0; 8];
            bytes_slice.read(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer) as u64)
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::read_compact_size_integer;

    #[test]
    fn test_read_compact_size_integer_one_byte() {
        let mut bytes = [1_u8].as_slice();
        let length = read_compact_size_integer(&mut bytes).unwrap();
        assert_eq!(length, 1_u64);
    }

    #[test]
    fn test_read_compact_size_integer_three_bytes() {
        let mut bytes = [253_u8, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes).unwrap();
        assert_eq!(length, 256_u64);
    }

    #[test]
    fn test_read_compact_size_integer_five_bytes() {
        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes).unwrap();
        assert_eq!(length, 256_u64.pow(3));
    }

    #[test]
    fn test_read_compact_size_integer_nine_bytes() {
        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes).unwrap();
        assert_eq!(length, 256_u64.pow(7));
    }

    #[test]
    fn test_read_compact_size_integer_real_example() {
        // https://mempool.space/tx/52539a56b1eb890504b775171923430f0355eb836a57134ba598170a2f8980c1
        // fd is 253
        // transaction has 20,000 empty inputs
        let hex = "fd204e";
        let decoded = hex::decode(hex).unwrap();
        let mut bytes = decoded.as_slice();
        let length = read_compact_size_integer(&mut bytes).unwrap();
        let expected_length = 20_000_u64;
        assert_eq!(length, expected_length);
    }
}