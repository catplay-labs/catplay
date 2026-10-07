#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DecodeError {
    #[error("{path}: {source}")]
    AtPath {
        path: catplay_path::PathBuf,
        source: alloc::boxed::Box<DecodeError>,
    },
    #[error("unexpected end of payload: need {needed} bytes, have {remaining}")]
    UnexpectedEof { needed: usize, remaining: usize },
    #[error("invalid boolean value {0}")]
    InvalidBool(u8),
    #[error("invalid enum value {value}")]
    InvalidEnumValue { value: u64 },
    #[error("unknown {token} token ({fid_type}, {fid_subtype})")]
    UnknownToken {
        token: &'static str,
        fid_type: u8,
        fid_subtype: u8,
    },
    #[error("invalid field value for {field}")]
    InvalidFieldValue { field: &'static str },
    #[error("invalid UTF-8 in payload")]
    InvalidUtf8,
    #[error("string is missing its terminating NUL byte")]
    MissingNullTerminator,
    #[error("{remaining} trailing payload bytes")]
    TrailingData { remaining: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EncodeError {
    #[error("encoded payload failed validation: {0}")]
    RoundTripDecode(DecodeError),
    #[error("encoded payload decodes to a different message")]
    RoundTripMismatch,
    #[error("invalid enum value {value}")]
    InvalidEnumValue { value: u64 },
    #[error("missing conditional field {field}")]
    MissingConditionalField { field: &'static str },
    #[error("unexpected conditional field {field}")]
    UnexpectedConditionalField { field: &'static str },
    #[error("invalid field value for {field}")]
    InvalidFieldValue { field: &'static str },
}

impl DecodeError {
    /// The deepest decoding location, when decoding a schema record or message.
    pub fn path(&self) -> Option<&catplay_path::PathBuf> {
        match self {
            Self::AtPath { path, .. } => Some(path),
            _ => None,
        }
    }

    pub fn cause(&self) -> &Self {
        match self {
            Self::AtPath { source, .. } => source.cause(),
            _ => self,
        }
    }

    pub(crate) fn at_path(self, path: &catplay_path::Path<'_>) -> Self {
        if matches!(self, Self::AtPath { .. }) {
            self
        } else {
            Self::AtPath {
                path: path.into(),
                source: alloc::boxed::Box::new(self),
            }
        }
    }
}
