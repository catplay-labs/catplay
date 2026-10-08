use std::io;

#[derive(thiserror::Error, Debug)]
pub enum MfiI2cError {
    #[error("MFI i2c read timeout (reg=0x{reg:02X}, n={n}, tries={tries}, status={status})")]
    ReadTimeout {
        reg: u8,
        n: usize,
        tries: usize,
        status: io::Error,
    },
    #[error("MFI i2c write timeout (reg=0x{reg:02X}, n={n} tries={tries}, status={status})")]
    WriteTimeout {
        reg: u8,
        n: usize,
        tries: usize,
        status: io::Error,
    },

    #[error("MFI I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("MFI signing status: status 0x{status:02X}, error 0x{code:02X}")]
    SigningError { status: u8, code: u8 },

    #[error("MFI unexpected data size: {0}")]
    UnexpectedSize(usize),

    #[cfg(feature = "local")]
    #[error("Invalid local MFi P-256 PKCS#8 private key")]
    InvalidLocalKey,

    #[cfg(feature = "local")]
    #[error("Invalid local MFi challenge size: expected 32 bytes, got {0}")]
    InvalidChallengeSize(usize),

    #[cfg(feature = "local")]
    #[error("Local MFi ECDSA signing failed")]
    LocalSigning,

    #[error("Other MFI error: {0}")]
    Other(String),

    #[error("{0}")]
    Remote(String),
}

impl From<&str> for MfiI2cError {
    fn from(s: &str) -> Self {
        MfiI2cError::Other(s.to_string())
    }
}

impl From<String> for MfiI2cError {
    fn from(s: String) -> Self {
        MfiI2cError::Other(s)
    }
}

pub type MfiResult<T> = Result<T, MfiI2cError>;
