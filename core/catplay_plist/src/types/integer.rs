use core::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Integer {
    value: i128,
}

impl Integer {
    pub fn as_signed(self) -> Option<i64> {
        i64::try_from(self.value).ok()
    }
    pub fn as_unsigned(self) -> Option<u64> {
        u64::try_from(self.value).ok()
    }
}

impl fmt::Debug for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

macro_rules! from_int {
    ($($ty:ty),*) => {$ (
        impl From<$ty> for Integer {
            fn from(value: $ty) -> Self { Self { value: value.into() } }
        }
    )*};
}
from_int!(i8, i16, i32, i64, u8, u16, u32, u64);

#[cfg(feature = "serde")]
impl Serialize for Integer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if let Some(value) = self.as_unsigned() {
            serializer.serialize_u64(value)
        } else {
            serializer.serialize_i64(self.as_signed().expect("integer range"))
        }
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Integer {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = Integer;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a plist integer")
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Integer, E> {
                Ok(value.into())
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Integer, E> {
                Ok(value.into())
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}
