use core::ops::Deref;

use crate::decoder::CsmWriter;

use crate::decoder::wire::{CsmDecode, CsmEncode};

/// A "flag" type with no payload, which could either exist in the message or not, but will never have any payload.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum CsmFlag {
    #[default]
    No,
    Yes,
}

impl Deref for CsmFlag {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::No => &false,
            Self::Yes => &true,
        }
    }
}

impl From<bool> for CsmFlag {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

impl CsmDecode for CsmFlag {
    fn decode_from_bytes(_data: &[u8]) -> Self {
        CsmFlag::Yes
    }
}

impl CsmEncode for CsmFlag {
    fn encode_param(&self, id: u16, out: &mut CsmWriter) {
        if *self == CsmFlag::Yes {
            out.write_tlv(id, &[]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CsmFlag;

    #[test]
    fn boolean_methods_work_through_owned_and_borrowed_flags() {
        assert_eq!(CsmFlag::No.then_some(7), None);
        let requested = &CsmFlag::Yes;
        assert_eq!(requested.then_some(false), Some(false));
        assert_eq!(requested.then_some(0), Some(0));
        assert!(*CsmFlag::Yes);
        assert!(!*CsmFlag::default());
    }

    #[test]
    fn converts_both_boolean_values_to_flags() {
        assert_eq!(CsmFlag::from(false), CsmFlag::No);
        assert_eq!(CsmFlag::from(true), CsmFlag::Yes);
    }
}
