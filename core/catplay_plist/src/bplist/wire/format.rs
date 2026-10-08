//! Wire constants and integer/trailer layout shared by the reader and writer.

pub const HEADER: &[u8; 8] = b"bplist00";
pub const TRAILER_SIZE: usize = 32;

pub mod marker {
    pub const FALSE: u8 = 0x08;
    pub const TRUE: u8 = 0x09;
    pub const INTEGER: u8 = 0x10;
    pub const REAL: u8 = 0x20;
    pub const DATE: u8 = 0x30;
    pub const DATA: u8 = 0x40;
    pub const ASCII: u8 = 0x50;
    pub const UTF16: u8 = 0x60;
    pub const UID: u8 = 0x80;
    pub const ARRAY: u8 = 0xa0;
    pub const DICTIONARY: u8 = 0xd0;
    pub const EXTENDED_LENGTH: u8 = 0x0f;
}

/// The writer emits canonical widths; the reader also accepts three-byte widths.
pub fn integer_width(value: u64) -> usize {
    if value <= u8::MAX as u64 {
        1
    } else if value <= u16::MAX as u64 {
        2
    } else if value <= u32::MAX as u64 {
        4
    } else {
        8
    }
}

pub fn valid_read_width(width: usize) -> bool {
    matches!(width, 1 | 2 | 3 | 4 | 8)
}

pub fn read_integer(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(0u64, |value, byte| (value << 8) | u64::from(*byte))
}

#[derive(Clone, Copy, Debug)]
pub struct Trailer {
    pub offset_width: u8,
    pub ref_width: u8,
    pub object_count: u64,
    pub root: u64,
    pub table_offset: u64,
}

impl Trailer {
    /// The caller ensures that `bytes` is the final 32 bytes of the input.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            offset_width: bytes[6],
            ref_width: bytes[7],
            object_count: read_integer(&bytes[8..16]),
            root: read_integer(&bytes[16..24]),
            table_offset: read_integer(&bytes[24..32]),
        }
    }

    pub fn to_bytes(self) -> [u8; TRAILER_SIZE] {
        let mut bytes = [0u8; TRAILER_SIZE];
        bytes[6] = self.offset_width;
        bytes[7] = self.ref_width;
        bytes[8..16].copy_from_slice(&self.object_count.to_be_bytes());
        bytes[16..24].copy_from_slice(&self.root.to_be_bytes());
        bytes[24..32].copy_from_slice(&self.table_offset.to_be_bytes());
        bytes
    }
}
