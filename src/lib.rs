use std::io::Read;
use std::error::Error;
use std::result::Result;
use sha2::{Digest, Sha256};

mod transaction;
use transaction::{Input, Output, Transaction, Txid};

mod amount;
use amount::Amount;

fn read_u32(bytes_slice: &mut &[u8]) -> std::io::Result<u32> {
    // Read slice into a buffer
    let mut buffer = [0; 4];
    bytes_slice.read(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u64(bytes_slice: &mut &[u8]) -> std::io::Result<u64> {
    let mut buffer = [0; 8];
    bytes_slice.read(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_amount(bytes_slice: &mut &[u8]) -> std::io::Result<Amount> {
    let value = read_u64(bytes_slice)?;
    Ok(Amount::from_sat(value))
}

fn read_script(bytes_slice: &mut &[u8]) -> std::io::Result<String> {
    let script_size = read_compact_size_integer(bytes_slice)? as usize;
    println!("script_size: {}", script_size);
    let mut buffer = vec![0_u8; script_size];

    bytes_slice.read(&mut buffer)?;
    Ok(hex::encode(buffer))
}

fn read_txid(bytes_slice:&mut &[u8]) -> std::io::Result<Txid> {
    let mut buffer = [0; 32];
    _ = bytes_slice.read(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
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

fn hash_transaction(raw_transaction: &[u8]) -> Txid {
    let mut hasher = Sha256::new();
    hasher.update(&raw_transaction);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&hash1);
    let hash2 = hasher.finalize();
    
    Txid::from_bytes(hash2.into())
}
pub fn run(raw_transaction_hex: String) -> Result<String, Box<dyn Error>> {
    let transaction_bytes = hex::decode(raw_transaction_hex).map_err(|e| format!("Hex decoding error: {}", e))?;
    let mut bytes_slice = transaction_bytes.as_slice();

    let version = read_u32(&mut bytes_slice)?;
    println!("Version: {}", version);

    let input_length = read_compact_size_integer(&mut bytes_slice)?;
    println!("input_length: {}", input_length);

    let mut inputs = vec![];

    for input_number in 0..input_length {
        let txid = read_txid(&mut bytes_slice)?;
        println!("txid[{}] = {:?}",input_number, txid);

        let output_index = read_u32(&mut bytes_slice)?;
        println!("output_index: {}", output_index);

        let script = read_script(&mut bytes_slice)?;
        println!("unlocking_script: {:?}", script);

        let sequence = read_u32(&mut bytes_slice)?;
        println!("sequence: {}", sequence);

        inputs.push(Input {
            txid,
            output_index,
            script,
            sequence,
        });
    }

    let output_count = read_compact_size_integer(&mut bytes_slice)?;
    println!("output_count: {}", output_count);
    let mut outputs: Vec<Output> = vec![];

    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let output_script = read_script(&mut bytes_slice)?;

        outputs.push(Output {
            amount,
            output_script
        });
    }

    let locktime = read_u32(&mut bytes_slice)?;
    println!("locktime: {}", locktime);

    let txid = hash_transaction(&transaction_bytes);
    println!("transaction_id: {:?}", txid);

    let transaction = Transaction {
        txid,
        version,
        inputs,
        outputs, 
        locktime,
    };

    let json_transaction = serde_json::to_string_pretty(&transaction)?;
    Ok(json_transaction)
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