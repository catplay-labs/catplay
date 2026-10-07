use catplay_iap2_link::{LinkError, PacketCoderError};
use catplay_util::ArcBox;

pub type CsmSessionResult<T> = Result<T, CsmSessionError>;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum CsmSessionError {
    #[error("IO error: {0}")]
    Io(#[from] ArcBox<std::io::Error>),

    #[error("MFI error: {0}")]
    #[cfg(feature = "mfi")]
    Mfi(#[from] ArcBox<catplay_mfi::MfiI2cError>),

    #[error("iAP2 session error: {0}")]
    Session(String),

    #[error("iAP2 link error: {0:?}")]
    Link(LinkError),

    #[error("iAP2 session terminated")]
    SessionTerminated,

    #[error("iAP2 received corrupted CSM payload")]
    ReceivedCorruptedCsmPayload,

    #[error("iAP2 attempted to serialize unknown CSM message type")]
    AttemptToSerializeUnknownPacket,

    #[error("Failed to encode/decode packet: {0}")]
    PacketCoder(#[from] PacketCoderError),

    #[error("iAP1 transaction error: {0}")]
    LingoTransaction(#[from] catplay_lingo::transaction_helper::TransactionError),

    #[error("iAP1 codec error: {0}")]
    LingoCodec(#[from] catplay_lingo::RegisteredError),

    #[error("Packet transmit drain has been closed/dropped")]
    DrainClosed,
}

impl From<&str> for CsmSessionError {
    fn from(s: &str) -> Self {
        CsmSessionError::Session(s.to_string())
    }
}

impl From<String> for CsmSessionError {
    fn from(s: String) -> Self {
        CsmSessionError::Session(s)
    }
}

impl From<std::io::Error> for CsmSessionError {
    fn from(value: std::io::Error) -> Self {
        CsmSessionError::Io(value.into())
    }
}

#[cfg(feature = "mfi")]
impl From<catplay_mfi::MfiI2cError> for CsmSessionError {
    fn from(value: catplay_mfi::MfiI2cError) -> Self {
        CsmSessionError::Mfi(value.into())
    }
}

impl From<LinkError> for CsmSessionError {
    fn from(value: LinkError) -> Self {
        CsmSessionError::Link(value)
    }
}
