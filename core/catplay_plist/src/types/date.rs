use alloc::string::String;
use core::{fmt, fmt::Write, time::Duration};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use time::{OffsetDateTime, UtcDateTime, format_description::well_known::Rfc3339};

/// UTC timestamp represented by a plist date object.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct Date {
    inner: UtcDateTime,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct InvalidXmlDate;

impl Date {
    const EPOCH_SECONDS: u64 = 978_307_200;

    pub fn from_xml_format(text: &str) -> Result<Self, InvalidXmlDate> {
        let date = OffsetDateTime::parse(text, &Rfc3339).map_err(|_| InvalidXmlDate)?;
        Ok(Self { inner: date.to_utc() })
    }

    pub fn to_xml_format(&self) -> String {
        self.xml_text().as_str().into()
    }

    pub(crate) fn xml_text(&self) -> XmlDateText {
        let date = self.inner;
        let mut output = XmlDateText { bytes: [0; 40], len: 0 };
        write!(
            output,
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            date.year(),
            date.month() as u8,
            date.day(),
            date.hour(),
            date.minute(),
            date.second()
        )
        .expect("date fits in XML buffer");
        if date.nanosecond() != 0 {
            write!(output, ".{:09}", date.nanosecond()).expect("date fits in XML buffer");
            while output.bytes[output.len - 1] == b'0' {
                output.len -= 1;
            }
        }
        output.write_char('Z').expect("date fits in XML buffer");
        output
    }

    /// Converts binary-plist seconds since 2001-01-01 to a date.
    pub fn from_seconds_since_plist_epoch(seconds: f64) -> Option<Self> {
        let duration = Duration::try_from_secs_f64(seconds.abs()).ok()?;
        let epoch = UtcDateTime::from_unix_timestamp(Self::EPOCH_SECONDS as i64).ok()?;
        let duration = time::Duration::try_from(duration).ok()?;
        let inner = if seconds < 0.0 {
            epoch.checked_sub(duration)?
        } else {
            epoch.checked_add(duration)?
        };
        Some(Self { inner })
    }

    /// Returns binary-plist seconds since 2001-01-01.
    pub fn as_seconds_since_plist_epoch(self) -> f64 {
        // Converting total nanoseconds to f64 first loses bits before division,
        // even for exactly representable binary fractions such as .125.
        (self.inner.unix_timestamp() - Self::EPOCH_SECONDS as i64) as f64 + f64::from(self.inner.nanosecond()) / 1_000_000_000.0
    }
}

// Enough for a signed six-digit year, nanoseconds, and the UTC suffix.
pub(crate) struct XmlDateText {
    bytes: [u8; 40],
    len: usize,
}
impl XmlDateText {
    pub(crate) fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).expect("formatted date is ASCII")
    }
}
impl fmt::Write for XmlDateText {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        self.bytes
            .get_mut(self.len..end)
            .ok_or(fmt::Error)?
            .copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl fmt::Debug for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.xml_text().as_str())
    }
}
impl fmt::Display for InvalidXmlDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("String was not a valid XML plist date")
    }
}
impl core::error::Error for InvalidXmlDate {}

#[cfg(feature = "serde")]
impl Serialize for Date {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_newtype_struct(crate::PLIST_DATE, self.xml_text().as_str())
    }
}
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Date {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Date;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a plist date")
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Date, E> {
                Date::from_xml_format(value).map_err(|_| E::custom("invalid plist date"))
            }
            fn visit_newtype_struct<D: Deserializer<'de>>(self, deserializer: D) -> Result<Date, D::Error> {
                deserializer.deserialize_str(self)
            }
        }
        deserializer.deserialize_newtype_struct(crate::PLIST_DATE, Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Date;

    #[test]
    fn utc_format_and_plist_epoch_preserve_fractional_seconds() {
        let date = Date::from_xml_format("2001-01-01T01:30:00.125000000+01:30").unwrap();
        assert_eq!(date.to_xml_format(), "2001-01-01T00:00:00.125Z");
        assert_eq!(date.as_seconds_since_plist_epoch(), 0.125);
        assert_eq!(
            Date::from_seconds_since_plist_epoch(-0.25)
                .unwrap()
                .to_xml_format(),
            "2000-12-31T23:59:59.75Z"
        );
    }
}
