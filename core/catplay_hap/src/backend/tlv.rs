use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
use core::str;

use log::debug;
use thiserror::Error;

use super::error;

/// Encodes a `Vec<(u8, Vec<u8>)>` in the format `(<Type>, <Value>)` to a `Vec<u8>` of concatenated TLVs.
pub fn encode(tlvs: Vec<(u8, Vec<u8>)>) -> Vec<u8> {
    let mut out = Vec::new();

    for (t, v) in tlvs {
        let mut remaining = v.as_slice();

        while !remaining.is_empty() {
            let chunk_len = remaining.len().min(255);
            out.push(t);
            out.push(chunk_len as u8);
            out.extend_from_slice(&remaining[..chunk_len]);
            remaining = &remaining[chunk_len..];
        }
    }

    out
}

/// Decodes concatenated TLVs into a `HashMap<u8, Vec<u8>>`.
/// If a type appears multiple times in chunks (len=255), the value is concatenated.
pub fn decode(tlv: &[u8]) -> TlvContainer {
    let mut hm = BTreeMap::new();
    let mut i = 0;

    while i + 2 <= tlv.len() {
        let t = tlv[i];
        let l = tlv[i + 1] as usize;
        i += 2;

        if i + l > tlv.len() {
            // Malformed TLV: claimed length too big
            break;
        }

        let chunk = &tlv[i..i + l];
        i += l;

        let buf: &mut Vec<u8> = hm.entry(t).or_default();
        buf.extend_from_slice(chunk);
    }

    hm
}

pub type TlvContainer = BTreeMap<u8, Vec<u8>>;

/// `Encodable` is implemented by types that can be encoded to a to a `Vec<u8>` of concatenated
/// TLVs.
pub trait Encodable {
    fn encode(self) -> Vec<u8>;
}

/// `Type` represents the TLV types defined by the protocol.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Type {
    Method = 0x00,
    Identifier = 0x01,
    Salt = 0x02,
    PublicKey = 0x03,
    Proof = 0x04,
    EncryptedData = 0x05,
    State = 0x06,
    Error = 0x07,
    RetryDelay = 0x08,
    Certificate = 0x09,
    Signature = 0x0A,
    // Permissions = 0x0B,
    FragmentData = 0x0C,
    FragmentLast = 0x0D,
    /// Pairing Type Flags (32 bit unsigned integer).
    Flags = 0x13,
    Separator = 0xFF,
}

/// The variants of `Value` can hold the corresponding values to the types provided by `Type`.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Method(Method),
    Identifier(String),
    Salt([u8; 16]),
    PublicKey(Vec<u8>),
    Proof(Vec<u8>),
    EncryptedData(Vec<u8>),
    State(u8),
    Error(Error),
    RetryDelay(usize),
    Certificate(Vec<u8>),
    Signature(Vec<u8>),
    // Permissions(Permissions),
    FragmentData(Vec<u8>),
    FragmentLast(Vec<u8>),
    Flags(u32),
    Separator,
}

impl Value {
    /// Converts a variant of `Value` to a `(u8, Vec<u8>)` tuple in the format `(Type, Value)`.
    pub fn as_tlv(self) -> (u8, Vec<u8>) {
        match self {
            Value::Method(method) => (Type::Method as u8, vec![method as u8]),
            Value::Identifier(identifier) => (Type::Identifier as u8, identifier.into_bytes()),
            Value::Salt(salt) => (Type::Salt as u8, salt.to_vec()),
            Value::PublicKey(public_key) => (Type::PublicKey as u8, public_key),
            Value::Proof(proof) => (Type::Proof as u8, proof),
            Value::EncryptedData(data) => (Type::EncryptedData as u8, data),
            Value::State(state) => (Type::State as u8, vec![state]),
            Value::Error(error) => (Type::Error as u8, vec![error as u8]),
            Value::RetryDelay(delay) => {
                // TODO: is there proof for correctness of this endian swap ?

                let val = delay as u16;
                let mut vec: Vec<u8> = Vec::new();
                vec.extend_from_slice(&val.to_le_bytes());
                (Type::RetryDelay as u8, vec)
            }
            Value::Certificate(certificate) => (Type::Certificate as u8, certificate),
            Value::Signature(signature) => (Type::Signature as u8, signature),
            // Value::Permissions(permissions) => (Type::Permissions as u8, vec![permissions.as_byte()]),
            Value::FragmentData(fragment_data) => (Type::FragmentData as u8, fragment_data),
            Value::FragmentLast(fragment_last) => (Type::FragmentLast as u8, fragment_last),
            Value::Flags(flags) => {
                // TODO: is there proof for correctness of this endian swap ?

                let mut vec: Vec<u8> = Vec::new();
                vec.extend_from_slice(&flags.to_le_bytes());
                (Type::Flags as u8, vec)
            }
            Value::Separator => (Type::Separator as u8, vec![0x00]),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum Method {
    None,
    MFiPairSetup,
    // PairSetup = 1,
    // PairVerify = 2,
    // AddPairing = 3,
    // RemovePairing = 4,
    // ListPairings = 5,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Error, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    #[error("Generic error to handle unexpected errors.")]
    Unknown = 0x01,
    #[error("Setup code or signature verification failed.")]
    Authentication = 0x02,
    #[error("Client must look at the retry delay TLV item and wait that many seconds before retrying.")]
    Backoff = 0x03,
    #[error("Server cannot accept any more pairings.")]
    MaxPeers = 0x04,
    #[error("Server reached its maximum number of authentication attempts.")]
    MaxTries = 0x05,
    #[error("Server pairing method is unavailable.")]
    Unavailable = 0x06,
    #[error("Server is busy and cannot accept a pairing request at this time.")]
    Busy = 0x07,
}

impl Error {
    pub fn decode(code: u8) -> Option<Self> {
        match code {
            x if x == Self::Unknown as u8 => Some(Self::Unknown),
            x if x == Self::Authentication as u8 => Some(Self::Authentication),
            x if x == Self::Backoff as u8 => Some(Self::Backoff),
            x if x == Self::MaxPeers as u8 => Some(Self::MaxPeers),
            x if x == Self::MaxTries as u8 => Some(Self::MaxTries),
            x if x == Self::Unavailable as u8 => Some(Self::Unavailable),
            x if x == Self::Busy as u8 => Some(Self::Busy),

            _ => None,
        }
    }
}
impl From<error::Error> for Error {
    fn from(err: error::Error) -> Self {
        debug!("Error: {:?}", err);
        Error::Unknown
    }
}

// impl From<io::Error> for Error {
//     fn from(err: io::Error) -> Self {
//         debug!("Error: {:?}", err);
//         Error::Unknown
//     }
// }

// impl From<tokio::task::JoinError> for Error {
//     fn from(err: tokio::task::JoinError) -> Self {
//         error!("{:?}", err);
//         Error::Unknown
//     }
// }

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Self {
        debug!("Error: {:?}", err);
        Error::Unknown
    }
}

impl From<uuid::Error> for Error {
    fn from(err: uuid::Error) -> Self {
        debug!("Error: {:?}", err);
        Error::Unknown
    }
}

impl From<srp::AuthError> for Error {
    fn from(err: srp::AuthError) -> Self {
        debug!("Error: {:?}", err);
        Error::Authentication
    }
}

pub type Container = Vec<Value>;

impl Encodable for Container {
    fn encode(self) -> Vec<u8> {
        encode(self.into_iter().map(|v| v.as_tlv()).collect::<Vec<_>>())
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ErrorContainer {
    pub step: u8,
    pub error: Error,
}

impl ErrorContainer {
    pub fn new(step: u8, error: Error) -> ErrorContainer {
        ErrorContainer { step, error }
    }

    pub fn decode(body: &[u8]) -> Option<ErrorContainer> {
        let tlv = decode(body);
        let step = *tlv
            .get(&(Type::State as u8))
            .and_then(|t| t.first())
            .unwrap_or(&0);
        let error = tlv
            .get(&(Type::Error as u8))
            .and_then(|t| t.first())
            .and_then(|e| Error::decode(*e))?;
        Some(ErrorContainer::new(step, error))
    }
}

impl Encodable for ErrorContainer {
    fn encode(self) -> Vec<u8> {
        vec![Value::State(self.step), Value::Error(self.error)].encode()
    }
}

#[test]
fn test_encode_tlv_chunks() {
    let small_value = vec![0xAA; 10];
    let large_value1 = vec![0xBB; 300];
    let large_value2 = vec![0xCC; 600];

    let encoded = encode(vec![(1, small_value.clone()), (2, large_value1.clone()), (3, large_value2.clone())]);

    let mut expected = Vec::new();

    expected.push(1);
    expected.push(small_value.len() as u8);
    expected.extend_from_slice(&small_value);

    expected.push(2);
    expected.push(255);
    expected.extend_from_slice(&large_value1[..255]);
    expected.push(2);
    expected.push(45);
    expected.extend_from_slice(&large_value1[255..]);

    expected.push(3);
    expected.push(255);
    expected.extend_from_slice(&large_value2[..255]);
    expected.push(3);
    expected.push(255);
    expected.extend_from_slice(&large_value2[255..510]);
    expected.push(3);
    expected.push(90);
    expected.extend_from_slice(&large_value2[510..]);

    assert_eq!(encoded, expected);
}

#[test]
fn test_decode_tlv_chunks() {
    let small_value = vec![0xAA; 10];
    let large_value1 = vec![0xBB; 300];
    let large_value2 = vec![0xCC; 600];

    let encoded = encode(vec![(1, small_value.clone()), (2, large_value1.clone()), (3, large_value2.clone())]);
    let decoded = decode(&encoded);

    assert_eq!(decoded.get(&1).unwrap(), &small_value);
    assert_eq!(decoded.get(&2).unwrap(), &large_value1);
    assert_eq!(decoded.get(&3).unwrap(), &large_value2);
}

#[test]
fn test_decode_handles_truncated_data() {
    // Malformed TLV: claims 10 bytes but only 5 present
    let bad_data = vec![1, 10, 0xAA, 0xBB, 0xCC];
    let decoded = decode(&bad_data);
    // Should not panic; should decode nothing
    assert!(decoded.is_empty());
}
