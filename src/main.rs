use std::io::Read;
use std::fmt::Debug;
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct Input {
    txid: String,
    output_index: u32,
    script: String,
    sequence: u32,
}
#[derive(Debug, Serialize)]

pub struct Amount(u64);
impl Amount {
    pub fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}

#[derive(Debug, Serialize)]
struct Output {
    amount: f64,
    output_script: String,
}

#[derive(Debug, Serialize)]
struct Transaction {
    version: u32,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
}

fn read_u32(bytes_slice: &mut &[u8]) -> u32 {
    // Read slice into a buffer
    let mut buffer = [0; 4];
    bytes_slice.read(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}

fn read_u64(bytes_slice: &mut &[u8]) -> u64 {
    let mut buffer = [0; 8];
    bytes_slice.read(&mut buffer).unwrap();
    u64::from_le_bytes(buffer)
}

fn main() {
    let transaction_hex = "010000000242d5c1d6f7308bbe95c0f6e1301dd73a8da77d2155b0773bc297ac47f9cd7380010000006a4730440220771361aae55e84496b9e7b06e0a53dd122a1425f85840af7a52b20fa329816070220221dd92132e82ef9c133cb1a106b64893892a11acf2cfa1adb7698dcdc02f01b0121030077be25dc482e7f4abad60115416881fe4ef98af33c924cd8b20ca4e57e8bd5feffffff75c87cc5f3150eefc1c04c0246e7e0b370e64b17d6226c44b333a6f4ca14b49c000000006b483045022100e0d85fece671d367c8d442a96230954cdda4b9cf95e9edc763616d05d93e944302202330d520408d909575c5f6976cc405b3042673b601f4f2140b2e4d447e671c47012103c43afccd37aae7107f5a43f5b7b223d034e7583b77c8cd1084d86895a7341abffeffffff02ebb10f00000000001976a9144ef88a0b04e3ad6d1888da4be260d6735e0d308488ac508c1e000000000017a91476c0c8f2fc403c5edaea365f6a284317b9cdf7258700000000";
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();

    let version = read_u32(&mut bytes_slice);
    println!("Version: {}", version);

    let input_length = read_compact_size_integer(&mut bytes_slice);
    println!("input_length: {}", input_length);

    let mut inputs = vec![];

    for input_number in 0..input_length {
        let txid = read_txid(&mut bytes_slice);
        println!("txid[{}] = {:?}",input_number, txid);

        let output_index = read_u32(&mut bytes_slice);
        println!("output_index: {}", output_index);

        let script = read_script(&mut bytes_slice);
        println!("unlocking_script: {:?}", script);

        let sequence = read_u32(&mut bytes_slice);
        println!("sequence: {}", sequence);

        inputs.push(Input {
            txid,
            output_index,
            script,
            sequence,
        });
    }

    let output_count = read_compact_size_integer(&mut bytes_slice);
    println!("output_count: {}", output_count);
    let mut outputs: Vec<Output> = vec![];

    for _ in 0..output_count {
        let amount = Amount(read_u64(&mut bytes_slice)).to_btc();
        let output_script = read_script(&mut bytes_slice);

        outputs.push(Output {
            amount,
            output_script
        });
    }


    let transaction = Transaction {
        version,
        inputs,
        outputs
    };

    let json_transaction = serde_json::to_string_pretty(&transaction).unwrap();
    println!("Transaction: {}", json_transaction);
}

fn read_script(bytes_slice: &mut &[u8]) -> String {
    let script_size = read_compact_size_integer(bytes_slice) as usize;
    println!("script_size: {}", script_size);
    let mut buffer = vec![0_u8; script_size];

    bytes_slice.read(&mut buffer).unwrap();
    hex::encode(buffer)
}

fn read_txid(bytes_slice:&mut &[u8]) -> String {
    let mut buffer = [0; 32];
    _ = bytes_slice.read(&mut buffer);
    buffer.reverse();
    hex::encode(buffer)
}

pub fn read_compact_size_integer(bytes_slice: &mut &[u8]) -> u64 {
    let mut compact_size = [0; 1];
    bytes_slice.read(&mut compact_size).unwrap();
    
    match compact_size[0] {
        0..=252 => compact_size[0] as u64,
        253 => {
            let mut buffer = [0; 2];
            bytes_slice.read(&mut buffer).unwrap();
            u16::from_le_bytes(buffer) as u64
        },
        254 => {
            let mut buffer = [0; 4];
            bytes_slice.read(&mut buffer).unwrap();
            u32::from_le_bytes(buffer) as u64
        },
        255 => {
            let mut buffer = [0; 8];
            bytes_slice.read(&mut buffer).unwrap();
            u64::from_le_bytes(buffer) as u64
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::read_compact_size_integer;

    #[test]
    fn test_read_compact_size_integer_one_byte() {
        let mut bytes = [1_u8].as_slice();
        let length = read_compact_size_integer(&mut bytes);
        assert_eq!(length, 1_u64);
    }

    #[test]
    fn test_read_compact_size_integer_three_bytes() {
        let mut bytes = [253_u8, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes);
        assert_eq!(length, 256_u64);
    }

    #[test]
    fn test_read_compact_size_integer_five_bytes() {
        let mut bytes = [254_u8, 0, 0, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes);
        assert_eq!(length, 256_u64.pow(3));
    }

    #[test]
    fn test_read_compact_size_integer_nine_bytes() {
        let mut bytes = [255_u8, 0, 0, 0, 0, 0, 0, 0, 1].as_slice();
        let length = read_compact_size_integer(&mut bytes);
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
        let length = read_compact_size_integer(&mut bytes);
        let expected_length = 20_000_u64;
        assert_eq!(length, expected_length);
    }
}