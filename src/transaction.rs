use core::slice::SlicePattern;
use std::{fmt::{Debug, *}, fs::read};
use clap::{builder, error};
use serde::{ser::SerializeStruct, Serialize, Serializer};
use serde_json::de::Read;
use sha2::{Digest, Sha256};
use std::io::Read;

use crate::amount::{Amount, BitcoinValue};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Io(ref e) => write!(f, "IO error: {}", e)
        }
    }
}

impl std::error::Error for Error {}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TxIn {
    pub previous_txid: Txid,
    pub previous_vout: u32,
    pub script_sig: String,
    pub sequence: u32,
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
pub struct Transaction {
    pub txid: Txid,
    pub version: Version,
    pub inputs: Vec<TxIn>,
    pub outputs: Vec<TxOut>,
    pub locktime: u32,
}

impl Transaction {
    pub fn txid(&self) -> Txid {
        let txid_data = Vec::new();
        self.version.consensus_encode(&mut txi)
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

#[derive(Debug)]
pub struct Txid([u8; 32]);

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }
}

impl Txid {
    pub fn new(data: Vec<u8>) -> Txid {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let hash1 = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(&hash1);
        let hash2 = hasher.finalize();
        
        Txid::from_bytes(hash2.into())
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

impl Encodable for u32 {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let bytes = self.to_le_bytes();
        let len = writer.write(bytes.as_slice()).map_err(Error::Io)?;
        Ok(len)
    }
}

impl Encodable for Version {
    fn consensus_encode<W: std::io::Write>(&self, writer: &mut W) -> std::prelude::v1::Result<usize, Error> {
        let len = self.0.consensus_encode(writer)?;
        Ok(len)
    }
}

pub trait Decodable: Sized {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error>;
}

impl Decodable for u8 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 1];
        reader.read_exact(buffer).map_err(Error::Io)?;
        Ok(u8::from_be_bytes(buffer))
    }
}

impl Decodable for u16 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 2];
        reader.read_exact(buffer).map_err(Error::Io)?;
        Ok(u16::from_be_bytes(buffer))
    }
}

impl Decodable for u32 {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        let mut buffer = [0; 4];
        reader.read_exact(&mut buffer).map_err(Error::Io)?;
        Ok(u32::from_be_bytes(buffer))
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
        reader.read_exact(buffer).map_err(Error::Io)?;
        Ok(u64::from_be_bytes(buffer))
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

impl Decodable for TxIn {
    fn consensus_decode<R: std::io::Read>(reader: &mut R) -> std::prelude::v1::Result<Self, Error> {
        Ok(TxIn {
            previous_txid: Txid::consensus_decode(reader)?,
            previous_vout: u32::consensus_decode(reader)?,
            script_sig: String::consensus_decode(reader)?,
            sequence: u32::consensus_decode(reader)?,
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
        Ok(Transaction {
            version: Version::consensus_decode(reader)?,
            inputs: Vec::<TxIn>::consensus_decode(reader)?,
            outputs: Vec::<TxOut>::consensus_decode(reader)?,
            locktime: u32::consensus_decode(reader)?,
        })
    }
}

