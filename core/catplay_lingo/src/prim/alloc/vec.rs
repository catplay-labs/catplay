use crate::{DecodeError, EncodeError, Iap1Decode, Iap1DecodeCounted, Iap1DecodeRemaining, Iap1Encode, Iap1EncodeCounted, Reader, Writer};
use alloc::vec::Vec;

impl<'a, T: Iap1Decode> Iap1DecodeCounted<'a> for Vec<T> {
    fn decode_counted(reader: &mut Reader<'a>, count: usize) -> Result<Self, DecodeError> {
        let mut result = Vec::with_capacity(count.min(reader.remaining()));
        reader.read_counted_erased(count, (&mut result as *mut Vec<T>).cast(), decode_and_push::<T>)?;
        Ok(result)
    }
}

unsafe fn decode_and_push<T: Iap1Decode>(output: *mut (), reader: &mut Reader<'_>) -> Result<(), DecodeError> {
    let value = T::decode(reader)?;
    // SAFETY: `read_counted_erased` receives this callback alongside a `Vec<T>` pointer.
    unsafe { (&mut *output.cast::<Vec<T>>()).push(value) };
    Ok(())
}

impl<T: Iap1Encode> Iap1EncodeCounted for Vec<T> {
    fn encode_counted(&self, writer: &mut Writer<'_>, count: usize, field: &'static str) -> Result<(), EncodeError> {
        self.as_slice().encode_counted(writer, count, field)
    }
}

impl<T: Iap1Decode> Iap1DecodeRemaining for Vec<T> {
    fn decode_remaining(reader: &mut Reader<'_>, field: &'static str) -> Result<Self, DecodeError> {
        let mut result = Vec::new();
        reader.read_remaining_erased(field, (&mut result as *mut Vec<T>).cast(), decode_and_push::<T>)?;
        Ok(result)
    }
}
