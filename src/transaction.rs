use std::{fmt::{write, Debug}, io::Bytes};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::amount::{Amount, BitcoinValue};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ParseFailed( &'static str),
    UnsuportedSegwithFlag(u8),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::ParseFailed(s) => write!(f, "parse failed: {}", s),
            Error::UnsuportedSegwithFlag(swflag) => write!(f, "unsupported segwit version {}", swflag),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Transaction {
    pub version: Version,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

impl Transaction {
    pub fn txid(&self) -> Txid {
        let mut txid_data = Vec::new();
        self.version.consensus_encode(&mut txid_data).unwrap();
        self.inputs.consensus_encode(&mut txid_data).unwrap();
        self.outputs.consensus_encode(&mut txid_data).unwrap();
        self.locktime.consensus_encode(&mut txid_data).unwrap();
        Txid::new(txid_data)
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
        where
            S: Serializer {
                let mut tx = serializer.serialize_struct("Transaction", 5)?;
                tx.serialize_field("transaction id", &self.txid())?;
                tx.serialize_field("version", &self.version)?;
                tx.serialize_field("inputs", &self.inputs)?;
                tx.serialize_field("outputs", &self.outputs)?;
                tx.serialize_field("locktime", &self.locktime)?;
                tx.end()
        
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Witness {
    content: Vec<Vec<u8>>,
}

impl Witness {
    pub fn new() -> Self {
        Witness { content: vec![] }
    }
    fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl Serialize for Witness {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer 
        {
            use serde::ser::SerializeSeq;

            let mut seq = serializer.serialize_seq(Some(self.content.len()))?;
            for elem in self.content.iter() {
                seq.serialize_element(&hex::encode(&elem))?;
            }
            seq.end()
        }
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TxIn {
    pub previous_txid: Txid,
    pub previous_vout: u32,
    pub script_sig: String,
    pub sequence: u32,
    pub witness: Witness,
}
#[derive(Debug, Serialize)]
pub struct TxOut {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: String,
}

fn as_btc<T: BitcoinValue, S: Serializer>(t: &T, s: S) -> std::prelude::v1::Result<S::Ok, S::Error> {
    let btc = t.to_btc();
    s.serialize_f64(btc)
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn new(data: Vec<u8>) -> Txid {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash1 = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&hash1);
        let hash2 = hasher.finalize();
        
        Txid(hash2.into())
    }
    
}

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
        where
            S: Serializer {
                let mut bytes = self.0.clone();
                bytes.reverse();
                serializer.serialize_str(&hex::encode(bytes))
        
    }
}

#[derive(Debug, Serialize)]
pub struct Version(pub u32);

#[derive(Debug, Serialize)]
pub struct CompactSize(pub u64);

pub trait Encodable {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error>; 
}

impl Encodable for u8 {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = self.to_le_bytes();
        let len = writer.write(bytes.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u16 {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = self.to_le_bytes();
        let len = writer.write(bytes.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u32 {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = self.to_le_bytes();
        let len = writer.write(bytes.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for u64 {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = self.to_le_bytes();
        let len = writer.write(bytes.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for [u8; 32] {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let len = writer.write(self.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for String {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = hex::decode(self).expect("Expected a hex String");
        let mut len = CompactSize(bytes.len() as u64).consensus_encode(writer)?;
        len += writer.write(&bytes).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for Txid {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        Ok(self.0.consensus_encode(writer)?)
    }
}

impl Encodable for Version {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let len = self.0.consensus_encode(writer)?;
        Ok(len)
    }
}

impl Encodable for CompactSize {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        match self.0 {
            0..=0xFC => {
                (self.0 as u8).consensus_encode(writer)?;
                Ok(1)
            }
            0xFD..=0xFFFF => {
                writer.write([0xFD].as_slice()).map_err(Error::Io)?;
                (self.0 as u16).consensus_encode(writer)?;
                Ok(3)
            }
            0x10000..=0xFFFFFFFF => {
                writer.write([0xFE].as_slice()).map_err(Error::Io)?;
                (self.0 as u32).consensus_encode(writer)?;
                Ok(5)
            }
            _ => {
                writer.write([0xFF].as_slice()).map_err(Error::Io)?;
                self.0.consensus_encode(writer)?;
                Ok(9)
            }
        }
    }
}

impl Encodable for TxIn {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let mut len = 0;
        len += self.previous_txid.consensus_encode(writer)?;
        len += self.previous_vout.consensus_encode(writer)?;
        len += self.script_sig.consensus_encode(writer)?;
        len += self.sequence.consensus_encode(writer)?;
        Ok(len)
    }
}

impl Encodable for Vec<TxIn> {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let mut len = 0;
        len += CompactSize(self.len() as u64).consensus_encode(writer)?;
        for tx_in in self.iter() {
            len += tx_in.consensus_encode(writer)?;
        }
        Ok(len)
    }
}

impl Encodable for TxOut {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let mut len = 0;
        len += self.amount.0.consensus_encode(writer)?;
        len += self.script_pubkey.consensus_encode(writer)?;
        Ok(len)
    }
}

impl Encodable for Vec<TxOut> {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let mut len = 0;
        len += CompactSize(self.len() as u64).consensus_encode(writer)?;
        for tx_out in self.iter() {
            len += tx_out.consensus_encode(writer)?;
        }
        Ok(len)
    }
}

pub trait Decodable: Sized {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error>;
}

impl Decodable for u8 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 1];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(u8::from_le_bytes(buffer))
    }
}

impl Decodable for u16 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 2];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(u16::from_le_bytes(buffer))
    }
}

impl Decodable for u32 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(u32::from_le_bytes(buffer))
    }
}

impl Decodable for Version {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        Ok(Version(u32::consensus_decode(reader)?))
    }
}

impl Decodable for u64 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 8];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(u64::from_le_bytes(buffer))
    }
}

impl Decodable for String {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let size = CompactSize::consensus_decode(reader)?;
        println!("string size: {}", &size.0);
        let mut buffer = vec![0_u8; size.0 as usize];

        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(hex::encode(buffer))
    }
}

impl Decodable for CompactSize {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let n = u8::consensus_decode(reader)?;

        match n {
            0xFF => {
                let x = u64::consensus_decode(reader)?;
                Ok(CompactSize(x))
            }
            0xFE => {
                let x = u32::consensus_decode(reader)?;
                Ok(CompactSize(x as u64))
            }
            0xFD => {
                let x = u16::consensus_decode(reader)?;
                Ok(CompactSize(x as u64))
            }
            n => Ok(CompactSize(n as u64)),
        }
    }
}

impl Decodable for Txid {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 32];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(Txid(buffer))
    }
}

impl Decodable for Witness {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut witness_items = vec![];
        let count = u8::consensus_decode(reader)?;
        for _ in 0..count {
            let len = CompactSize::consensus_decode(reader)?.0;
            println!("witness buffer len: {}", len);
            let mut buffer = vec![0; len as usize];
            reader.read(&mut buffer).map_err(Error::Io)?;
            witness_items.push(buffer);
        }
        Ok(Witness{
            content: witness_items
        })
    }
}

impl Decodable for TxIn {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        Ok(TxIn {
            previous_txid: Txid::consensus_decode(reader)?,
            previous_vout: u32::consensus_decode(reader)?,
            script_sig: String::consensus_decode(reader)?,
            sequence: u32::consensus_decode(reader)?,
            witness: Witness::consensus_decode(reader)?,
        })
    }
}
impl Decodable for Vec<TxIn> {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let len = CompactSize::consensus_decode(reader)?;
        let mut vector = Vec::with_capacity(len.0 as usize);
        for _ in 0..len.0 {
            vector.push(TxIn::consensus_decode(reader)?);
        }
        Ok(vector)
    }
}

impl Decodable for TxOut {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        Ok(TxOut {
            amount: Amount::from_sat(u64::consensus_decode(reader)?),
            script_pubkey: String::consensus_decode(reader)?,
        })
    }
}

impl Decodable for Vec<TxOut> {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let len = CompactSize::consensus_decode(reader)?;
        let mut vector = Vec::with_capacity(len.0 as usize);
        for _ in 0..len.0 {
            vector.push(TxOut::consensus_decode(reader)?);
        }
        Ok(vector)
    }
}

impl Decodable for Transaction {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let version = Version::consensus_decode(reader)?;
        let inputs = Vec::<TxIn>::consensus_decode(reader)?; //Marker 00 reads as Zero inputs
        if inputs.is_empty() {
            let segwit_flag = u8::consensus_decode(reader)?;
            match segwit_flag {
                1 => {
                    let mut inputs = Vec::<TxIn>::consensus_decode(reader)?;
                    let outputs = Vec::<TxOut>::consensus_decode(reader)?;
                    for txin in inputs.iter_mut() {
                        txin.witness = Witness::consensus_decode(reader)?;
                    }
                    if !inputs.is_empty() && inputs.iter().all(|input| input.witness.is_empty()) {
                        Err(Error::ParseFailed("Witness flag set but no witnesses present"))
                    } else {
                        Ok( Transaction {
                            version,
                            inputs,
                            outputs,
                            locktime: u32::consensus_decode(reader)?,
                        })
                    }
                }
                x => Err(Error::UnsuportedSegwithFlag(x)),
            }
        // non-segwit
        } else {
            Ok(Transaction {
                version: version,
                inputs: inputs,
                outputs: Vec::<TxOut>::consensus_decode(reader)?,
                locktime: u32::consensus_decode(reader)?,
            })
        }
    }
}

