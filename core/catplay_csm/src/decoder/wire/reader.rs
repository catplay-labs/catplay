use super::{CsmError, CsmParam};

pub struct CsmReader<'a> {
    data: &'a [u8],
    underflow: bool,
    overflow: bool,
}

impl<'a> CsmReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            overflow: false,
            underflow: false,
        }
    }

    pub fn count_repeating(&self, id: u16) -> usize {
        let mut count = 0;
        let mut reader = Self::new(self.data);
        let mut count_param = |param: CsmParam<'a>| {
            if param.id == id {
                count += 1;
            }
        };
        reader.stream_all_erased(&mut count_param);
        count
    }

    pub fn stream_all(&mut self, mut callback: impl FnMut(CsmParam<'a>)) {
        self.stream_all_erased(&mut callback);
    }

    pub(crate) fn stream_all_erased(&mut self, callback: &mut dyn FnMut(CsmParam<'a>)) {
        let mut data = self.data;
        while data.len() >= 4 {
            let len = u16::from_be_bytes([data[0], data[1]]) as usize;
            let id = u16::from_be_bytes([data[2], data[3]]);
            if data.len() < len || len < 4 {
                self.overflow = true;
                break;
            }

            callback(CsmParam::new(id, &data[4..len]));

            data = &data[len..];
        }

        self.underflow = !data.is_empty();
    }

    pub fn take_error(&mut self) -> Option<CsmError> {
        if self.underflow {
            return Some(CsmError::Underflow);
        } else if self.overflow {
            return Some(CsmError::Overflow);
        }

        None
    }
}
