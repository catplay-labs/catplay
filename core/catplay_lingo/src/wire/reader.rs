use super::{DecodeError, Iap1Decode};
use alloc::vec::Vec;

pub struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
    path: Option<&'a catplay_path::Path<'a>>,
}

type ErasedVecDecoder = for<'a> unsafe fn(*mut (), &mut Reader<'a>) -> Result<(), DecodeError>;

impl<'a> Reader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            offset: 0,
            path: None,
        }
    }
    /// Run a decoder with a stack-local field frame.
    pub fn with_field<T>(
        &mut self,
        key: &'static str,
        decode: impl FnOnce(&mut Reader<'_>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        let root = catplay_path::Path::root();
        let path = self.path.unwrap_or(&root).field(key);
        self.with_path(&path, decode)
    }

    pub(crate) fn with_field_fn<T>(
        &mut self,
        key: &'static str,
        decode: &dyn for<'r> Fn(&mut Reader<'r>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        let root = catplay_path::Path::root();
        let path = self.path.unwrap_or(&root).field(key);
        self.with_path_fn(&path, decode)
    }

    /// Add the initial type name only when no parent decoder is active.
    pub fn with_root<T>(
        &mut self,
        name: &'static str,
        decode: impl FnOnce(&mut Reader<'_>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        if self.path.is_some() {
            decode(self)
        } else {
            self.with_field(name, decode)
        }
    }

    pub(crate) fn with_root_fn<T>(
        &mut self,
        name: &'static str,
        decode: &dyn for<'r> Fn(&mut Reader<'r>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        if self.path.is_some() {
            decode(self)
        } else {
            self.with_field_fn(name, decode)
        }
    }

    fn with_index<T>(
        &mut self,
        index: usize,
        decode: &dyn for<'r> Fn(&mut Reader<'r>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        if let Some(parent) = self.path {
            let path = parent.index(index);
            self.with_path_fn(&path, decode)
        } else {
            decode(self)
        }
    }

    fn with_path<T>(
        &mut self,
        path: &catplay_path::Path<'_>,
        decode: impl FnOnce(&mut Reader<'_>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        let mut child = Reader {
            input: self.input,
            offset: self.offset,
            path: Some(path),
        };
        let result = decode(&mut child).map_err(|error| error.at_path(path));
        self.offset = child.offset;
        result
    }

    fn with_path_fn<T>(
        &mut self,
        path: &catplay_path::Path<'_>,
        decode: &dyn for<'r> Fn(&mut Reader<'r>) -> Result<T, DecodeError>,
    ) -> Result<T, DecodeError> {
        let mut child = Reader {
            input: self.input,
            offset: self.offset,
            path: Some(path),
        };
        let result = decode(&mut child).map_err(|error| error.at_path(path));
        self.offset = child.offset;
        result
    }

    pub fn check_finished(&self) -> Result<(), DecodeError> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            let error = DecodeError::TrailingData {
                remaining: self.remaining(),
            };
            Err(match self.path {
                Some(path) => error.at_path(path),
                None => error,
            })
        }
    }
    pub fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }
    pub fn position(&self) -> usize {
        self.offset
    }
    pub fn finish(self) -> Result<(), DecodeError> {
        self.check_finished()
    }
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], DecodeError> {
        if self.remaining() < count {
            return Err(DecodeError::UnexpectedEof {
                needed: count,
                remaining: self.remaining(),
            });
        }
        let start = self.offset;
        self.offset += count;
        Ok(&self.input[start..self.offset])
    }
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.read_bytes(1)?[0])
    }
    pub fn read_i8(&mut self) -> Result<i8, DecodeError> {
        Ok(self.read_u8()? as i8)
    }
    pub fn read_u16_be(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_i16_be(&mut self) -> Result<i16, DecodeError> {
        Ok(i16::from_be_bytes(self.read_bytes(2)?.try_into().unwrap()))
    }
    pub fn read_u32_be(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_i32_be(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }
    pub fn read_u64_be(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
    pub fn read_i64_be(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_be_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(DecodeError::InvalidBool(value)),
        }
    }
    pub fn read_tagged_token(&mut self) -> Result<(u8, u8, Reader<'a>), DecodeError> {
        let length = usize::from(self.read_u8()?);
        if length < 2 {
            return Err(DecodeError::InvalidFieldValue { field: "token length" });
        }
        let mut token = Reader::new(self.read_bytes(length)?);
        token.path = self.path;
        let fid_type = token.read_u8()?;
        let fid_subtype = token.read_u8()?;
        Ok((fid_type, fid_subtype, token))
    }

    pub fn read_counted<T: Iap1Decode>(&mut self, count: usize) -> Result<Vec<T>, DecodeError> {
        let mut result = Vec::with_capacity(count.min(self.remaining()));
        for index in 0..count {
            result.push(self.with_index(index, &T::decode)?);
        }
        Ok(result)
    }

    #[doc(hidden)]
    #[inline(never)]
    pub(crate) fn read_counted_erased(&mut self, count: usize, output: *mut (), decode: ErasedVecDecoder) -> Result<(), DecodeError> {
        for index in 0..count {
            self.with_index(index, &|reader| {
                // SAFETY: caller pairs the output Vec<T> with its typed append callback.
                unsafe { decode(output, reader) }
            })?;
        }
        Ok(())
    }
    pub fn read_remaining<T: Iap1Decode>(&mut self, field: &'static str) -> Result<Vec<T>, DecodeError> {
        let mut result = Vec::new();
        while self.remaining() > 0 {
            let before = self.position();
            let item = self.with_index(result.len(), &|reader| {
                let item = T::decode(reader)?;
                if reader.position() == before {
                    return Err(DecodeError::InvalidFieldValue { field });
                }
                Ok(item)
            })?;
            result.push(item);
        }
        Ok(result)
    }

    #[doc(hidden)]
    #[inline(never)]
    pub(crate) fn read_remaining_erased(
        &mut self,
        field: &'static str,
        output: *mut (),
        decode: ErasedVecDecoder,
    ) -> Result<(), DecodeError> {
        let mut index = 0;
        while self.remaining() > 0 {
            let before = self.position();
            self.with_index(index, &|reader| {
                // SAFETY: caller pairs the output Vec<T> with its typed append callback.
                unsafe { decode(output, reader)? };
                if reader.position() == before {
                    return Err(DecodeError::InvalidFieldValue { field });
                }
                Ok(())
            })?;
            index += 1;
        }
        Ok(())
    }
}
