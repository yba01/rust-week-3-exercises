// use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
// use std::fmt::{self, write};
use std::fmt::{self};

use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        Self { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:
        // [0x00–0xFC] => 1 byte
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)

        match self.value {
            0..=252 => vec![self.value as u8],
            253..=0xFFFF => {
                let mut v = vec![0xFD];
                v.extend_from_slice(&(self.value as u16).to_le_bytes());
                v
            }
            0x10000..=0xFFFF_FFFF => {
                let mut v = vec![0xFE];
                v.extend_from_slice(&(self.value as u32).to_le_bytes());
                v
            }
            _ => {
                let mut v = vec![0xFF];
                v.extend_from_slice(&(self.value).to_le_bytes());
                v
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        // Check that enough bytes are available based on prefix.

        let (value, size) = match bytes {
            [] => return Err(BitcoinError::InvalidFormat),
            [n @ 0x00..=0xFC, ..] => (*n as u64, 1),
            [0xFD, b1, b2, ..] => (u16::from_le_bytes([*b1, *b2]) as u64, 3),
            [0xFE, b1, b2, b3, b4, ..] => (u32::from_le_bytes([*b1, *b2, *b3, *b4]) as u64, 5),
            [0xFF, b1, b2, b3, b4, b5, b6, b7, b8, ..] => (
                u64::from_le_bytes([*b1, *b2, *b3, *b4, *b5, *b6, *b7, *b8]),
                9,
            ),
            _ => return Err(BitcoinError::InsufficientBytes),
        };
        Ok((CompactSize::new(value), size))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        let hex_encoded = hex::encode(self.0);
        serializer.serialize_str(&hex_encoded)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32

        let hex_str = String::deserialize(deserializer)?;
        let decoded = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;

        if decoded.len() != 32 {
            return Err(serde::de::Error::custom("Txid must be 32 bytes"));
        }

        let table = decoded.try_into().unwrap();
        Ok(Txid(table))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]

pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        Self {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.txid.0);
        bytes.extend_from_slice(&self.vout.to_le_bytes());

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() != 36 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let txid: [u8; 32] = bytes[0..32].try_into().unwrap();

        let vout_byte: [u8; 4] = bytes[32..36].try_into().unwrap();
        let vout: u32 = u32::from_le_bytes(vout_byte);

        Ok((OutPoint::new(txid, vout), 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Self { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let mut vec: Vec<u8> = Vec::new();

        let len = self.bytes.len() as u64;

        vec.extend_from_slice(&CompactSize::new(len).to_bytes());
        vec.extend_from_slice(&self.bytes);

        vec
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes

        let (_, len) = CompactSize::from_bytes(bytes)?;

        let vec = bytes[len..].to_vec();

        Ok((Script::new(vec), bytes.len()))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]

pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        // TODO: Basic constructor
        Self {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut byte: Vec<u8> = Vec::new();
        byte.extend_from_slice(&self.previous_output.to_bytes());
        byte.extend_from_slice(&self.script_sig.to_bytes());
        byte.extend_from_slice(&self.sequence.to_le_bytes());

        byte
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let len = bytes.len();
        let output = OutPoint::from_bytes(&bytes[0..36])?;
        let (script, _) = Script::from_bytes(&bytes[36..len - 4])?;
        let sequence = u32::from_le_bytes(bytes[len - 4..len].try_into().unwrap());
        Ok((TransactionInput::new(output.0, script, sequence), len))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]

pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        // TODO: Construct a transaction from parts
        Self {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        // - version (4 bytes LE)
        // - CompactSize (number of inputs)
        // - each input serialized
        // - lock_time (4 bytes LE)

        let mut vec: Vec<u8> = Vec::new();

        vec.extend_from_slice(&self.version.to_le_bytes());
        vec.extend_from_slice(&self.inputs[0].to_bytes());
        vec.extend_from_slice(&self.lock_time.to_le_bytes());

        vec
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Read version, CompactSize for input count
        // Parse inputs one by one
        // Read final 4 bytes for lock_time
        let len = bytes.len();
        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let (inputs, _) = TransactionInput::from_bytes(&bytes[4..len - 4])?;
        let lock = u32::from_le_bytes(bytes[len - 4..len].try_into().unwrap());
        Ok((BitcoinTransaction::new(version, vec![inputs], lock), len))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        write!(
            f,
            "Version: {}, Lock Time: {}, Previous Output Vout: {:?}, ScriptSig length: {}, ScriptSig bytes: {}, The transaction input sequence: {}",
            self.version,
            self.lock_time,
            self.inputs[0].previous_output.vout,
            self.inputs[0].script_sig.len(),
            self.inputs[0].script_sig.len() + size_of::<Script>(),
            self.inputs[0].sequence
        )
    }
}
