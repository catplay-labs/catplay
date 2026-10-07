use super::EncodeError;
use alloc::vec::Vec;

pub struct Writer<'a> {
    output: &'a mut Vec<u8>,
}

impl<'a> Writer<'a> {
    pub fn new(output: &'a mut Vec<u8>) -> Self {
        Self { output }
    }
    pub fn position(&self) -> usize {
        self.output.len()
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.output.extend_from_slice(bytes);
    }
    pub fn write_u8(&mut self, value: u8) {
        self.output.push(value);
    }
    pub fn write_i8(&mut self, value: i8) {
        self.write_u8(value as u8);
    }
    pub fn write_u16_be(&mut self, value: u16) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_i16_be(&mut self, value: i16) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_u32_be(&mut self, value: u32) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_i32_be(&mut self, value: i32) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_u64_be(&mut self, value: u64) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_i64_be(&mut self, value: i64) {
        self.write_bytes(&value.to_be_bytes());
    }
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }
    pub fn write_tagged_token(
        &mut self,
        fid_type: u8,
        fid_subtype: u8,
        write_payload: impl FnOnce(&mut Self) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        let start = self.output.len();
        self.write_bytes(&[0, fid_type, fid_subtype]);
        if let Err(error) = write_payload(self) {
            self.output.truncate(start);
            return Err(error);
        }
        let length = self.output.len() - start - 1;
        let Ok(length) = u8::try_from(length) else {
            self.output.truncate(start);
            return Err(EncodeError::InvalidFieldValue { field: "token length" });
        };
        self.output[start] = length;
        Ok(())
    }
}
