use crate::Value;
use core::fmt;

pub const PLIST_PRETTY_DATA_LIMIT: usize = 256;

pub struct PrettyPlistValue<'a> {
    value: &'a Value,
    data_limit: usize,
}

#[inline]
pub fn pretty_plist_value(value: &Value) -> PrettyPlistValue<'_> {
    pretty_plist_value_with_limit(value, PLIST_PRETTY_DATA_LIMIT)
}

#[inline]
pub fn pretty_plist_value_with_limit(value: &Value, data_limit: usize) -> PrettyPlistValue<'_> {
    PrettyPlistValue { value, data_limit }
}

#[inline]
fn write_indent(f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
    for _ in 0..depth {
        f.write_str("  ")?;
    }
    Ok(())
}

fn fmt_data(bytes: &[u8], data_limit: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let shown = bytes.len().min(data_limit);

    write!(f, "Data({}b", bytes.len())?;
    if shown == 0 {
        return f.write_str(")");
    }

    f.write_str(": ")?;
    for (idx, b) in bytes[..shown].iter().enumerate() {
        if idx != 0 {
            f.write_str(" ")?;
        }
        write!(f, "{b:02x}")?;
    }
    if shown < bytes.len() {
        write!(f, " ... +{}b", bytes.len() - shown)?;
    }
    f.write_str(")")
}

fn fmt_plist_value(value: &Value, depth: usize, data_limit: usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match value {
        Value::Array(items) => {
            if items.is_empty() {
                return f.write_str("[]");
            }

            f.write_str("[\n")?;
            for (idx, item) in items.iter().enumerate() {
                write_indent(f, depth + 1)?;
                fmt_plist_value(item, depth + 1, data_limit, f)?;
                if idx + 1 != items.len() {
                    f.write_str(",")?;
                }
                f.write_str("\n")?;
            }
            write_indent(f, depth)?;
            f.write_str("]")
        }
        Value::Dictionary(map) => {
            if map.is_empty() {
                return f.write_str("{}");
            }

            f.write_str("{\n")?;
            for (idx, (key, entry)) in map.iter().enumerate() {
                write_indent(f, depth + 1)?;
                write!(f, "{key:?}: ")?;
                fmt_plist_value(entry, depth + 1, data_limit, f)?;
                if idx + 1 != map.len() {
                    f.write_str(",")?;
                }
                f.write_str("\n")?;
            }
            write_indent(f, depth)?;
            f.write_str("}")
        }
        Value::Boolean(v) => write!(f, "{v}"),
        Value::Data(v) => fmt_data(v, data_limit, f),
        Value::Date(v) => write!(f, "{v:?}"),
        Value::Real(v) => write!(f, "{v}"),
        Value::Integer(v) => write!(f, "{v:?}"),
        Value::String(v) => write!(f, "{v:?}"),
        Value::Uid(v) => write!(f, "Uid({v:?})"),
    }
}

impl fmt::Display for PrettyPlistValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_plist_value(self.value, 0, self.data_limit, f)
    }
}

impl fmt::Debug for PrettyPlistValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
