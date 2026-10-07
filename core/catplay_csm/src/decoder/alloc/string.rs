extern crate alloc;

use crate::decoder::{CsmDecode, CsmEncode, CsmWriter};
use alloc::string::String;

pub type CsmString = String;

// String
impl CsmDecode for String {
    fn decode_from_bytes(data: &[u8]) -> Self {
        let clean_data = if data.ends_with(&[0]) { &data[..data.len() - 1] } else { data };
        String::from_utf8_lossy(clean_data).into_owned()
    }
}

impl CsmEncode for String {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        self.as_str().encode_param(id, out);
    }
}
