//! Shared XOR for cached AES and ChaCha keystreams.

#[cfg(not(target_arch = "mips"))]
pub(crate) fn xor(dst: &mut [u8], keystream: &[u8]) {
    debug_assert_eq!(dst.len(), keystream.len());
    for (byte, key) in dst.iter_mut().zip(keystream) {
        *byte ^= key;
    }
}

// MIPS32 lowers unaligned u32 accesses to lwl/lwr and swl/swr, avoiding a
// loop per byte even at opt-level="s". On RISC-V32 these accesses instead
// expand to byte loads/stores plus shifts, so retain the simple loop there.
#[cfg(target_arch = "mips")]
pub(crate) fn xor(dst: &mut [u8], keystream: &[u8]) {
    debug_assert_eq!(dst.len(), keystream.len());
    let mut dst_words = dst.chunks_exact_mut(4);
    let mut key_words = keystream.chunks_exact(4);
    for (dst, src) in dst_words.by_ref().zip(key_words.by_ref()) {
        let lhs = u32::from_ne_bytes((&*dst).try_into().unwrap());
        let rhs = u32::from_ne_bytes(src.try_into().unwrap());
        dst.copy_from_slice(&(lhs ^ rhs).to_ne_bytes());
    }
    for (dst, src) in dst_words
        .into_remainder()
        .iter_mut()
        .zip(key_words.remainder())
    {
        *dst ^= src;
    }
}

#[cfg(test)]
mod tests {
    use super::xor;

    #[test]
    fn unaligned_fragments_and_guards() {
        let key: [u8; 144] = core::array::from_fn(|i| (i as u8).wrapping_mul(37));
        for dst_offset in 0..8 {
            for key_offset in 0..8 {
                for len in 0..=129 {
                    let mut data = [0xa5; 144];
                    xor(&mut data[dst_offset..dst_offset + len], &key[key_offset..key_offset + len]);
                    for (i, byte) in data.iter().enumerate() {
                        let expected = if (dst_offset..dst_offset + len).contains(&i) {
                            0xa5 ^ key[key_offset + i - dst_offset]
                        } else {
                            0xa5
                        };
                        assert_eq!(*byte, expected);
                    }
                }
            }
        }
    }
}
