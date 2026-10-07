use core::fmt;

/// Shared Debug formatter for all iAP1 bitflags with the same storage type.
#[doc(hidden)]
pub fn fmt_iap1_bitflags<Storage: Copy + Into<u64>>(
    bits: u64,
    known_bits: u64,
    storage_width: u32,
    names: &[&str],
    values: &[Storage],
    formatter: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let mut first = true;
    let mut remaining = bits;

    for (name, value) in names.iter().zip(values) {
        let value = (*value).into();
        if bits & value == value && remaining & value != 0 {
            if !first {
                formatter.write_str(" | ")?;
            }
            formatter.write_str(name)?;
            first = false;
            remaining &= !value;
        }
    }

    let unknown = bits & !known_bits;
    for bit in 0..storage_width {
        if unknown & (1u64 << bit) != 0 {
            if !first {
                formatter.write_str(" | ")?;
            }
            write!(formatter, "BIT_{bit}")?;
            first = false;
        }
    }

    if first {
        formatter.write_str("(empty)")?;
    }
    Ok(())
}
