extern crate alloc;

use crate::decoder::{CsmAccum, CsmDecode, CsmEncode, CsmParam, CsmWriter};
use alloc::vec::Vec;

pub type CsmVec<T> = Vec<T>;

// CsmAccum for Vec (store repeating parameters)
impl<T: CsmDecode> CsmAccum for Vec<T> {
    fn add_param(&mut self, param: &CsmParam) {
        debug_assert!(
            self.len() < self.capacity(),
            "Capacity violation: prealloc vs add_param mismatch at id {}; len {} vs capacity {}",
            param.id,
            self.len(),
            self.capacity()
        );
        self.push(T::decode_from_bytes(param.value));
    }

    fn prealloc(&mut self, size: usize) {
        *self = Vec::with_capacity(size);
    }
}

// Special case - Vec<T> - multiple params of the same id
impl<T: CsmEncode> CsmEncode for Vec<T> {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        self.as_slice().encode_param(id, out);
    }
}
