use alloc::string::String;
use core::ops::Deref;
use serde::{Deserialize, Deserializer};

/// An unsigned integer accepting decimal strings sent by nonconforming CarPlay head units.
/// Serializes as an integer, including when the input was a string.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct FlexU64(u64);

impl<'de> Deserialize<'de> for FlexU64 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Int(u64),
            Str(String),
        }
        match Repr::deserialize(deserializer)? {
            Repr::Int(value) => Ok(Self(value)),
            Repr::Str(value) => value.parse().map(Self).map_err(serde::de::Error::custom),
        }
    }
}

impl From<u64> for FlexU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<FlexU64> for u64 {
    fn from(value: FlexU64) -> Self {
        value.0
    }
}

impl Deref for FlexU64 {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Value, from_bytes, to_value, to_writer_binary};

    #[test]
    fn accepts_integer_and_string_boundaries_and_serializes_integers() {
        for number in [0, 100000, u64::MAX] {
            for input in [Value::from(number), Value::from(number.to_string())] {
                let mut bytes = Vec::new();
                to_writer_binary(&mut bytes, &input).unwrap();
                let value: FlexU64 = from_bytes(&bytes).unwrap();
                assert_eq!(*value, number);
                assert_eq!(u64::from(value), number);
                assert_eq!(to_value(&value).unwrap(), Value::from(number));
            }
        }
        assert_eq!(*FlexU64::default(), 0);
        assert_eq!(*FlexU64::from(42), 42);
    }

    #[test]
    fn rejects_invalid_or_out_of_range_numbers() {
        for input in [
            Value::from(""),
            Value::from("abc"),
            Value::from("-1"),
            Value::from("18446744073709551616"),
            Value::from("1.5"),
            Value::from(-1i64),
            Value::from(true),
            Value::from(1.5f64),
        ] {
            let mut bytes = Vec::new();
            to_writer_binary(&mut bytes, &input).unwrap();
            assert!(from_bytes::<FlexU64>(&bytes).is_err(), "accepted {input:?}");
        }
    }
}
