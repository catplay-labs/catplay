use super::{CsmError, CsmParam};

pub struct CsmWriter<'a> {
    callback: &'a mut dyn FnMut(&[u8]),
    pub written: usize,
    pub overflow: bool,
}

impl<'a> CsmWriter<'a> {
    pub fn new(callback: &'a mut dyn FnMut(&[u8])) -> Self {
        Self {
            callback,
            written: 0,
            overflow: false,
        }
    }

    pub fn measure(mut callback: impl FnMut(&mut CsmWriter<'_>)) -> usize {
        let mut noop = |_b: &[u8]| {};
        let mut writer = CsmWriter::new(&mut noop);
        callback(&mut writer);
        writer.written
    }

    pub fn write_params(&mut self, params: &[CsmParam]) {
        for param in params {
            self.write_tlv(param.id, param.value);
        }
    }

    pub fn write_tlv(&mut self, id: u16, value: &[u8]) {
        self.write_tlv_header(id, value.len());
        self.write_data_chunk(value);
    }

    pub fn write_tlv_header(&mut self, id: u16, value_len: usize) {
        let overflow = value_len > u16::MAX as usize - 4;
        // debug_assert!(
        //     !overflow,
        //     "TLV length overflow in CsmWriter::write_tlv_header(): attempted to write header with size {}",
        //     value_len
        // );
        if overflow {
            self.overflow = true;
            return;
        }

        let len = (value_len + 4) as u16;
        self.write_data_chunk(&len.to_be_bytes());
        self.write_data_chunk(&id.to_be_bytes());
    }

    pub fn write_data_chunk(&mut self, data: &[u8]) {
        if let Some(written) = self.written.checked_add(data.len()) {
            self.written = written;
            (self.callback)(data)
        } else {
            self.overflow = true;
        }
    }

    pub fn take_error(&mut self) -> Option<CsmError> {
        if self.overflow {
            return Some(CsmError::Overflow);
        }

        None
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
impl<'a> CsmWriter<'a> {
    pub fn with_vec<T: FnMut(&mut CsmWriter<'_>)>(vec: &mut alloc::vec::Vec<u8>, mut callback: T) -> usize {
        let mut cb = |b: &[u8]| {
            vec.extend_from_slice(b);
        };

        let mut writer = CsmWriter::new(&mut cb);
        callback(&mut writer);
        writer.written
    }

    pub fn serialize<T: FnMut(&mut CsmWriter<'_>)>(mut callback: T) -> alloc::vec::Vec<u8> {
        let size = Self::measure(|w| callback(w));
        let mut vec = alloc::vec::Vec::with_capacity(size);
        Self::with_vec(&mut vec, |w| callback(w));

        debug_assert!(
            vec.len() == size,
            "Measurement violation for Vec in CsmWriter::serialize(): {} vs {}",
            vec.len(),
            size
        );

        vec
    }
}
