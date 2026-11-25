//! Shared XOR for cached AES and ChaCha keystreams.

// Keep the generic byte loop for NEON-capable ARM and other targets: it has
// been tested to produce optimally vectorized code for ARMv7 NEON.
#[cfg(not(any(
    target_arch = "mips",
    target_arch = "riscv32",
    all(target_arch = "arm", not(target_feature = "neon"))
)))]
pub(crate) fn xor(dst: &mut [u8], keystream: &[u8]) {
    debug_assert_eq!(dst.len(), keystream.len());
    for (byte, key) in dst.iter_mut().zip(keystream) {
        *byte ^= key;
    }
}

#[cfg(any(
    target_arch = "mips",
    target_arch = "riscv32",
    all(target_arch = "arm", not(target_feature = "neon"))
))]
#[inline(always)]
fn xor_word(dst: &mut u32, src: *const u8) {
    // SAFETY: Caller guarantees at least 4 readable bytes.
    let key = unsafe { src.cast::<u32>().read_unaligned() };
    *dst ^= key;
}

#[cfg(any(
    target_arch = "mips",
    target_arch = "riscv32",
    all(target_arch = "arm", not(target_feature = "neon"))
))]
pub(crate) fn xor(dst: &mut [u8], keystream: &[u8]) {
    debug_assert_eq!(dst.len(), keystream.len());

    // SAFETY: u32 accepts every bit pattern.
    let (head, words, tail) = unsafe { dst.align_to_mut::<u32>() };

    let (key_head, rest) = keystream.split_at(head.len());
    let (key_body, key_tail) = rest.split_at(words.len() * 4);

    for (d, k) in head.iter_mut().zip(key_head) {
        *d ^= *k;
    }

    let mut chunks = words.chunks_exact_mut(8);
    let mut key_ptr = key_body.as_ptr();

    for chunk in chunks.by_ref() {
        xor_word(&mut chunk[0], key_ptr);
        xor_word(&mut chunk[1], unsafe { key_ptr.add(4) });
        xor_word(&mut chunk[2], unsafe { key_ptr.add(8) });
        xor_word(&mut chunk[3], unsafe { key_ptr.add(12) });
        xor_word(&mut chunk[4], unsafe { key_ptr.add(16) });
        xor_word(&mut chunk[5], unsafe { key_ptr.add(20) });
        xor_word(&mut chunk[6], unsafe { key_ptr.add(24) });
        xor_word(&mut chunk[7], unsafe { key_ptr.add(28) });

        // SAFETY: Stays within key_body or one-past-the-end.
        key_ptr = unsafe { key_ptr.add(32) };
    }

    for word in chunks.into_remainder() {
        xor_word(word, key_ptr);
        key_ptr = unsafe { key_ptr.add(4) };
    }

    for (d, k) in tail.iter_mut().zip(key_tail) {
        *d ^= *k;
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
