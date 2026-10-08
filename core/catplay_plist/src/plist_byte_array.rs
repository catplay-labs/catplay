#[cfg(feature = "serde")]
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use bytes::Bytes;
use core::{fmt, ops::Deref};
#[cfg(feature = "serde")]
use serde::{de, ser};

#[derive(Clone, PartialEq, Eq)]
pub struct PlistByteArray(pub(crate) Bytes);

impl From<Vec<u8>> for PlistByteArray {
    fn from(from: Vec<u8>) -> Self {
        PlistByteArray(from.into())
    }
}

impl<const N: usize> From<[u8; N]> for PlistByteArray {
    fn from(from: [u8; N]) -> Self {
        PlistByteArray(from.to_vec().into())
    }
}

impl From<PlistByteArray> for Vec<u8> {
    fn from(from: PlistByteArray) -> Self {
        from.0.into()
    }
}

impl AsRef<[u8]> for PlistByteArray {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl fmt::Debug for PlistByteArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(all(debug_assertions, not(test)))]
        {
            const MAX_TRACE: usize = 256;
            return write_hexdump_limited(f, self.as_ref(), MAX_TRACE);
        }

        #[cfg(any(not(debug_assertions), test))]
        write!(f, "<plist binary {}b>", self.as_ref().len())
    }
}

#[cfg(all(debug_assertions, not(test)))]
fn write_hexdump_limited(f: &mut fmt::Formatter<'_>, data: &[u8], max: usize) -> fmt::Result {
    use fmt::Write;

    const BYTES_PER_LINE: usize = 16;
    let shown = &data[..data.len().min(max)];
    for (line, chunk) in shown.chunks(BYTES_PER_LINE).enumerate() {
        write!(f, "{:08x}: ", line * BYTES_PER_LINE)?;
        for byte in chunk {
            write!(f, "{byte:02x} ")?;
        }
        for _ in chunk.len()..BYTES_PER_LINE {
            f.write_str("   ")?;
        }
        f.write_char('|')?;
        for byte in chunk {
            f.write_char(if byte.is_ascii_graphic() || *byte == b' ' {
                *byte as char
            } else {
                '.'
            })?;
        }
        f.write_str("|\n")?;
    }
    if data.len() > max {
        write!(f, " ... was a partial trace of total {} bytes ...\n", data.len())?;
    }
    Ok(())
}

impl Default for PlistByteArray {
    fn default() -> Self {
        Self(Bytes::new())
    }
}

impl Deref for PlistByteArray {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[cfg(feature = "serde")]
impl ser::Serialize for PlistByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(self.as_ref())
    }
}

#[cfg(feature = "serde")]
struct DataVisitor;

#[cfg(feature = "serde")]
impl de::Visitor<'_> for DataVisitor {
    type Value = PlistByteArray;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_byte_buf(v.to_owned())
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(v.into())
    }
}

#[cfg(feature = "serde")]
impl<'de> de::Deserialize<'de> for PlistByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(DataVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::PlistByteArray;

    #[test]
    fn debug_in_unit_tests_shows_length_only() {
        assert_eq!(format!("{:?}", PlistByteArray::from([0, 1, 255])), "<plist binary 3b>");
    }
}
