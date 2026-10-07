use crate::decoder::{CsmEncode, CsmWriter};

// Special case - Vec<T> - multiple params of the same id
impl<T: CsmEncode> CsmEncode for [T] {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        for item in self {
            item.encode_param(id, out);
        }
    }
}
