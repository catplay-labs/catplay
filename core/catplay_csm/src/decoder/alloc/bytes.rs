extern crate alloc;

use crate::decoder::{CsmDecode, CsmEncode, CsmWriter};
use alloc::vec::Vec;
use core::fmt;

// CsmByteArray

/// As `Vec<X>` has been reserved for collecting repeating parameters, supporting `Vec<u8>` became very problematic.
///
/// It is currently not valid to use `Vec<u8>` type as it will assume you want to parse embedded TLV.
///
/// Additionally, it has a default Debug printer that's very verbose and would be annoying for debugging.
///
/// For that reasons, a specialized type is created here for byte arrays (like certificates etc.)
#[derive(Clone, PartialEq, Default)]
pub struct CsmByteArray {
    pub data: Vec<u8>,
}

impl CsmByteArray {
    pub fn new(data: Vec<u8>) -> Self {
        CsmByteArray { data }
    }
}

impl fmt::Debug for CsmByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CsmByteArray [")?;
        for chunk in self.data.chunks(8) {
            write!(f, "    ")?;
            for byte in chunk {
                write!(f, "{:02X} ", byte)?;
            }
            writeln!(f)?;
        }
        write!(f, "]")
    }
}

impl From<Vec<u8>> for CsmByteArray {
    fn from(value: Vec<u8>) -> Self {
        CsmByteArray::new(value.clone())
    }
}

impl From<&[u8]> for CsmByteArray {
    fn from(value: &[u8]) -> Self {
        CsmByteArray::new(value.into())
    }
}

impl<const N: usize> From<[u8; N]> for CsmByteArray {
    fn from(arr: [u8; N]) -> Self {
        CsmByteArray::new(arr.to_vec())
    }
}

// CsmByteArray - decode/encode
impl CsmDecode for CsmByteArray {
    fn decode_from_bytes(data: &[u8]) -> Self {
        CsmByteArray::new(data.to_vec())
    }
}

// Special case: CsmByteArray - not to be confused with Vec<u8>!
impl CsmEncode for CsmByteArray {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        CsmEncode::encode_param(&self.data.as_slice(), id, out);
    }
}
