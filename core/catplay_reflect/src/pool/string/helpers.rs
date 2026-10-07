//! Compile-time string layout and lookup helpers for exported pool macros.

/// A compile-time intermediate. It is not part of the runtime string pool.
pub struct StringLayout<const N: usize> {
    pub ids: [u16; N],
    pub offsets: [u16; N],
    pub byte_len: usize,
    pub unique_len: usize,
}

/// At most half-full, power-of-two table used only during const evaluation.
pub const fn string_hash_slots(count: usize) -> usize {
    count
        .checked_mul(2)
        .expect("catplay_reflect: too many string declarations")
        .checked_next_power_of_two()
        .expect("catplay_reflect: too many string declarations")
}

const fn bytes_equal(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut i = 0;
    while i < left.len() {
        if left[i] != right[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn string_hash(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut i = 0;
    while i < bytes.len() {
        // Every declaration reaches this pass, including duplicate values.
        assert!(bytes[i] != 0, "catplay_reflect: strings must not contain NUL bytes");
        hash = (hash ^ bytes[i] as u64).wrapping_mul(0x0000_0100_0000_01b3);
        i += 1;
    }
    hash
}

/// Const-only index for resolving declared strings without scanning the byte pool.
pub const fn string_lookup_table<const N: usize, const H: usize>(input: &[&str; N]) -> [usize; H] {
    let mut slots = [usize::MAX; H];
    let mut index = 0;
    while index < N {
        let bytes = input[index].as_bytes();
        let mut slot = (string_hash(bytes) as usize) & (H - 1);
        loop {
            let previous = slots[slot];
            if previous == usize::MAX {
                slots[slot] = index;
                break;
            }
            if bytes_equal(bytes, input[previous].as_bytes()) {
                break;
            }
            slot = (slot + 1) & (H - 1);
        }
        index += 1;
    }
    slots
}

/// Return the declaration index, including aliases, for an exact match.
pub const fn lookup_declared<const N: usize, const H: usize>(input: &[&str; N], slots: &[usize; H], value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == 0 {
            return None;
        }
        i += 1;
    }
    let mut slot = (string_hash(bytes) as usize) & (H - 1);
    loop {
        let previous = slots[slot];
        if previous == usize::MAX {
            return None;
        }
        let index = previous;
        if bytes_equal(bytes, input[index].as_bytes()) {
            return Some(index);
        }
        slot = (slot + 1) & (H - 1);
    }
}

/// Deduplicate exact strings and assign byte offsets before linking.
///
/// A one-byte prefix reserves offset zero for a direct u16 handle. A dense
/// u8 pool uses no prefix, since zero is reserved in its separate ID space.
pub const fn string_layout<const N: usize, const H: usize>(input: &[&str; N], prefix: usize) -> StringLayout<N> {
    assert!(prefix <= 1, "catplay_reflect: invalid string pool prefix");
    assert!(H == string_hash_slots(N), "catplay_reflect: invalid hash table size");
    let mut slots = [usize::MAX; H];
    let mut result = StringLayout {
        ids: [0; N],
        offsets: [0; N],
        byte_len: prefix,
        unique_len: 0,
    };
    let mut i = 0;
    while i < N {
        let bytes = input[i].as_bytes();
        assert!(
            bytes.len() <= 65_535 - prefix,
            "catplay_reflect: a string plus its NUL terminator and pool prefix exceeds 64 KiB"
        );
        let mut slot = (string_hash(bytes) as usize) & (H - 1);
        loop {
            let previous = slots[slot];
            if previous == usize::MAX {
                let record_len = 1 + bytes.len();
                assert!(
                    record_len <= 65_536 - result.byte_len,
                    "catplay_reflect: deduplicated string pool exceeds 64 KiB"
                );
                assert!(
                    result.unique_len < u16::MAX as usize,
                    "catplay_reflect: too many unique strings for u16"
                );
                // Every record includes its terminator, so its start is at
                // most 65_535. Check the ID width independently of byte size.
                result.ids[i] = (result.unique_len + 1) as u16;
                result.offsets[i] = result.byte_len as u16;
                result.byte_len += record_len;
                result.unique_len += 1;
                slots[slot] = i;
                break;
            }
            if bytes_equal(bytes, input[previous].as_bytes()) {
                result.ids[i] = result.ids[previous];
                result.offsets[i] = result.offsets[previous];
                break;
            }
            slot = (slot + 1) & (H - 1);
        }
        i += 1;
    }
    result
}

/// Produce the immutable runtime array of UTF-8 records terminated by NUL.
pub const fn pack_strings<const N: usize, const B: usize>(input: &[&str; N], layout: &StringLayout<N>, prefix: usize) -> [u8; B] {
    assert!(prefix <= 1, "catplay_reflect: invalid string pool prefix");
    assert!(B == layout.byte_len, "catplay_reflect: invalid output size");
    // Zero initialization also writes the optional reserved NUL prefix.
    let mut out = [0; B];
    let mut next = prefix;
    let mut i = 0;
    while i < N {
        let pos = layout.offsets[i] as usize;
        // A duplicate points to an already written record.
        if pos == next {
            let src = input[i].as_bytes();
            let destination = out.split_at_mut(pos).1;
            destination.split_at_mut(src.len()).0.copy_from_slice(src);
            out[pos + src.len()] = 0;
            next += 1 + src.len();
        }
        i += 1;
    }
    assert!(next == B, "catplay_reflect: inconsistent string layout");
    out
}

/// One u16 byte offset per unique string for a dense u8 pool.
pub const fn string_offsets<const N: usize, const U: usize>(layout: &StringLayout<N>) -> [u16; U] {
    assert!(U == layout.unique_len, "catplay_reflect: invalid index size");
    let mut offsets = [0; U];
    let mut i = 0;
    while i < N {
        offsets[layout.ids[i] as usize - 1] = layout.offsets[i];
        i += 1;
    }
    offsets
}

/// Scan to the NUL terminator and return the preceding record bytes.
///
/// Panics if the offset is out of bounds or no terminator follows it. All
/// accesses are checked; this function makes no UTF-8 claim.
pub const fn record_bytes(pool: &[u8], offset: usize) -> &[u8] {
    // CStr's word-at-a-time scan adds code on small targets, even with LTO.
    let mut end = offset;
    while pool[end] != 0 {
        end += 1;
    }
    pool.split_at(offset).1.split_at(end - offset).0
}

/// Test membership in a well-formed pool with a reserved leading NUL.
///
/// NUL-free record contents make every byte after a terminator a record start,
/// unless it is past the used pool length. This test remains memory-safe for
/// arbitrary input bytes; it does not validate the prefix or UTF-8 itself.
pub const fn is_string_offset(pool: &[u8], raw: u16) -> bool {
    let offset = raw as usize;
    raw != 0 && offset < pool.len() && pool[offset - 1] == 0
}

/// Return the dense ordinal of a direct offset by counting preceding records.
///
/// The reserved prefix is not a record. Panics on an invalid prefix, an
/// out-of-bounds offset, or an offset inside a record. No UTF-8 claim is made.
pub const fn string_index(pool: &[u8], offset: usize) -> usize {
    assert!(
        offset != 0 && offset < pool.len() && pool[0] == 0 && pool[offset - 1] == 0,
        "catplay_reflect: invalid direct string offset"
    );
    let mut index = 0;
    let mut pos = 1;
    while pos < offset {
        if pool[pos] == 0 {
            index += 1;
        }
        pos += 1;
    }
    index
}

/// Return an encoded handle by exact content, without changing the pool.
///
/// Direct u16 pools skip the reserved NUL and return a byte offset. Dense u8
/// pools begin at byte zero and return a one-based record ordinal.
pub const fn lookup_string<const DIRECT: bool>(pool: &[u8], value: &str) -> Option<u16> {
    let mut pos = if DIRECT { 1 } else { 0 };
    let mut raw = 1usize;
    while pos < pool.len() {
        let bytes = record_bytes(pool, pos);
        // Compare the entire query: an embedded NUL cannot match a record
        // whose bytes exclude its terminator.
        if bytes_equal(bytes, value.as_bytes()) {
            let encoded = if DIRECT { pos } else { raw };
            assert!(encoded <= u16::MAX as usize, "catplay_reflect: a string ID or offset exceeds u16");
            return Some(encoded as u16);
        }
        pos += 1 + bytes.len();
        if !DIRECT {
            raw += 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{is_string_offset, lookup_string, record_bytes, string_index};

    #[test]
    fn record_scan_checks_termination_without_validating_utf8() {
        assert_eq!(record_bytes(b"skip\0\xffx\0tail", 5), b"\xffx");
        assert_eq!(record_bytes(b"\0", 0), b"");
    }

    #[test]
    #[should_panic]
    fn record_offset_must_be_inside_the_pool() {
        record_bytes(b"x\0", 2);
    }

    #[test]
    #[should_panic]
    fn record_must_have_a_terminator() {
        record_bytes(b"unterminated", 0);
    }

    #[test]
    fn lookup_counters_do_not_wrap_on_many_empty_records() {
        // This artificial buffer is not a deduplicated generated pool. The
        // safe helper must still handle scanning it without a u16 overflow.
        let pool = [0u8; 65_536];
        assert_eq!(lookup_string::<false>(&pool, "missing"), None);
        assert_eq!(lookup_string::<true>(&pool, "missing"), None);
    }

    #[test]
    #[should_panic(expected = "catplay_reflect: a string ID or offset exceeds u16")]
    fn lookup_cannot_return_a_wrapped_id_from_an_invalid_pool() {
        let mut pool = [0u8; 65_537];
        pool[65_535] = b'x';
        lookup_string::<false>(&pool, "x");
    }

    #[test]
    #[should_panic(expected = "catplay_reflect: a string ID or offset exceeds u16")]
    fn lookup_cannot_return_a_wrapped_offset_from_an_invalid_pool() {
        let mut pool = [0u8; 65_538];
        pool[65_536] = b'x';
        lookup_string::<true>(&pool, "x");
    }

    #[test]
    fn direct_membership_checks_bounds_without_validating_utf8() {
        let pool = b"\0\xff\0\0x\0";
        for raw in 0..=7 {
            assert_eq!(is_string_offset(pool, raw), matches!(raw, 1 | 3 | 4));
        }
        assert!(!is_string_offset(pool, u16::MAX));
        assert!(!is_string_offset(b"", 0));
        assert!(!is_string_offset(b"", 1));
        assert!(!is_string_offset(b"\0", 1));
    }

    #[test]
    fn direct_index_counts_empty_and_non_utf8_records() {
        let pool = b"\0\xff\0\0x\0";
        assert_eq!(string_index(pool, 1), 0);
        assert_eq!(string_index(pool, 3), 1);
        assert_eq!(string_index(pool, 4), 2);
    }

    #[test]
    #[should_panic(expected = "catplay_reflect: invalid direct string offset")]
    fn direct_index_rejects_an_interior_byte() {
        string_index(b"\0ab\0", 2);
    }

    #[test]
    #[should_panic(expected = "catplay_reflect: invalid direct string offset")]
    fn direct_index_checks_bounds_before_reading() {
        string_index(b"", usize::MAX);
    }

    #[test]
    #[should_panic(expected = "catplay_reflect: invalid direct string offset")]
    fn direct_index_rejects_an_invalid_prefix() {
        string_index(b"x\0a\0", 2);
    }

    #[test]
    fn direct_lookup_does_not_treat_the_prefix_as_an_empty_record() {
        assert_eq!(lookup_string::<true>(b"\0x\0", "x"), Some(1));
        assert_eq!(lookup_string::<true>(b"\0x\0", ""), None);
        assert_eq!(lookup_string::<true>(b"\0x\0\0", ""), Some(3));
        assert_eq!(lookup_string::<true>(b"\0", ""), None);
        assert_eq!(lookup_string::<true>(b"", ""), None);
    }
}
