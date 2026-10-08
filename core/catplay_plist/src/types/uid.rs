use core::fmt;
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Uid {
    value: u64,
}

impl Uid {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
    pub fn get(self) -> u64 {
        self.value
    }
}

impl fmt::Debug for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Uid {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct(crate::PLIST_UID, &self.value)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Uid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Uid;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a plist uid")
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Uid, E> {
                Ok(Uid::new(value))
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Uid, D::Error> {
                u64::deserialize(deserializer).map(Uid::new)
            }
        }
        deserializer.deserialize_newtype_struct(crate::PLIST_UID, Visitor)
    }
}
